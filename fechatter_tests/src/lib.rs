//! Fechatter 集成测试库
//!
//! 这个库包含了 Fechatter 项目的集成测试，包括：
//! - API 端到端测试
//! - NATS 消息传递测试
//! - 数据库集成测试
//! - 文件上传下载测试
//! - 实时通知测试

pub mod api_tests;
pub mod common;
pub mod database_tests;
pub mod file_tests;
pub mod nats_tests;
pub mod notification_tests;

// 重新导出常用的测试工具
pub use common::*;

pub fn add(left: usize, right: usize) -> usize {
  left + right
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let result = add(2, 2);
    assert_eq!(result, 4);
  }
}
