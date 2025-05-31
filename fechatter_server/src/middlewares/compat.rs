//! # 兼容性扩展层
//!
//! **目的**: 在现有RouterExtensions基础上提供lib.rs期望的方法名
//! **设计**: 不破坏现有架构，只是添加别名方法

use crate::AppState;
use axum::Router;
use std::sync::Arc;

/// lib.rs兼容性扩展trait
///
/// **职责**: 为lib.rs提供期望的方法名，内部委托给现有的RouterExtensions
pub trait LibCompatRouterExt<S>: Sized {
  /// 基础认证中间件 - 委托给with_core_auth
  fn with_auth(self, state: AppState) -> Self;

  /// 工作空间认证中间件 - 委托给with_workspace_auth  
  fn with_workspace(self, state: AppState) -> Self;

  /// 聊天认证中间件 - 委托给with_chat_auth
  fn with_chat(self, state: AppState) -> Self;

  /// 完整认证中间件 - 委托给enhanced_auth_flow
  fn with_full_auth(self, state: AppState) -> Self;
}

impl<S> LibCompatRouterExt<S> for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn with_auth(self, state: AppState) -> Self {
    use crate::middlewares::extensions::RouterExtensions;
    self.with_core_auth(Arc::new(state))
  }

  fn with_workspace(self, state: AppState) -> Self {
    use crate::middlewares::extensions::RouterExtensions;
    self.with_workspace_auth(Arc::new(state))
  }

  fn with_chat(self, state: AppState) -> Self {
    use crate::middlewares::extensions::RouterExtensions;
    self.with_chat_auth(Arc::new(state))
  }

  fn with_full_auth(self, state: AppState) -> Self {
    use crate::middlewares::extensions::RouterExtensions;
    // 使用enhanced_auth - 包含auth + workspace + chat认证
    self
      .with_enhanced_auth(Arc::new(state.clone()))
      .with_workspace_auth(Arc::new(state.clone()))
      .with_chat_auth(Arc::new(state))
  }
}

// 重新导出兼容方法到prelude
pub mod compat_prelude {
  pub use super::LibCompatRouterExt;

  // TODO: 同时导出原有的扩展（当prelude模块存在时）
  // pub use crate::middlewares::prelude::*;
}
