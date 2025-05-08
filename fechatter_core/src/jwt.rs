pub fn internal_verify_token(&self, token: &str) -> Result<UserClaims, CoreError> {
  tracing::debug!(
    "TokenManager.internal_verify_token: token length={}",
    token.len()
  );

  if token.is_empty() {
    tracing::error!("TokenManager.internal_verify_token: Empty token provided");
    return Err(CoreError::Validation("Empty token".to_string()));
  }

  let token_parts: Vec<&str> = token.split('.').collect();
  // Check token format
  if token_parts.len() != 3 {
    tracing::error!("TokenManager.internal_verify_token: Invalid token format");
    return Err(CoreError::Validation("Invalid token format".to_string()));
  }

  match decode::<Claims>(token, &self.decoding_key, &self.validation) {
    Ok(token_data) => {
      tracing::debug!(
        "TokenManager.internal_verify_token: Successfully decoded token for user_id={}",
        token_data.claims.user.id
      );
      Ok(token_data.claims.user)
    }
    Err(e) => {
      tracing::error!(
        "TokenManager.internal_verify_token: Validation error: {:?}",
        e
      );
      Err(CoreError::Validation(e.to_string()))
    }
  }
}
