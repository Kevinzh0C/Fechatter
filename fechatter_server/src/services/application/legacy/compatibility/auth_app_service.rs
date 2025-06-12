//! # Authentication Application Service
//!
//! **职责**: 认证应用服务的兼容性模块
//! **原则**: 重新导出auth模块中的服务

// 重新导出workers::auth模块中的服务 - 简化版本，无trait
pub use crate::services::application::workers::auth::{AuthUserService, create_auth_user_service};

// 类型别名保持兼容性
pub type AuthService = AuthUserService;
