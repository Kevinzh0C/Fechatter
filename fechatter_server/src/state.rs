// ============================================================================
// State Internal Implementation - Complex trait implementations and initialization
// ============================================================================

use fechatter_core::{
  chat::ChatSidebar,
  error::CoreError,
  middlewares::{
    ActualAuthServiceProvider, TokenVerifier as CoreTokenVerifier, WithServiceProvider,
  },
  models::jwt::{TokenManager, UserClaims},
  models::{AuthUser, Workspace},
  AuthTokens, CreateUser, SigninUser, UserId,
};
use sqlx::PgPool;
use std::{sync::Arc, time::Instant};

use crate::config::{AppConfig, CorsConfig};
use crate::domains::chat::ChatMemberRepository;
use crate::error::{membership_status_to_app_error, AppError};
use crate::services::application::builders::ServiceProvider as ApplicationServiceProvider;
use crate::services::infrastructure::cache::{RedisCacheService, SyncCacheAdapter};
use crate::services::infrastructure::event::{
  AnalyticsConfig, LegacyEventPublisher, NatsAnalyticsPublisher, TransportFactory, EventTransport,
};
use axum::http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

// Import AppState from lib.rs
use crate::{AppState, AppStateInner};

// ============================================================================
// Complex Trait Implementations - Internal implementation details
// ============================================================================

impl CoreTokenVerifier for AppState {
  type Claims = UserClaims;
  type Error = CoreError;

  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error> {
    <TokenManager as fechatter_core::middlewares::TokenVerifier>::verify_token(
      self.inner.application_services.token_manager().as_ref(),
      token,
    )
  }
}

// Implement WithTokenManager trait - required by core middlewares
impl fechatter_core::middlewares::WithTokenManager for AppState {
  type TokenManagerType = TokenManager;

  fn token_manager(&self) -> &Self::TokenManagerType {
    self.inner.token_manager.as_ref()
  }
}

// Production auth service wrapper to satisfy trait requirements
#[derive(Clone)]
pub struct ProductionAuthServiceWrapper {
  inner: Arc<crate::services::application::workers::auth::HighAvailabilityAuthService>,
}

impl ProductionAuthServiceWrapper {
  pub fn new(app_state: &AppState) -> Self {
    let auth_service =
      crate::services::application::workers::auth::AuthUserService::from_app_state(app_state);
    let ha_auth_service =
      crate::services::application::workers::auth::HighAvailabilityAuthService::new(Arc::new(
        auth_service,
      ));
    Self {
      inner: Arc::new(ha_auth_service),
    }
  }
}

// Implement required traits for ProductionAuthServiceWrapper
#[async_trait::async_trait]
impl fechatter_core::models::jwt::RefreshTokenService for ProductionAuthServiceWrapper {
  async fn refresh_token(
    &self,
    refresh_token: &str,
    auth_context: Option<fechatter_core::contracts::AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    self.inner.refresh_token(refresh_token, auth_context).await
  }
}

#[async_trait::async_trait]
impl fechatter_core::models::jwt::SignupService for ProductionAuthServiceWrapper {
  async fn signup(
    &self,
    payload: &CreateUser,
    auth_context: Option<fechatter_core::contracts::AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    self.inner.signup(payload, auth_context).await
  }
}

#[async_trait::async_trait]
impl fechatter_core::models::jwt::SigninService for ProductionAuthServiceWrapper {
  async fn signin(
    &self,
    payload: &SigninUser,
    auth_context: Option<fechatter_core::contracts::AuthContext>,
  ) -> Result<Option<AuthTokens>, CoreError> {
    self.inner.signin(payload, auth_context).await
  }
}

#[async_trait::async_trait]
impl fechatter_core::models::jwt::LogoutService for ProductionAuthServiceWrapper {
  async fn logout(&self, refresh_token: &str) -> Result<(), CoreError> {
    self.inner.logout(refresh_token).await
  }

  async fn logout_all(&self, user_id: UserId) -> Result<(), CoreError> {
    self.inner.logout_all(user_id).await
  }
}

impl fechatter_core::models::jwt::AuthServiceTrait for ProductionAuthServiceWrapper {}

