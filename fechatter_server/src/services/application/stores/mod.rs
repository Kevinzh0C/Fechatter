//! # Stores Module - 应用存储
//!
//! **职责**: 缓存管理和数据存储策略
//! **原则**: 简化职责，专注核心缓存功能

pub mod cache;

// 重新导出核心缓存服务
pub use cache::{CacheDataType, CacheStrategyService, InvalidationPattern};
