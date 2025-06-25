//! # Old Services - 旧服务兼容层
//!
//! **职责**: 保持向后兼容
//! **状态**: 废弃中，请使用新的workers模块

// 重新导出兼容层服务
#[deprecated(note = "Use workers::auth instead")]
pub use crate::services::application::compatibility::auth_app_service::*;

#[deprecated(note = "Use workers::chat instead")]
pub use crate::services::application::compatibility::chat_app_service::*;

#[deprecated(note = "Use workers::message instead")]
pub use crate::services::application::compatibility::message_app_service::*;

#[deprecated(note = "Use workers module instead")]
pub use crate::services::application::compatibility::user_app_service::*;
