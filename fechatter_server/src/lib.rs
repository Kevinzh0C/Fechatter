// ============================================================================
// Fechatter Server - Core Library Interface (Clean)
// ============================================================================
//
// **Responsibility**: Core public API - AppState definition, Router configuration
// **Principles**: Clean external interfaces, separated internal implementation

pub mod config;
pub mod domains;
pub mod dtos;
pub mod error;
pub mod handlers;
pub mod middlewares;
pub mod openapi;
pub mod services;
pub mod state;
pub mod tests;
pub mod utils;

// ============================================================================
// Core Imports for Public API
// ============================================================================

use axum::{
  extract::{Request},
  middleware::Next,
  response::Response,
  routing::{get, post},
  Router,
};
use std::{fmt, ops::Deref, sync::Arc};
use tracing::{debug, warn, info};
use tower_http::services::ServeDir;

// Import AuthUser and other necessary types
use fechatter_core::models::AuthUser;

// ============================================================================
// Core Type Re-exports - Essential public API
// ============================================================================

pub use config::AppConfig;
pub use error::{AppError, ErrorOutput};
pub use services::application::workers::auth::AuthUserService as AuthService;

// ============================================================================
// Handler Re-exports - HTTP Handlers
// ============================================================================

pub use handlers::*;

// ============================================================================
// Domain Re-exports - Business Logic
// ============================================================================

pub use domains::*;

// ============================================================================
// Core AppState Definition - Public API
// ============================================================================

/// Application State - Main state container for the application
#[derive(Debug, Clone)]
pub struct AppState {
  inner: Arc<AppStateInner>,
}

/// Internal state structure
pub struct AppStateInner {
  pub(crate) config: AppConfig,
  pub(crate) application_services: crate::services::application::builders::ServiceProvider,
  // Token manager stored separately for trait implementation
  pub(crate) token_manager: Arc<fechatter_core::models::jwt::TokenManager>,
  // Event publisher with production transport
  pub(crate) event_publisher:
    Option<Arc<crate::services::infrastructure::event::DynEventPublisher>>,
  // Unified event publisher for enhanced events (replaces scattered event publishers)
  pub(crate) unified_event_publisher:
    Option<Arc<crate::services::infrastructure::event::DynEventPublisher>>,
  // Enhanced event publisher for notify_server SSE integration
  pub(crate) enhanced_event_publisher:
    Option<Arc<crate::services::infrastructure::event::EnhancedEventPublisher>>,
  pub(crate) cache_service: Option<Arc<crate::services::infrastructure::cache::RedisCacheService>>,
  pub(crate) sync_cache_adapter: crate::services::infrastructure::cache::SyncCacheAdapter,
  // Unified analytics publisher using NATS + Protobuf
  pub(crate) analytics_publisher:
    Option<Arc<crate::services::infrastructure::event::NatsAnalyticsPublisher>>,
  // Cached auth service wrapper for middleware performance
  pub(crate) cached_auth_service:
    std::sync::RwLock<Option<Arc<crate::state::ProductionAuthServiceWrapper>>>,
}

// ============================================================================
// AppState Core Infrastructure Access - Public API
// ============================================================================

impl AppState {
  /// Get database pool
  #[inline]
  pub fn pool(&self) -> Arc<sqlx::PgPool> {
    self.inner.application_services.pool()
  }

  /// Get token manager
  #[inline]
  pub fn token_manager(&self) -> Arc<fechatter_core::models::jwt::TokenManager> {
    self.inner.application_services.token_manager()
  }

  /// Get Redis cache service
  #[inline]
  pub fn cache_service(
    &self,
  ) -> Option<&Arc<crate::services::infrastructure::cache::RedisCacheService>> {
    self.inner.cache_service.as_ref()
  }

  /// Get application services
  #[inline]
  pub fn application_services(&self) -> &crate::services::application::builders::ServiceProvider {
    &self.inner.application_services
  }

  /// Get event publisher
  #[inline]
  pub fn event_publisher_dyn(
    &self,
  ) -> Option<&Arc<crate::services::infrastructure::event::DynEventPublisher>> {
    self.inner.event_publisher.as_ref()
  }

