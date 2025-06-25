//! # Compatibility Module
//!
//! **职责**: 向后兼容的服务包装器
//! **原则**: 保持API稳定性，平滑迁移

pub mod auth_app_service;
pub mod chat_app_service;
pub mod message_app_service;
pub mod user_app_service;

// 重新导出以保持兼容性
pub use auth_app_service::*;
pub use chat_app_service::*;
pub use message_app_service::*;
pub use user_app_service::*;
