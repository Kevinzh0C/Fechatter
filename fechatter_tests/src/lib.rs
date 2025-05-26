//! Fechatter Integration Test Library
//!
//! This library contains integration tests for the Fechatter project, including:
//! - API end-to-end tests
//! - NATS messaging tests
//! - Database integration tests
//! - File upload/download tests
//! - Real-time notification tests

pub mod api_tests;
pub mod common;
pub mod database_tests;
pub mod file_tests;
pub mod nats_tests;
pub mod notification_tests;

// Additional test modules
mod auth_tests;
mod search_tests;
mod stress_tests;
mod workspace_tests;

// Re-export common test utilities
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
