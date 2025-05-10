use serde::{Deserialize, Serialize};

pub mod auth_service;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
}