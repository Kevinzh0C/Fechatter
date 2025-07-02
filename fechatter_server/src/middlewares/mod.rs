/// Middleware Modules - ONLY builder_old directory enabled
///
/// This module uses only the proven builder_old directory system.
/// All other middleware modules are disabled.
// ============================================================================
// OLD Builder System - ONLY the builder_old DIRECTORY is enabled
// ============================================================================
pub mod builder_old; // Use the builder_old directory

// ============================================================================
// Re-exports for Public API - ONLY from builder_old directory
// ============================================================================

// Re-export key items from builder_old
pub use builder_old::{
    builder::RouterExt, chat::verify_chat_membership_middleware, workspace::with_workspace_context,
};

// ============================================================================
// Extension-based Middleware Functions
// ============================================================================

use crate::AppState;
use axum::{
    extract::{Extension, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use fechatter_core::models::AuthUser;

/// Auth middleware that works with Extension
pub async fn auth_extension_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get AppState from Extension
    let state = req
        .extensions()
        .get::<AppState>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Extract auth header
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify token using direct method from token_manager
    let claims = state
        .token_manager()
        .verify_jwt_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let auth_user: AuthUser = claims.into();

    // Add AuthUser to extensions
    req.extensions_mut().insert(auth_user);

    Ok(next.run(req).await)
}

/// Workspace middleware that works with Extension
pub async fn workspace_extension_middleware(req: Request, next: Next) -> Response {
    // Simply pass through - workspace validation happens in handlers
    // This is a placeholder for future workspace-level middleware logic
    next.run(req).await
}

/// Chat membership middleware that works with Extension
pub async fn chat_extension_middleware(req: Request, next: Next) -> Response {
    // Simply pass through - chat membership validation happens in handlers
    // This is a placeholder for future chat-level middleware logic
    next.run(req).await
}

// ============================================================================
// Development Notes
// ============================================================================
//
// Current status: ONLY builder_old/ directory is active
// The builder_old.rs file is ignored
// builder_old/ directory has its own chat.rs and workspace.rs implementations
