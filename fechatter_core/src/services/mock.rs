use crate::models::jwt::RefreshTokenData;
use crate::{
  contracts::AuthService,
  error::CoreError,
  models::jwt::{AuthTokens, UserClaims},
  models::{AuthUser, CreateUser, SigninUser, UserId, UserStatus, WorkspaceId},
};
use chrono::Utc;
use std::future::Future;

pub struct MockAuthService;

impl crate::services::AuthService for MockAuthService {
  fn signup(
    &self,
    _payload: &CreateUser,
    _auth_context: Option<crate::services::AuthContext>,
  ) -> impl Future<Output = Result<AuthTokens, CoreError>> + Send {
    async move {
      Ok(AuthTokens {
        access_token: "mock_access_token".to_string(),
        refresh_token: RefreshTokenData {
          token: "mock_refresh_token".to_string(),
          expires_at: Utc::now(),
          absolute_expires_at: Utc::now(),
        },
      })
    }
  }

  fn signin(
    &self,
    _payload: &SigninUser,
    _auth_context: Option<crate::services::AuthContext>,
  ) -> impl Future<Output = Result<Option<AuthTokens>, CoreError>> + Send {
    async move {
      Ok(Some(AuthTokens {
        access_token: "mock_access_token".to_string(),
        refresh_token: RefreshTokenData {
          token: "mock_refresh_token".to_string(),
          expires_at: Utc::now(),
          absolute_expires_at: Utc::now(),
        },
      }))
    }
  }

  fn refresh_token(
    &self,
    _refresh_token: &str,
    _auth_context: Option<crate::services::AuthContext>,
  ) -> impl Future<Output = Result<AuthTokens, CoreError>> + Send {
    async move {
      Ok(AuthTokens {
        access_token: "mock_access_token".to_string(),
        refresh_token: RefreshTokenData {
          token: "mock_refresh_token".to_string(),
          expires_at: Utc::now(),
          absolute_expires_at: Utc::now(),
        },
      })
    }
  }

  fn logout(&self, _refresh_token: &str) -> impl Future<Output = Result<(), CoreError>> + Send {
    async move { Ok(()) }
  }

  fn logout_all(&self, _user_id: UserId) -> impl Future<Output = Result<(), CoreError>> + Send {
    async { Ok(()) }
  }

  fn verify_token(&self, _token: &str) -> Result<UserClaims, CoreError> {
    Ok(UserClaims {
      id: UserId::new(1),
      workspace_id: WorkspaceId::new(1),
      fullname: "Mock User".to_string(),
      email: "mock@example.com".to_string(),
      status: UserStatus::Active,
      created_at: Utc::now(),
    })
  }

  fn user_from_claims(&self, claims: UserClaims) -> AuthUser {
    AuthUser {
      id: claims.id,
      workspace_id: claims.workspace_id,
      fullname: claims.fullname,
      email: claims.email,
      status: claims.status,
      created_at: claims.created_at,
    }
  }
}