// Implement ActualAuthServiceProvider for ProductionAuthServiceWrapper
impl ActualAuthServiceProvider for ProductionAuthServiceWrapper {
  type AuthService = Self;

  fn create_service(&self) -> Self::AuthService {
    self.clone()
  }
}

// Implement WithServiceProvider trait for AppState
impl fechatter_core::middlewares::WithServiceProvider for AppState {
  type ServiceProviderType = Self; // AppState itself is the service provider

  fn service_provider(&self) -> &Self::ServiceProviderType {
    self
  }
}

// Implement ActualAuthServiceProvider for AppState with caching
impl fechatter_core::middlewares::ActualAuthServiceProvider for AppState {
  type AuthService = ProductionAuthServiceWrapper;

  fn create_service(&self) -> Self::AuthService {
    // Try to get cached service first
    {
      let cache = self.inner.cached_auth_service.read().unwrap();
      if let Some(cached) = cache.as_ref() {
        return ProductionAuthServiceWrapper {
          inner: cached.inner.clone(),
        };
      }
    }

    // Create new service and cache it
    let new_service = ProductionAuthServiceWrapper::new(self);
    {
      let mut cache = self.inner.cached_auth_service.write().unwrap();
      *cache = Some(Arc::new(new_service.clone()));
    }
    new_service
  }
}

// ============================================================================
// Cache Implementation for Legacy Compatibility - Internal details
// ============================================================================

// Define cache trait for internal use
trait WithCache<K, V> {
  fn get_from_cache(&self, key: &K) -> Option<V>;
  fn insert_into_cache(&self, key: K, value: V, ttl_seconds: u64);
  fn remove_from_cache(&self, key: &K);
}

impl WithCache<i64, (Arc<Vec<ChatSidebar>>, Instant)> for AppState {
  fn get_from_cache(&self, key: &i64) -> Option<(Arc<Vec<ChatSidebar>>, Instant)> {
    self.inner.sync_cache_adapter.get_chat_list_sync(*key)
  }

  fn insert_into_cache(&self, key: i64, value: (Arc<Vec<ChatSidebar>>, Instant), ttl_seconds: u64) {
    let (chats_arc, _timestamp) = value;
    let chats: Vec<ChatSidebar> = (*chats_arc).clone();
    self
      .inner
      .sync_cache_adapter
      .set_chat_list_sync(key, chats, ttl_seconds);
  }

  fn remove_from_cache(&self, key: &i64) {
    self.inner.sync_cache_adapter.remove_chat_list_sync(*key);
  }
}

// ============================================================================
// State Module - AppState Implementation & Business Methods
// ============================================================================

// Remove duplicate imports - already declared above
use tracing::{debug, error, info, warn};

// ============================================================================
// AppState Business Methods - Core Implementation
// ============================================================================

impl AppState {
  /// Check if user is a member of the specified chat - Enhanced production-grade version
  /// Used by chat middleware for precise access control with detailed error classification
  pub async fn ensure_user_is_chat_member(
    &self,
    chat_id: i64,
    user_id: i64,
  ) -> Result<bool, AppError> {
    info!(
      "🔍 [APPSTATE] ========== Enhanced membership check: user {} in chat {} ==========",
      user_id, chat_id
    );

    let chat_member_repo = ChatMemberRepository::new(self.pool());

    match chat_member_repo
      .validate_chat_and_membership(chat_id, user_id)
      .await
    {
      Ok(status) => {
        use crate::domains::chat::ChatMembershipStatus;

        match status {
          ChatMembershipStatus::ActiveMember { role, .. } => {
            info!(
              "🔍 [APPSTATE] ✅ SUCCESS: User {} is active member of chat {} with role '{}'",
              user_id, chat_id, role
            );
            Ok(true)
          }
          other_status => {
            let error_msg = other_status.error_message();
            warn!("🔍 [APPSTATE] 🚫 FAILED: Chat access denied: {}", error_msg);

            // Convert membership status to appropriate AppError
            let app_error = membership_status_to_app_error(other_status);
            error!("🔍 [APPSTATE] ❌ Generated AppError: {:?}", app_error);

            Err(app_error)
          }
        }
      }
      Err(e) => {
        error!("🔍 [APPSTATE] ❌ Database error during membership validation for user {} in chat {}: {:?}", user_id, chat_id, e);
        Err(AppError::from(e))
      }
    }
  }

