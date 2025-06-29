use axum::{Router, middleware::from_fn};
use fechatter_core::SetLayer as _;
use std::sync::Arc;
use tracing::{debug, info};

// Locally define type state markers instead of importing
// Authentication state markers
pub struct WithoutAuth;
pub struct WithAuth;

// Token refresh state markers
pub struct WithoutRefresh;
pub struct WithRefresh;

// Server-specific workspace and chat membership state markers
pub struct WithoutWorkspace;
pub struct WithWorkspace;
pub struct WithoutChatMembership;
pub struct WithChatMembership;

// Import required middleware functions
use super::{chat::verify_chat_membership_middleware, workspace::with_workspace_context};
use crate::AppState;
use axum::body::Body;
use axum::extract::{Extension, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use fechatter_core::jwt::TokenManager;
use fechatter_core::middlewares::custom_builder::{add_auth_middleware, add_refresh_middleware};
use fechatter_core::middlewares::server_time::ServerTimeLayer as _; // Import server time middleware
use fechatter_core::models::AuthUser;

// ============================================================================
// Simple Middleware Builder - Essential Features Only
// ============================================================================

/// Simple Middleware Builder - Only essential auth, chat, workspace
pub struct SimpleMiddlewareBuilder<S> {
  router: Router<S>,
  state: AppState,
}

impl<S> SimpleMiddlewareBuilder<S>
where
  S: Clone + Send + Sync + 'static,
{
  /// Create new simple middleware builder
  pub fn new(router: Router<S>, state: AppState) -> Self {
    Self {
      router,
      state,
    }
  }

  // ========================================================================
  // Core Authentication (from fechatter_core)
  // ========================================================================

  /// Add JWT authentication middleware with refresh token support (from core)
  pub fn with_auth(mut self) -> Self {
    debug!("[SIMPLE_BUILDER] Adding authentication middleware with refresh token support");
    // First add refresh token middleware, then auth middleware
    self.router = add_refresh_middleware(self.router, self.state.clone());
    self.router = add_auth_middleware(self.router, self.state.clone());
    self
  }

  // ========================================================================
  // Simple Workspace Middleware 
  // ========================================================================

  /// Add workspace validation middleware (simple version)
  pub fn with_workspace(mut self) -> Self {
    debug!("[SIMPLE_BUILDER] Adding simple workspace middleware");
    let state_clone = self.state.clone();
    self.router = self.router.layer(from_fn(move |req: Request<Body>, next: Next| {
      let state = state_clone.clone();
      async move {
        // Extract AuthUser from request
        let (mut parts, body) = req.into_parts();
        if let Some(auth_user) = parts.extensions.get::<AuthUser>().cloned() {
          let req = Request::from_parts(parts, body);
          with_workspace_context(
            State(state),
            axum::extract::Extension(auth_user),
            req,
            next,
          ).await
        } else {
          let req = Request::from_parts(parts, body);
          debug!("[SIMPLE_BUILDER] No AuthUser found, proceeding without workspace context");
          next.run(req).await
        }
      }
    }));
    self
  }

  // ========================================================================
  // Simple Chat Middleware
  // ========================================================================

  /// Add chat access control middleware (simple version)
  pub fn with_chat_access(mut self) -> Self {
    debug!("[SIMPLE_BUILDER] Adding simple chat access middleware");
    let state_clone = self.state.clone();
    self.router = self.router.layer(from_fn(move |req: Request<Body>, next: Next| {
      let state = state_clone.clone();
      async move {
        // Apply to all routes containing "/chat/" in the path
        if req.uri().path().contains("/chat/") {
          info!("[SIMPLE_BUILDER] Applying chat middleware to path: {}", req.uri().path());
          verify_chat_membership_middleware(state, req, next).await
        } else {
          info!("[SIMPLE_BUILDER] Skipping chat middleware for path: {}", req.uri().path());
          next.run(req).await
        }
      }
    }));
    self
  }

  // ========================================================================
  // Simple Authentication Check
  // ========================================================================

  /// Add simple authentication requirement (check AuthUser exists)
  pub fn with_auth_required(mut self) -> Self {
    debug!("[SIMPLE_BUILDER] Adding auth required middleware");
    self.router = self.router.layer(from_fn(|req: Request<Body>, next: Next| async move {
      if req.extensions().get::<AuthUser>().is_some() {
        next.run(req).await
      } else {
        (StatusCode::UNAUTHORIZED, "Authentication required").into_response()
      }
    }));
    self
  }

  // ========================================================================
  // Simple Middleware Stacks
  // ========================================================================

  /// Public routes (no authentication required)
  pub fn with_public_stack(self) -> Self {
    debug!("[SIMPLE_BUILDER] Applying public stack (no middleware)");
    self
  }

  /// Basic authenticated routes (Auth only)
  pub fn with_auth_stack(self) -> Self {
    debug!("[SIMPLE_BUILDER] Applying auth stack (auth only)");
    self.with_auth()
  }

  /// Workspace routes (Auth + Workspace)
  pub fn with_workspace_stack(self) -> Self {
    debug!("[SIMPLE_BUILDER] Applying workspace stack (auth + workspace)");
    self
      .with_auth()
      .with_workspace()
  }

  /// Chat routes (Auth + Chat access)
  pub fn with_chat_stack(self) -> Self {
    debug!("[SIMPLE_BUILDER] Applying chat stack (auth + chat access)");
    // Apply auth and chat access middleware
    let result = self
      .with_auth()
      .with_chat_access();
    debug!("[SIMPLE_BUILDER] Chat stack applied successfully");
    result
  }

  /// Chat with workspace routes (Auth + Workspace + Chat access)
  pub fn with_chat_workspace_stack(self) -> Self {
    debug!("[SIMPLE_BUILDER] Applying chat+workspace stack");
    self
      .with_auth()
      .with_workspace()
      .with_chat_access()
  }

  // ========================================================================
  // Build and Finalize
  // ========================================================================

  /// Build the final router with minimal infrastructure middleware
  pub fn build(self) -> Router<S> {
    debug!("[SIMPLE_BUILDER] Building final simple router");
    // Apply minimal infrastructure middleware (ServerTime, RequestId)
    self.router.set_layer()
  }
}

// ========================================================================
// Simple RouterExt Extension Trait
// ========================================================================

/// Simple router extension trait for middleware composition
pub trait SimpleRouterExt<S>: Sized {
  fn with_middlewares(self, state: AppState) -> SimpleMiddlewareBuilder<S>;

  // Convenient stack methods
  fn with_public_stack(self, state: AppState) -> Router<S>;
  fn with_auth_stack(self, state: AppState) -> Router<S>;
  fn with_workspace_stack(self, state: AppState) -> Router<S>;
  fn with_chat_stack(self, state: AppState) -> Router<S>;
  fn with_chat_workspace_stack(self, state: AppState) -> Router<S>;
}

impl<S> SimpleRouterExt<S> for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn with_middlewares(self, state: AppState) -> SimpleMiddlewareBuilder<S> {
    SimpleMiddlewareBuilder::new(self, state)
  }

  fn with_public_stack(self, state: AppState) -> Router<S> {
    self.with_middlewares(state).with_public_stack().build()
  }

  fn with_auth_stack(self, state: AppState) -> Router<S> {
    self.with_middlewares(state).with_auth_stack().build()
  }

  fn with_workspace_stack(self, state: AppState) -> Router<S> {
    self.with_middlewares(state).with_workspace_stack().build()
  }

  fn with_chat_stack(self, state: AppState) -> Router<S> {
    debug!("[ROUTER_EXT] 🧪 with_chat_stack() called!");
    let result = self.with_middlewares(state).with_chat_stack().build();
    debug!("[ROUTER_EXT] 🧪 with_chat_stack() completed!");
    result
  }

  fn with_chat_workspace_stack(self, state: AppState) -> Router<S> {
    self.with_middlewares(state).with_chat_workspace_stack().build()
  }
}

// ========================================================================
// Legacy Compatibility (for existing code)
// ========================================================================

// Export simple builder as default
pub use SimpleMiddlewareBuilder as MiddlewareBuilder;
pub use SimpleRouterExt as RouterExt;

// Legacy complex builder (temporarily disabled)
pub mod complex {
  // Complex business middleware builder can be imported separately if needed
  // This keeps the simple version as default
}
