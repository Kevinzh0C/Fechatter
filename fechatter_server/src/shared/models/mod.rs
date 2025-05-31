use serde::{Deserialize, Serialize};

// Shared domain models - business entities that can be used across layers
pub mod file;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatFile {
  pub workspace_id: i64,
  pub ext: String, // extract from the uploaded filename
  pub hash: String,
}
