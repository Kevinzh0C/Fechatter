//! # Message Application Service
//!
//! **职责**: 消息应用服务的兼容性模块
//! **原则**: 重新导出message模块中的服务

// 重新导出workers::message模块中的服务 - 简化版本，无trait和adapter
pub use crate::services::application::workers::message::{MessageApplicationService, MessageView};
