use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use fechatter_core::{
    contracts::repositories::UserRepository,
    contracts::AuthContext,
    error::CoreError,
    models::jwt::{AuthTokens, RefreshTokenData, UserClaims},
    models::{CreateUser, SigninUser, UserId},
};

use super::token_repository::{RefreshTokenRepository, ReplaceTokenPayload};

// Token service trait for core operations
#[async_trait]
pub trait TokenService: Send + Sync {
    fn generate_token(&self, user_claims: &UserClaims) -> Result<String, CoreError>;
    fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError>;
    async fn generate_auth_tokens(
        &self,
        user_claims: &UserClaims,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<AuthTokens, CoreError>;
}

/// Pure business logic authentication service
/// This mirrors the pattern from back_src/auth_Service.rs
/// Dependencies are injected as traits to keep the domain pure
pub struct AuthDomainService<U, T, R> {
    user_repository: U,
    token_service: T,
    refresh_token_repository: R,
}

impl<U, T, R> AuthDomainService<U, T, R>
where
    U: UserRepository + Send + Sync,
    T: TokenService + Send + Sync,
    R: RefreshTokenRepository + Send + Sync,
{
    pub fn new(user_repository: U, token_service: T, refresh_token_repository: R) -> Self {
        Self {
            user_repository,
            token_service,
            refresh_token_repository,
        }
    }

    /// User registration with complete token generation
    pub async fn signup(
        &self,
        payload: &CreateUser,
        auth_context: Option<AuthContext>,
    ) -> Result<AuthTokens, CoreError> {
        // Create user through repository
        let user = self.user_repository.create(payload).await?;

        // Create user claims
        let user_claims = UserClaims {
            id: user.id,
            workspace_id: user.workspace_id,
            fullname: user.fullname.clone(),
            email: user.email.clone(),
            status: user.status,
            created_at: user.created_at,
        };

        // Generate tokens using token service
        let auth_context_data = auth_context.unwrap_or_default();
        let auth_tokens = self
            .token_service
            .generate_auth_tokens(
                &user_claims,
                auth_context_data.user_agent,
                auth_context_data.ip_address,
            )
            .await?;

        Ok(auth_tokens)
    }

    /// User authentication and token generation
    pub async fn signin(
        &self,
        payload: &SigninUser,
        auth_context: Option<AuthContext>,
    ) -> Result<Option<AuthTokens>, CoreError> {
        // Authenticate user
        match self.user_repository.authenticate(payload).await? {
            Some(user) => {
                // Create user claims
                let user_claims = UserClaims {
                    id: user.id,
                    workspace_id: user.workspace_id,
                    fullname: user.fullname.clone(),
                    email: user.email.clone(),
                    status: user.status,
                    created_at: user.created_at,
                };

                // Generate tokens
                let auth_context_data = auth_context.unwrap_or_default();
                let auth_tokens = self
                    .token_service
                    .generate_auth_tokens(
                        &user_claims,
                        auth_context_data.user_agent,
                        auth_context_data.ip_address,
                    )
                    .await?;

                Ok(Some(auth_tokens))
            }
            None => Ok(None),
        }
    }

    /// Refresh token with rotation
    pub async fn refresh_token(
        &self,
        refresh_token_str: &str,
        auth_context: Option<AuthContext>,
    ) -> Result<AuthTokens, CoreError> {
        // Find refresh token
        let refresh_token = self
            .refresh_token_repository
            .find_by_token(refresh_token_str)
            .await?
            .ok_or_else(|| {
                CoreError::Unauthorized("Invalid or expired refresh token".to_string())
            })?;

        // Validate token
        let now = Utc::now();
        if refresh_token.expires_at < now {
            return Err(CoreError::Unauthorized(
                "Refresh token has expired".to_string(),
            ));
        }

        if refresh_token.revoked {
            return Err(CoreError::Unauthorized(
                "Refresh token has been revoked".to_string(),
            ));
        }

        if refresh_token.absolute_expires_at < now {
            return Err(CoreError::Unauthorized(
                "Refresh token has reached absolute expiration".to_string(),
            ));
        }

        // Security context validation
        if let Some(ctx) = &auth_context {
            if let Some(token_user_agent) = &refresh_token.user_agent {
                if let Some(request_user_agent) = &ctx.user_agent {
                    if token_user_agent != request_user_agent {
                        return Err(CoreError::Unauthorized(
                            "Authentication context mismatch".to_string(),
                        ));
                    }
                }
            }
        }

        // Get user - convert i64 to UserId
        let user = self
            .user_repository
            .find_by_id(UserId(refresh_token.user_id))
            .await?
            .ok_or_else(|| CoreError::Unauthorized("User not found".to_string()))?;

        // Create user claims
        let user_claims = UserClaims {
            id: user.id,
            workspace_id: user.workspace_id,
            fullname: user.fullname.clone(),
            email: user.email.clone(),
            status: user.status,
            created_at: user.created_at,
        };

        // Generate new access token
        let new_access_token = self.token_service.generate_token(&user_claims)?;

        // Generate new refresh token
        let new_raw_refresh_token = Uuid::new_v4().to_string();
        let new_expires_at = now
            + chrono::Duration::seconds(super::token_repository::REFRESH_TOKEN_EXPIRATION as i64);
        let new_absolute_expires_at = refresh_token.absolute_expires_at;

        // Replace old refresh token (token rotation)
        let auth_context_data = auth_context.unwrap_or_default();
        let replace_payload = ReplaceTokenPayload {
            old_token_id: refresh_token.id,
            new_raw_token: new_raw_refresh_token.clone(),
            new_expires_at,
            new_absolute_expires_at,
            user_agent: auth_context_data.user_agent,
            ip_address: auth_context_data.ip_address,
        };

        let new_refresh_token_record = self
            .refresh_token_repository
            .replace(replace_payload)
            .await?;

        // Create response
        let refresh_token_data = RefreshTokenData {
            token: new_raw_refresh_token,
            expires_at: new_refresh_token_record.expires_at,
            absolute_expires_at: new_refresh_token_record.absolute_expires_at,
        };

        Ok(AuthTokens {
            access_token: new_access_token,
            refresh_token: refresh_token_data,
        })
    }

    /// Logout - revoke refresh token
    pub async fn logout(&self, refresh_token_str: &str) -> Result<(), CoreError> {
        let refresh_token = self
            .refresh_token_repository
            .find_by_token(refresh_token_str)
            .await?
            .ok_or_else(|| CoreError::Unauthorized("Invalid refresh token".to_string()))?;

        self.refresh_token_repository
            .revoke(refresh_token.id)
            .await?;

        Ok(())
    }

    /// Logout all sessions - revoke all user tokens
    pub async fn logout_all(&self, user_id: UserId) -> Result<(), CoreError> {
        self.refresh_token_repository
            .revoke_all_for_user(user_id)
            .await?;
        Ok(())
    }
}

// Helper function
fn generate_refresh_token() -> String {
    Uuid::new_v4().to_string()
}