  /// Find workspace by ID - Using WorkspaceApplicationService
  /// Used by builder_old workspace middleware for workspace context
  pub async fn find_by_id_with_pool(
    &self,
    workspace_id: i64,
  ) -> Result<Option<Workspace>, AppError> {
    let workspace_service = self
      .inner
      .application_services
      .workspace_application_service();

    match workspace_service
      .get_workspace_details(fechatter_core::WorkspaceId(workspace_id))
      .await
    {
      Ok(workspace_view) => {
        // Convert WorkspaceView back to Workspace model
        let workspace = Workspace {
          id: fechatter_core::WorkspaceId(workspace_view.id),
          name: workspace_view.name,
          owner_id: fechatter_core::UserId(workspace_view.owner_id),
          created_at: workspace_view.created_at,
        };

        Ok(Some(workspace))
      }
      Err(AppError::NotFound(_)) => Ok(None),
      Err(e) => {
        error!("Failed to find workspace {}: {:?}", workspace_id, e);
        Err(e)
      }
    }
  }

  /// Create workspace with specified name and owner - Using WorkspaceApplicationService
  /// Used by builder_old workspace middleware when workspace is missing
  pub async fn create_workspace_with_pool(
    &self,
    name: &str,
    owner_id: i64,
  ) -> Result<Workspace, AppError> {
    let workspace_service = self
      .inner
      .application_services
      .workspace_application_service();

    match workspace_service
      .create_workspace(name, fechatter_core::UserId(owner_id))
      .await
    {
      Ok(workspace_view) => {
        info!(
          "🏢 [APPSTATE] Successfully created workspace '{}' for user {}",
          workspace_view.name, owner_id
        );

        // Convert WorkspaceView back to Workspace model
        let workspace = Workspace {
          id: fechatter_core::WorkspaceId(workspace_view.id),
          name: workspace_view.name,
          owner_id: fechatter_core::UserId(workspace_view.owner_id),
          created_at: workspace_view.created_at,
        };

        Ok(workspace)
      }
      Err(e) => {
        error!(
          "Failed to create workspace '{}' for user {}: {:?}",
          name, owner_id, e
        );
        Err(e)
      }
    }
  }
}

// ============================================================================
// State Creation Functions - Core Implementation
// ============================================================================

/// Create AppState for tests
#[cfg(test)]
pub fn create_test_state() -> AppState {
  // Implementation for test state creation
  todo!("Implement test state creation")
}

