//! # Builders Module - 构建器
//!
//! **职责**: 创建、配置和管理服务实例
//! **特点**: 依赖注入、生命周期管理

pub mod provider;

// 重新导出构建器相关服务
pub use provider::{ServiceProvider, ServiceProviderBuilder};
