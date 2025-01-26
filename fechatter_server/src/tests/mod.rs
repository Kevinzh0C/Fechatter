// 导入测试模块
mod auth_service_test;
#[cfg(test)]
mod auth_tests;
// #[cfg(test)]
// mod chat_member_tests;
// #[cfg(test)]
// mod chat_tests;
#[cfg(test)]
mod macros;
// #[cfg(test)]
// mod message_tests;
#[cfg(test)]
pub mod middleware_tests;
#[cfg(test)]
mod refresh_token_tests;
#[cfg(test)]
mod service_validation;
#[cfg(test)]
mod token_validator;
//mod builder_tests;
//mod workspace_tests;

#[cfg(test)]
pub mod search_tests_simplified;
// #[cfg(test)]
// mod test_app_state;
// #[cfg(test)]
// mod test_setup;
#[cfg(any(test, feature = "test-util"))]
pub mod test_utils;

// Re-export commonly used test utilities
// #[cfg(test)]
// pub use test_utils::*;

// Re-export test methods for external packages when test-util feature is enabled
#[cfg(feature = "test-util")]
pub use crate::AppState;
