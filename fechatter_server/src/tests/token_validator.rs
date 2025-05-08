#[cfg(test)]
mod tests {
  use chrono::Utc;
  use std::sync::Arc;

  use crate::{
    AppConfig, AppError,
    models::{User, UserStatus, jwt::TokenManager},
    utils::RefreshTokenAdaptor,
  };
  use fechatter_core::{TokenService, middlewares::TokenVerifier};

  #[test]
  fn token_parser_trait_should_work() -> Result<(), AppError> {
    let config = AppConfig::load()?;
    // Create a mock repository
    let pool = sqlx::PgPool::connect_lazy(&config.server.db_url).unwrap();
    let refresh_token_repo = Arc::new(RefreshTokenAdaptor::new(Arc::new(pool)));
    let token_manager = TokenManager::from_config(&config.auth, refresh_token_repo)?;

    let user = User {
      id: 1,
      fullname: "John Doe".to_string(),
      email: "john.doe@example.com".to_string(),
      password_hash: Default::default(),
      status: UserStatus::Active,
      created_at: Utc::now(),
      workspace_id: 1,
    };

    // Create user claims from user
    let user_claims = fechatter_core::models::jwt::UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    // Generate token using TokenService trait
    let token = token_manager.generate_token(&user_claims)?;

    // Parse token using TokenService trait explicitly
    let parsed_claims = <TokenManager as TokenService>::verify_token(&token_manager, &token)?;

    assert_eq!(parsed_claims.id, user.id);
    assert_eq!(parsed_claims.email, user.email);
    assert_eq!(parsed_claims.fullname, user.fullname);

    Ok(())
  }

  #[test]
  fn token_validator_trait_should_work() -> Result<(), AppError> {
    let config = AppConfig::load()?;
    // Create a mock repository
    let pool = sqlx::PgPool::connect_lazy(&config.server.db_url).unwrap();
    let refresh_token_repo = Arc::new(RefreshTokenAdaptor::new(Arc::new(pool)));
    let token_manager = TokenManager::from_config(&config.auth, refresh_token_repo)?;

    let user = User {
      id: 1,
      fullname: "John Doe".to_string(),
      email: "john.doe@example.com".to_string(),
      password_hash: Default::default(),
      status: UserStatus::Active,
      created_at: Utc::now(),
      workspace_id: 1,
    };

    // Create user claims from user
    let user_claims = fechatter_core::models::jwt::UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    // Generate token using TokenService trait
    let token = token_manager.generate_token(&user_claims)?;

    // Validate token using TokenVerifier trait explicitly
    let validated_claims = <TokenManager as TokenVerifier>::verify_token(&token_manager, &token)?;

    assert_eq!(validated_claims.id, user.id);
    assert_eq!(validated_claims.email, user.email);
    assert_eq!(validated_claims.fullname, user.fullname);

    Ok(())
  }
}
