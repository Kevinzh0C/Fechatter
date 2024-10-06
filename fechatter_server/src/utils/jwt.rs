use crate::{AppError, User, config::AuthConfig, models::UserStatus};
use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

const JWT_ISSUER: &str = "fechatter-server";
const JWT_AUDIENCE: &str = "fechatter-web";
const JWT_LEEWAY: u64 = 60;
const JWT_EXPIRATION_TIME: usize = 6 * 60 * 60;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  sub: String, // User ID
  exp: usize,  // Expiration time (as UTC timestamp)
  iat: usize,  // Issued at (as UTC timestamp)
  aud: String, // Audience
  iss: String, // Issuer
  user: UserClaims,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserClaims {
  pub id: i64,
  pub fullname: String,
  pub email: String,
  pub status: UserStatus,
  pub created_at: DateTime<Utc>,
}

pub struct TokenManager {
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  validation: Validation,
}

impl Claims {
  fn new(user: &User) -> Self {
    let now = chrono::Utc::now().timestamp() as usize;
    Self {
      sub: user.id.to_string(),
      exp: now + JWT_EXPIRATION_TIME,
      iat: now,
      aud: JWT_AUDIENCE.to_string(),
      iss: JWT_ISSUER.to_string(),
      user: UserClaims {
        id: user.id,
        email: user.email.clone(),
        status: user.status.clone(),
        fullname: user.fullname.clone(),
        created_at: user.created_at,
      },
    }
  }
}

impl TokenManager {
  pub fn from_config(auth: &AuthConfig) -> Result<Self, AppError> {
    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.leeway = JWT_LEEWAY;
    validation.reject_tokens_expiring_in_less_than = 300;

    validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);

    validation.validate_aud = true;
    validation.set_audience(&[JWT_AUDIENCE]);
    validation.set_issuer(&[JWT_ISSUER]);

    let sk = auth.sk.replace("\\n", "\n");
    let pk = auth.pk.replace("\\n", "\n");

    Ok(Self {
      encoding_key: EncodingKey::from_ed_pem(sk.as_bytes())?,
      decoding_key: DecodingKey::from_ed_pem(pk.as_bytes())?,
      validation,
    })
  }

  pub fn generate_token(&self, user: &User) -> Result<String, AppError> {
    let claims = Claims::new(user);
    let header = Header::new(Algorithm::EdDSA);
    let token = encode(&header, &claims, &self.encoding_key)?;

    Ok(token)
  }

  pub fn verify_token(&self, token: &str) -> Result<UserClaims, AppError> {
    let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)?;
    let user_claims = UserClaims {
      id: token_data.claims.user.id,
      fullname: token_data.claims.user.fullname,
      email: token_data.claims.user.email,
      status: token_data.claims.user.status,
      created_at: token_data.claims.user.created_at,
    };

    Ok(user_claims)
  }
}

#[cfg(test)]
mod tests {

  use crate::{AppConfig, models::UserStatus};

  use super::*;
  use anyhow::Result;

  #[test]
  fn jwt_token_authentication_should_work() -> Result<()> {
    let config = AppConfig::load()?;
    let token_manager = TokenManager::from_config(&config.auth)?;

    let user = User {
      id: 1,
      fullname: "John Doe".to_string(),
      email: "john.doe@example.com".to_string(),
      password_hash: Default::default(),
      status: UserStatus::Active,
      created_at: chrono::Utc::now(),
    };
    let user_claims = UserClaims {
      id: user.id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    let token = token_manager.generate_token(&user)?;

    let user_claims2 = token_manager.verify_token(&token)?;
    assert_eq!(user_claims, user_claims2);

    Ok(())
  }
}
