//! Core AI services using ai_sdk
//!
//! This module provides basic AI operations using the ai_sdk library,
//! serving as the foundation for more complex AI features in fechatter_server.

pub mod ai_service_adapter;

pub use ai_service_adapter::AiServiceAdapter;

/// Re-export ai_sdk types for convenience
pub use ai_sdk::{
    AiAdapter, AiService, Message as AiMessage, OllamaAdapter, OpenaiAdapter, Role as AiRole,
};
