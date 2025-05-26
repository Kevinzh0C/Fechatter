#[cfg(test)]
mod tests {
  use chrono::Utc;
  use std::sync::Arc;

  use crate::{
    AppConfig, AppError, AppState,
    models::{User, UserClaims, UserStatus, jwt::TokenManager},
    utils::RefreshTokenAdaptor,
    verify_token,
  };
  use fechatter_core::{TokenService, middlewares::TokenVerifier};

  #[tokio::test]
  async fn token_parser_trait_should_work() -> Result<(), AppError> {
    let config = AppConfig::load()?;
    // Create a mock repository
    let pool = sqlx::PgPool::connect_lazy(&config.server.db_url).unwrap();
    let refresh_token_repo = Arc::new(RefreshTokenAdaptor::new(Arc::new(pool)));
    let token_manager = TokenManager::from_config(&config.auth, refresh_token_repo)?;

    let user = User {
      id: fechatter_core::UserId(1),
      fullname: "John Doe".to_string(),
      email: "john.doe@example.com".to_string(),
      password_hash: Default::default(),
      status: UserStatus::Active,
      created_at: Utc::now(),
      workspace_id: fechatter_core::WorkspaceId(1),
    };

    // Create user claims from user
    let user_claims = fechatter_core::models::jwt::UserClaims {
      id: user.id,
      workspace_id: user.workspace_id.into(),
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    // Generate token using TokenService trait - 使用完全限定语法
    let token = <TokenManager as TokenService>::generate_token(&token_manager, &user_claims)?;

    // Parse token using TokenService trait explicitly - 使用完全限定语法
    let parsed_claims = <TokenManager as TokenService>::verify_token(&token_manager, &token)?;

    assert_eq!(parsed_claims.id, user.id);
    assert_eq!(parsed_claims.email, user.email);
    assert_eq!(parsed_claims.fullname, user.fullname);

    Ok(())
  }

  #[tokio::test]
  async fn token_validator_trait_should_work() -> Result<(), AppError> {
    let config = AppConfig::load()?;
    // Create a mock repository
    let pool = sqlx::PgPool::connect_lazy(&config.server.db_url).unwrap();
    let refresh_token_repo = Arc::new(RefreshTokenAdaptor::new(Arc::new(pool)));
    let token_manager = TokenManager::from_config(&config.auth, refresh_token_repo)?;

    let user = User {
      id: fechatter_core::UserId(1),
      fullname: "John Doe".to_string(),
      email: "john.doe@example.com".to_string(),
      password_hash: Default::default(),
      status: UserStatus::Active,
      created_at: Utc::now(),
      workspace_id: fechatter_core::WorkspaceId(1),
    };

    // Create user claims from user
    let user_claims = fechatter_core::models::jwt::UserClaims {
      id: user.id,
      workspace_id: user.workspace_id.into(),
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    // Generate token using TokenService trait - 使用完全限定语法
    let token = <TokenManager as TokenService>::generate_token(&token_manager, &user_claims)?;

    // Validate token using TokenVerifier trait explicitly
    let validated_claims = <TokenManager as TokenVerifier>::verify_token(&token_manager, &token)?;

    assert_eq!(validated_claims.id, user.id);
    assert_eq!(validated_claims.email, user.email);
    assert_eq!(validated_claims.fullname, user.fullname);

    Ok(())
  }

  #[tokio::test]
  async fn test_verify_token_macro() -> Result<(), anyhow::Error> {
    let config = AppConfig::load()?;
    let app_state = AppState::try_new(config).await?;

    let user_claims = UserClaims {
      id: fechatter_core::UserId(1),
      workspace_id: fechatter_core::WorkspaceId(1),
      fullname: "Test User".to_string(),
      email: "test@example.com".to_string(),
      status: fechatter_core::UserStatus::Active,
      created_at: chrono::Utc::now(),
    };

    let token =
      <fechatter_core::models::jwt::TokenManager as fechatter_core::TokenService>::generate_token(
        app_state.token_manager(),
        &user_claims,
      )?;

    let result = verify_token!(app_state, &token)?;
    assert_eq!(result.id, user_claims.id);

    Ok(())
  }
}