  /// Get unified event publisher with enhanced functionality
  #[inline]
  pub fn unified_event_publisher(
    &self,
  ) -> Option<&Arc<crate::services::infrastructure::event::DynEventPublisher>> {
    self.inner.unified_event_publisher.as_ref()
  }

  /// Get enhanced event publisher for notify_server SSE integration
  #[inline]
  pub fn enhanced_event_publisher(
    &self,
  ) -> Option<&Arc<crate::services::infrastructure::event::EnhancedEventPublisher>> {
    self.inner.enhanced_event_publisher.as_ref()
  }

  /// Get unified analytics publisher
  #[inline]
  pub fn analytics_publisher(
    &self,
  ) -> Option<&Arc<crate::services::infrastructure::event::NatsAnalyticsPublisher>> {
    self.inner.analytics_publisher.as_ref()
  }

  /// Get NATS client from event publisher if available
  #[inline]
  pub fn nats_client(&self) -> Option<async_nats::Client> {
    use crate::services::infrastructure::event::NatsTransport;

    self.inner.event_publisher.as_ref().and_then(|publisher| {
      // Try to downcast the transport to NatsTransport
      let transport = publisher.transport();
      transport
        .as_any()
        .downcast_ref::<NatsTransport>()
        .map(|nats_transport| nats_transport.client().clone())
    })
  }

  /// Get search service
  #[inline]
  pub fn search_service(&self) -> Option<Arc<crate::services::SearchService>> {
    self.inner.application_services.search_service()
  }

  /// Check if search is enabled
  #[inline]
  pub fn is_search_enabled(&self) -> bool {
    self
      .search_service()
      .map(|service| service.is_enabled())
      .unwrap_or(false)
  }

  /// Get search service with application interface
  #[inline]
  pub fn search_application_service(
    &self,
  ) -> Option<Arc<dyn crate::services::application::workers::search::SearchApplicationServiceTrait>>
  {
    self
      .inner
      .application_services
      .search_service()
      .map(|infra_service| {
        Arc::new(
          crate::services::application::workers::search::SearchServiceAdapter::new(infra_service),
        )
          as Arc<dyn crate::services::application::workers::search::SearchApplicationServiceTrait>
      })
  }

  // ========================================================================
  // State Creation - Core Public API
  // ========================================================================

  /// Create AppState for tests
  #[cfg(test)]
  pub fn default_for_tests() -> Self {
    crate::state::create_test_state()
  }

  /// Create AppState - Main initialization method
  pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
    crate::state::create_production_state(config).await
  }
}

impl Deref for AppState {
  type Target = AppStateInner;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl fmt::Debug for AppStateInner {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "AppStateInner {{ config: {:?}, token_manager: <hidden> }}",
      self.config
    )
  }
}

/// Create database pool - Utility function
pub async fn create_pool(db_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
  crate::state::create_pool_impl_pub(db_url).await
}

/// Debug middleware to log route information
async fn route_debug_middleware(req: Request, next: Next) -> Response {
  let method = req.method().clone();
  let uri = req.uri().clone();
  let path = uri.path();

  debug!("🔍 [ROUTE_DEBUG] {} {} - Processing request", method, path);

  let response = next.run(req).await;
  let status = response.status();

  if status.is_client_error() || status.is_server_error() {
    warn!(
      "🔍 [ROUTE_DEBUG] {} {} - Response: {}",
      method, path, status
    );
  } else {
    debug!(
      "🔍 [ROUTE_DEBUG] {} {} - Response: {}",
      method, path, status
    );
  }

  response
}


