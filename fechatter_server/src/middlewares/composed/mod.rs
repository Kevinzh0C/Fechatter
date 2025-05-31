//! # Composed Middleware Module
//!
//! **Responsibility**: Provides complex middleware flow compositions
//! - auth_flows: Authentication and authorization flow compositions

pub mod auth_flows;

// Re-exports
pub use auth_flows::*;