/// Create AppState - Main initialization method
pub async fn create_production_state(config: AppConfig) -> Result<AppState, AppError> {
  // Use the database URL from configuration instead of hardcoded value
  let pool = create_pool_impl(&config.server.db_url).await?;

  // Create refresh token repository for production use
  let refresh_token_repo = Arc::new(
    crate::domains::auth::token_repository::CoreRefreshTokenRepositoryAdapter::new(Arc::new(
      pool.clone(),
    )),
  );

  // Create token manager using auth config and production repository
  let token_manager =
    fechatter_core::models::jwt::TokenManager::from_config(&config.auth, refresh_token_repo)
      .map_err(|e| AppError::Internal(format!("Failed to create token manager: {}", e)))?;

  // ============================================================================
  // 🔧 Enhanced Event Publisher for notify_server Integration
  // ============================================================================

  let enhanced_event_publisher = if config.features.messaging.enabled {
    info!("🚀 Initializing enhanced event publisher for notify_server integration...");
    match crate::services::infrastructure::event::create_enhanced_publisher_for_notify_server(
      &config.features.messaging.nats_url
    ).await {
      Ok(publisher) => {
        info!("✅ Enhanced event publisher created successfully for notify_server SSE");
        Some(Arc::new(publisher))
      }
      Err(e) => {
        error!("❌ Failed to create enhanced event publisher: {}. notify_server SSE limited.", e);
        warn!("⚠️ Application will run with limited real-time notification functionality");
        None
      }
    }
  } else {
    info!("📄 Enhanced event publisher disabled - messaging not enabled");
    None
  };

  // ============================================================================
  // 🔧 NATS Event Publisher Initialization
  // ============================================================================

  let event_publisher = if config.features.messaging.enabled {
    info!("🚀 Initializing NATS event publisher...");
    match TransportFactory::create_from_config(&config).await {
      Ok(transport) => {
        info!(
          "✅ NATS transport created successfully: {}",
          config.features.messaging.nats_url
        );
        let publisher = Arc::new(LegacyEventPublisher::with_dyn_transport(transport));
        Some(publisher)
      }
      Err(e) => {
        error!("❌ Failed to create NATS transport: {}. Using fallback.", e);
        warn!("⚠️ Application will run with limited messaging functionality");
        None
      }
    }
  } else {
    info!("📄 Messaging disabled in configuration");
    None
  };

  // ============================================================================
  // 🔧 Analytics Publisher Initialization
  // ============================================================================

  let analytics_publisher = if config.features.messaging.enabled {
    if let Some(ref event_pub) = event_publisher {
      // Try to get NATS transport from event publisher (not client)
      let transport = event_pub.transport().clone();
      if let Some(nats_transport) = transport
        .as_any()
        .downcast_ref::<crate::services::infrastructure::event::NatsTransport>(
      ) {
        info!("🔧 Creating NATS analytics publisher...");
        let analytics_config = AnalyticsConfig {
          enabled: true,
          subject_prefix: "fechatter.analytics".to_string(),
          batch_size: 100,
          flush_interval_ms: 5000,
        };

        // Clone the transport and wrap in Arc for the analytics publisher
        let transport_arc = Arc::new(nats_transport.clone());
        let publisher = NatsAnalyticsPublisher::new(transport_arc, analytics_config);
        info!("✅ NATS analytics publisher initialized successfully");
        Some(Arc::new(publisher))
      } else {
        warn!("⚠️ No NATS transport available, analytics disabled");
        None
      }
    } else {
      info!("📄 Analytics disabled - no event publisher available");
      None
    }
  } else {
    info!("📄 Analytics disabled - messaging not enabled");
    None
  };

  // ============================================================================
  // 🔧 Application Services with Search Support
  // ============================================================================

  // Create application services with proper search configuration
  let mut application_services_builder =
    ApplicationServiceProvider::builder(Arc::new(pool.clone()), Arc::new(token_manager.clone()));

  // Add NATS URL if messaging is enabled
  if config.features.messaging.enabled {
    application_services_builder =
      application_services_builder.with_nats_url(config.features.messaging.nats_url.clone());
  }

  // Add search service if enabled
  if config.features.search.enabled {
    info!(
      "🔍 Initializing search service with provider: {}",
      config.features.search.provider
    );
    info!(
      "🔍 Search config details: url={}, batch_size={}, async_indexing={}",
      config.features.search.meilisearch_url,
      config.features.search.batch_size,
      config.features.search.async_indexing
    );

    match crate::services::SearchService::new_from_config(&config.features.search).await {
      Ok(search_service) => {
        info!(
          "✅ Search service initialized successfully: {}",
          config.features.search.meilisearch_url
        );
        application_services_builder =
          application_services_builder.with_search_service(Arc::new(search_service));
      }
      Err(e) => {
        error!(
          "⚠️ Failed to initialize search service: {}. Search functionality disabled. Config: {:?}",
          e, config.features.search
        );
      }
    }
  } else {
    info!("📄 Search disabled in configuration");
  }

  let application_services = application_services_builder.build();

  // Initialize Redis cache service if enabled
  let cache_service = if config.features.cache.enabled {
    match RedisCacheService::new(&config.features.cache.redis_url, "fechatter").await {
      Ok(redis_service) => {
        info!("✅ Redis cache service initialized successfully");
        Some(Arc::new(redis_service))
      }
      Err(e) => {
        warn!(
          "⚠️ Failed to initialize Redis cache service: {}. Continuing without cache.",
          e
        );
        None
      }
    }
  } else {
    info!("📄 Cache disabled in configuration");
    None
  };

  let sync_cache_adapter =
    crate::services::infrastructure::cache::SyncCacheAdapter::new(cache_service.clone());
  let cached_auth_service = std::sync::RwLock::new(None);

  let inner = AppStateInner {
    config,
    application_services,
    token_manager: Arc::new(token_manager),
    event_publisher: event_publisher.clone(),
    unified_event_publisher: event_publisher,
    enhanced_event_publisher,
    cache_service,
    sync_cache_adapter,
    analytics_publisher,
    cached_auth_service,
  };

  let app_state = AppState {
    inner: Arc::new(inner),
  };

  // ============================================================================
  // 🔧 System Status Summary
  // ============================================================================

  info!("🎯 System initialization complete:");
  info!("  📄 Database: ✅ Connected");
  info!(
    "  📄 Cache: {}",
    if app_state.cache_service().is_some() {
      "✅ Enabled"
    } else {
      "❌ Disabled"
    }
  );
  info!(
    "  📄 NATS: {}",
    if app_state.nats_client().is_some() {
      "✅ Connected"
    } else {
      "❌ Not connected"
    }
  );
  info!(
    "  📄 Search: {}",
    if app_state.is_search_enabled() {
      "✅ Enabled"
    } else {
      "❌ Disabled"
    }
  );
  info!(
    "  📄 Analytics: {}",
    if app_state.analytics_publisher().is_some() {
      "✅ Enabled"
    } else {
      "❌ Disabled"
    }
  );
  info!(
    "  📄 notify_server SSE: {}",
    if app_state.enhanced_event_publisher().is_some() {
      "✅ Enabled"
    } else {
      "❌ Disabled"
    }
  );

  Ok(app_state)
}

