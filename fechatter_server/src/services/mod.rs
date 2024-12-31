use serde::{Deserialize, Serialize};

pub mod auth_service;
mod workspace_service;
mod service_provider;

pub use workspace_service::WorkspaceService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
}
