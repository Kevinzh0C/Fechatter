//! # Legacy Module - 兼容
//!
//! **职责**: 向后兼容的旧代码
//! **特点**: 过渡性、临时性、逐步废弃

pub mod compatibility;
pub mod old_services;

// 重新导出兼容服务（标记为废弃）
pub use compatibility::*;