/// Helper function to extract NATS client from event publisher
fn get_nats_client_from_publisher(
  publisher: &Arc<crate::services::infrastructure::event::DynEventPublisher>,
) -> Option<async_nats::Client> {
  use crate::services::infrastructure::event::NatsTransport;

  let transport = publisher.transport();
  transport
    .as_any()
    .downcast_ref::<NatsTransport>()
    .map(|nats_transport| nats_transport.client().clone())
}

/// Create database pool - Implementation (separate from the existing one)
async fn create_pool_impl(db_url: &str) -> Result<PgPool, sqlx::Error> {
  let pool = sqlx::postgres::PgPoolOptions::new()
    .max_connections(10)
    .connect(db_url)
    .await?;

  // Skip migrations for now as the directory doesn't exist
  // sqlx::migrate!("./migrations").run(&pool).await?;

  Ok(pool)
}

// ============================================================================
// CORS Configuration - Implementation
// ============================================================================

/// CORS configuration function - standard implementation in English
pub fn build_cors(config: &CorsConfig) -> CorsLayer {
  use tower_http::cors::AllowOrigin;

  let mut cors_layer = CorsLayer::new();

  // 1) Specify allowed origins - dynamic matching
  let allowed_origins = config.allow_origins.clone();
  cors_layer = cors_layer.allow_origin(AllowOrigin::predicate(
    move |origin: &HeaderValue, _request_parts| {
      if let Ok(origin_str) = origin.to_str() {
        let is_allowed = allowed_origins.contains(&origin_str.to_string());
        debug!("CORS origin check: {} -> {}", origin_str, is_allowed);
        is_allowed
      } else {
        false
      }
    },
  ));

  // 2) Specify allowed HTTP methods for complex requests
  let methods: Vec<Method> = config
    .allow_methods
    .iter()
    .filter_map(|method| method.parse().ok())
    .collect();
  if !methods.is_empty() {
    cors_layer = cors_layer.allow_methods(methods);
  }

  // 3) Specify allowed headers if the frontend needs to send custom headers
  let headers: Vec<axum::http::HeaderName> = config
    .allow_headers
    .iter()
    .filter_map(|header| header.parse().ok())
    .collect();
  if !headers.is_empty() {
    cors_layer = cors_layer.allow_headers(headers);
  }

  // 4) Allow credentials (cookies/JWT) if needed
  if config.allow_credentials {
    cors_layer = cors_layer.allow_credentials(true);
  }

  // 5) Set preflight cache duration (in seconds)
  cors_layer = cors_layer.max_age(std::time::Duration::from_secs(config.max_age));

  cors_layer
}

// Keep the original function for compatibility
pub async fn create_pool_impl_pub(db_url: &str) -> Result<PgPool, sqlx::Error> {
  create_pool_impl(db_url).await
}