/// This implementation uses ONLY Extension-based middleware to avoid Axum 0.7.9 with_state() bugs
/// All handlers use Extension<AppState> instead of State<AppState>
/// Returns Router<()> for complete type unification
pub async fn get_router(state: AppState) -> Result<Router, AppError> {
  use crate::middlewares::builder_old::builder::{
    create_extension_middleware_builder, create_stateless_router_with_routes,
  };

  // ============================================================================
  // Public routes (no auth required, but still need state)
  // ============================================================================
  let public_routes = create_stateless_router_with_routes(|router| {
    router
      .route("/signup", post(handlers::auth::signup_handler))
      .route("/signin", post(handlers::auth::signin_handler))
      .route("/refresh", post(handlers::auth::refresh_token_handler))
  });

  let public_routes = create_extension_middleware_builder(public_routes, state.clone())
    .with_state_extension()
    .finalize_extension_based();

  // ============================================================================
  // Authenticated routes (auth middleware required)
  // ============================================================================
  let auth_routes = create_stateless_router_with_routes(|router| {
    router
      .route("/logout", post(handlers::auth::logout_handler))
      .route("/logout-all", post(handlers::auth::logout_all_handler))
      .route("/cache/stats", get(handlers::cache_stats::get_cache_stats_handler))
      .route("/cache/config", get(handlers::cache_stats::get_cache_config_handler))
      // File management routes
      .route("/files/single", post(handlers::files::upload_single_file_handler))
      .route("/files/download/{file_id}", get(handlers::files::download_file_handler))
      // Global search routes
      .route("/search/messages", post(handlers::search::global_search_messages))
      // Simplified chat search route (only requires auth, not chat membership)
      .route("/search/chat/{chat_id}/messages", get(handlers::search::simple_search_messages_in_chat))
      // Realtime presence routes (user-level)
      .route("/realtime/presence", post(handlers::realtime::update_presence))
      // Unread counts routes
      .route("/unread-counts", get(handlers::messages::get_all_unread_counts_handler))
      // Mentions routes
      .route("/mentions/unread", get(handlers::messages::get_unread_mentions_handler))
      // Bot routes (require authentication and quota check)
      .route("/bot/translate", post(handlers::bot::translate_message_handler))
      .route("/bot/languages", get(handlers::bot::get_supported_languages_handler))
      .route("/bot/detect-language", post(handlers::bot::detect_language_handler))
  });

  let auth_routes = create_extension_middleware_builder(auth_routes, state.clone())
    .with_state_extension()
    .with_auth()
    .finalize_extension_based();

  // ============================================================================
  // Workspace routes (auth + workspace middleware)
  // ============================================================================
  let workspace_routes = create_stateless_router_with_routes(|router| {
    router
      .route(
        "/workspace/chats",
        get(handlers::chat::list_chats_handler).post(handlers::chat::create_chat_handler),
      )
      // User routes
      .route("/users", get(handlers::users::list_workspace_users_handler))
      .route("/users/profile", get(handlers::users::get_user_profile).put(handlers::users::update_user_profile))
      .route("/users/{user_id}/profile", get(handlers::users::get_user_profile_by_id).put(handlers::users::update_user_profile_by_id))
      // Presence status (alias for workspace users)
      .route("/presence/status", get(handlers::users::list_workspace_users_handler))
      // Password management
      .route("/users/change-password", post(handlers::users::change_password_handler))
  });

  let workspace_routes = create_extension_middleware_builder(workspace_routes, state.clone())
    .with_workspace()         // Apply workspace middleware first (executes third)
    .with_auth()             // Apply auth middleware second (executes second)
    .with_state_extension()   // Apply state extension last (executes first)
    .finalize_extension_based();

  // ============================================================================
  // Chat routes with parameters (auth + workspace + chat membership)
  // ============================================================================
  let chat_routes = create_stateless_router_with_routes(|router| {
    router
      // Chat basic operations
      .route("/chat/{id}", 
        get(handlers::chat::get_chat_handler)
          .patch(handlers::chat::update_chat_handler)
          .delete(handlers::chat::delete_chat_handler)
      )
      // Chat members operations
      .route("/chat/{id}/members", 
        get(handlers::chat_members::list_chat_members_handler)
        .post(handlers::chat_members::add_chat_members_handler)
      )
      // Chat messages operations
      .route("/chat/{id}/messages", 
        get(handlers::messages::list_messages_handler)
        .post(handlers::messages::send_message_handler)
      )
      // Chat search operations
      .route("/chat/{id}/messages/search", 
        get(handlers::search::simple_search_messages_in_chat)
        .post(handlers::search::search_messages_in_chat)
      )
      // Realtime operations (require chat membership)
      .route("/chat/{id}/typing/start", post(handlers::realtime::start_typing))
      .route("/chat/{id}/typing/stop", post(handlers::realtime::stop_typing))
      .route("/chat/{id}/typing/users", get(handlers::realtime::get_typing_users))
      .route("/chat/{id}/messages/{message_id}/read", post(handlers::realtime::mark_message_read))
      .route("/messages/{message_id}/receipts", get(handlers::realtime::get_message_receipts))
      // Enhanced message operations
      .route("/messages/{message_id}/mentions", get(handlers::messages::get_message_mentions_handler))
      .route("/messages/{message_id}/receipts/detailed", get(handlers::messages::get_detailed_message_receipts_handler))
      .route("/chat/{chat_id}/messages/{message_id}/read/enhanced", post(handlers::messages::mark_message_read_enhanced_handler))
      // Unread count for specific chat
      .route("/chat/{id}/unread", get(handlers::messages::get_unread_count_handler))
      // Admin operations
      .route("/admin/chat/{id}/reindex", 
        post(handlers::search::reindex_chat_messages)
      )
  });

  let chat_routes = create_extension_middleware_builder(chat_routes, state.clone())
    .with_chat_membership()   // Apply chat middleware first (executes fourth)
    .with_workspace()         // Apply workspace middleware second (executes third)
    .with_auth()             // Apply auth middleware third (executes second)
    .with_state_extension()   // Apply state extension last (executes first)
    .finalize_extension_based();

  // ============================================================================
  // Health Routes (use State for simplicity - infrastructure level)
  // ============================================================================
  let health_routes = Router::new()
    .route("/health", get(handlers::health::health_check))
    .route("/health/readiness", get(handlers::health::simple_health_check))
    .with_state(state.clone());

  // ============================================================================
  // Final Router Assembly - ALL stateless Router<()>
  // ============================================================================
  let api_routes = Router::new()
    .merge(public_routes)
    .merge(auth_routes)
    .merge(workspace_routes)
    .merge(chat_routes);

  // ============================================================================
  // Static Files Service - Use config storage path
  // ============================================================================
  let storage_path = &state.config.storage.path;
  let url_prefix = &state.config.storage.url_prefix;
  
  info!("🗂️ [STATIC_FILES] Configuring file service:");
  info!("🗂️ [STATIC_FILES] - Storage path: {}", storage_path);
  info!("🗂️ [STATIC_FILES] - URL prefix: {}", url_prefix);
  
  // Verify storage directory exists
  if !std::path::Path::new(storage_path).exists() {
    warn!("⚠️ [STATIC_FILES] Storage directory does not exist: {}", storage_path);
    warn!("⚠️ [STATIC_FILES] Attempting to create directory...");
    if let Err(e) = std::fs::create_dir_all(storage_path) {
      return Err(AppError::ChatFileError(format!(
        "Failed to create storage directory {}: {}", storage_path, e
      )));
    }
    info!("✅ [STATIC_FILES] Created storage directory: {}", storage_path);
  } else {
    info!("✅ [STATIC_FILES] Storage directory exists: {}", storage_path);
    
    // Log directory contents for debugging
    if let Ok(entries) = std::fs::read_dir(storage_path) {
      let count = entries.count();
      debug!("🗂️ [STATIC_FILES] Directory contains {} items", count);
    }
  }
  
  let files_service = ServeDir::new(storage_path)
    .append_index_html_on_directories(false);

  // Initialize symlinks for existing files
  crate::handlers::files::initialize_file_symlinks(storage_path).await
    .map_err(|e| AppError::ChatFileError(format!("Failed to initialize file symlinks: {}", e)))?;

  // Build final application - NO with_state() calls!
  let app = Router::new()
    .nest("/api", api_routes)
    .merge(health_routes)
    .nest_service("/files", files_service)
    .layer(axum::middleware::from_fn(route_debug_middleware));

  Ok(app)
}
