use crate::{
  contracts::{AuthContext, AuthService},
  error::CoreError,
  models::jwt::{AuthTokens, RefreshTokenData, TokenManager, UserClaims},
  models::{AuthUser, CreateUser, SigninUser, UserId, UserStatus, WorkspaceId},
};
use async_trait::async_trait;
use chrono::Utc;

pub struct MockAuthService;

#[async_trait]
impl AuthService for MockAuthService {
  async fn signup(
    &self,
    _payload: &CreateUser,
    _auth_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    Ok(AuthTokens {
      access_token: "mock_access_token".to_string(),
      refresh_token: crate::models::jwt::RefreshTokenData {
        token: "mock_refresh_token".to_string(),
        expires_at: Utc::now() + chrono::Duration::hours(24),
        absolute_expires_at: Utc::now() + chrono::Duration::days(30),
      },
    })
  }

  async fn signin(
    &self,
    _payload: &SigninUser,
    _auth_context: Option<AuthContext>,
  ) -> Result<Option<AuthTokens>, CoreError> {
    Ok(Some(AuthTokens {
      access_token: "mock_access_token".to_string(),
      refresh_token: crate::models::jwt::RefreshTokenData {
        token: "mock_refresh_token".to_string(),
        expires_at: Utc::now() + chrono::Duration::hours(24),
        absolute_expires_at: Utc::now() + chrono::Duration::days(30),
      },
    }))
  }

  async fn refresh_token(
    &self,
    _refresh_token: &str,
    _auth_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    Ok(AuthTokens {
      access_token: "mock_new_access_token".to_string(),
      refresh_token: crate::models::jwt::RefreshTokenData {
        token: "mock_new_refresh_token".to_string(),
        expires_at: Utc::now() + chrono::Duration::hours(24),
        absolute_expires_at: Utc::now() + chrono::Duration::days(30),
      },
    })
  }

  async fn logout(&self, _refresh_token: &str) -> Result<(), CoreError> {
    Ok(())
  }

  async fn logout_all(&self, _user_id: UserId) -> Result<(), CoreError> {
    Ok(())
  }

  fn verify_token(&self, _token: &str) -> Result<UserClaims, CoreError> {
    Ok(UserClaims {
      id: UserId::new(1),
      fullname: "Mock User".to_string(),
      email: "mock@example.com".to_string(),
      workspace_id: WorkspaceId::new(1),
      status: UserStatus::Active,
      created_at: Utc::now(),
    })
  }

  fn user_from_claims(&self, claims: UserClaims) -> AuthUser {
    AuthUser {
      id: claims.id,
      fullname: claims.fullname,
      email: claims.email,
      status: claims.status,
      created_at: claims.created_at,
      workspace_id: claims.workspace_id,
    }
  }
}
