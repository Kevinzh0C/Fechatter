use chrono::Utc;
use fechatter_server::{
  AppConfig, AppError, User,
  models::UserStatus,
  utils::jwt::TokenManager,
  utils::token::{TokenParser, TokenValidator},
};

#[test]
fn token_parser_trait_should_work() -> Result<(), AppError> {
  let config = AppConfig::load()?;
  let token_manager = TokenManager::from_config(&config.auth)?;

  let user = User {
    id: 1,
    fullname: "John Doe".to_string(),
    email: "john.doe@example.com".to_string(),
    password_hash: Default::default(),
    status: UserStatus::Active,
    created_at: Utc::now(),
    workspace_id: 1,
  };

  let token = token_manager.generate_token(&user)?;

  let user_claims = token_manager.parse_token(&token)?;

  assert_eq!(user_claims.id, user.id);
  assert_eq!(user_claims.email, user.email);
  assert_eq!(user_claims.fullname, user.fullname);

  Ok(())
}

#[test]
fn token_validator_trait_should_work() -> Result<(), AppError> {
  let config = AppConfig::load()?;
  let token_manager = TokenManager::from_config(&config.auth)?;

  let user = User {
    id: 1,
    fullname: "John Doe".to_string(),
    email: "john.doe@example.com".to_string(),
    password_hash: Default::default(),
    status: UserStatus::Active,
    created_at: Utc::now(),
    workspace_id: 1,
  };

  let token = token_manager.generate_token(&user)?;

  let user_claims = token_manager.validate_token(&token)?;

  assert_eq!(user_claims.id, user.id);
  assert_eq!(user_claims.email, user.email);
  assert_eq!(user_claims.fullname, user.fullname);

  Ok(())
}
