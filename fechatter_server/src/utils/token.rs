use crate::{AppError};
use fechatter_core::models::jwt::UserClaims;

pub trait TokenParser {
  fn parse_token(&self, token: &str) -> Result<UserClaims, AppError>;
}

pub trait TokenValidator: TokenParser {
  fn validate_token(&self, token: &str) -> Result<UserClaims, AppError> {
    self.parse_token(token)
  }
}
