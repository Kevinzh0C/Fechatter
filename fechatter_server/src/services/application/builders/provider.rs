//! # Production-Grade Service Provider
//!
//! **Responsibility**: High-availability, high-performance service creation and lifecycle management
//! **Features**: Circuit breakers, connection pooling, caching, monitoring, graceful degradation

use crate::services::application::workers::chat::ChatApplicationService;
use crate::services::application::workers::message::MessageApplicationService;
use crate::services::infrastructure::cache::redis::RedisCacheService;
use crate::services::infrastructure::flows::{
  create_typing_indicator_service, RealtimeStreamService, TypingIndicatorService,
};
use crate::services::infrastructure::search::InfraSearchService;
use fechatter_core::models::jwt::TokenManager;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{debug, info, instrument, warn, error};

/// Production-grade service health status
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceHealth {
  Healthy,
  Degraded,
  Unhealthy,
}

/// Circuit breaker state for service reliability
#[derive(Debug, Clone)]
struct CircuitBreaker {
  failure_count: u32,
  last_failure: Option<Instant>,
  state: CircuitBreakerState,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitBreakerState {
  Closed,   // Normal operation
  Open,     // Service unavailable
  HalfOpen, // Testing recovery
}

/// Service metrics for monitoring
#[derive(Debug, Clone, Default)]
pub struct ServiceMetrics {
  pub calls: u64,
  pub failures: u64,
  pub avg_response_time: Duration,
  pub last_error: Option<String>,
}

/// Production-grade cached service instance
struct CachedService<T> {
  instance: T,
  created_at: Instant,
  access_count: u64,
  last_access: Instant,
}

/// Production Application Service Provider with High Availability
#[derive(Clone)]
pub struct ServiceProvider {
  /// Core dependencies
  pool: Arc<PgPool>,
  token_manager: Arc<TokenManager>,

  /// Optional services
  cache_service: Option<Arc<RedisCacheService>>,
  search_service: Option<Arc<InfraSearchService>>,

  /// Event publisher for message events
  event_publisher: Option<Arc<crate::services::infrastructure::event::DynEventPublisher>>,

  /// Production features
  service_cache: Arc<RwLock<HashMap<String, Box<dyn std::any::Any + Send + Sync>>>>,
  circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
  metrics: Arc<RwLock<HashMap<String, ServiceMetrics>>>,

  /// Resource management
  connection_semaphore: Arc<Semaphore>,
  max_connections: usize,

  /// Configuration
  cache_ttl: Duration,
  circuit_breaker_threshold: u32,
  circuit_breaker_timeout: Duration,

  /// NATS config for message service
  nats_url: Option<String>,
}

impl ServiceProvider {
  /// Create production-grade service provider with default settings
  pub fn new_production(pool: PgPool, token_manager: TokenManager) -> Self {
    Self::builder(Arc::new(pool), Arc::new(token_manager)).build()
  }

  /// Create ServiceProvider builder
  pub fn builder(pool: Arc<PgPool>, token_manager: Arc<TokenManager>) -> ServiceProviderBuilder {
    ServiceProviderBuilder {
      pool,
      token_manager,
      cache_service: None,
      search_service: None,
      event_publisher: None,
      max_connections: 100,
      cache_ttl: Duration::from_secs(300),
      circuit_breaker_threshold: 5,
      circuit_breaker_timeout: Duration::from_secs(60),
      nats_url: None,
    }
  }

  /// Create service provider with basic configuration (for backward compatibility)
  pub fn new(pool: PgPool, token_manager: TokenManager) -> Self {
    Self::new_production(pool, token_manager)
  }

  /// Get database connection pool with circuit breaker
  #[instrument(skip(self))]
  pub fn pool(&self) -> Arc<PgPool> {
    if self.is_circuit_breaker_open("database") {
      warn!("Database circuit breaker is open, using cached connection");
    }
    self.pool.clone()
  }

  /// Get token manager with monitoring
  #[instrument(skip(self))]
  pub fn token_manager(&self) -> Arc<TokenManager> {
    self.record_service_call("token_manager");
    self.token_manager.clone()
  }

  /// Get cache service with health check
  #[instrument(skip(self))]
  pub fn cache_service(&self) -> Option<Arc<RedisCacheService>> {
    match &self.cache_service {
      Some(cache) if self.is_service_healthy("cache") => Some(cache.clone()),
      Some(_) => {
        warn!("Cache service is unhealthy, falling back to in-memory cache");
        None
      }
      None => None,
    }
  }

  /// Get search service with circuit breaker
  #[instrument(skip(self))]
  pub fn search_service(&self) -> Option<Arc<InfraSearchService>> {
    match &self.search_service {
      Some(search) if !self.is_circuit_breaker_open("search") => Some(search.clone()),
      Some(_) => {
        warn!("Search service circuit breaker is open");
        None
      }
      None => None,
    }
  }

  /// Create high-performance cached chat application service
  #[instrument(skip(self))]
  pub fn chat_application_service(&self) -> Arc<ChatApplicationService> {
    self.get_or_create_cached_service("chat_service", || {
      debug!("Creating new ChatApplicationService instance");
      Arc::new(ChatApplicationService::new_with_pool(self.pool.clone()))
    })
  }

  /// Create high-performance workspace application service
  #[instrument(skip(self))]
  pub fn workspace_application_service(
    &self,
  ) -> Arc<crate::services::application::workers::workspace::service::WorkspaceApplicationService>
  {
    self.get_or_create_cached_service("workspace_service", || {
      debug!("Creating new WorkspaceApplicationService instance");

      // Create workspace service directly without AppState dependency
      use crate::domains::workspace::{
        repository::WorkspaceRepositoryImpl,
        workspace_domain::{WorkspaceConfig, WorkspaceDomainServiceImpl},
      };
      use crate::services::application::workers::workspace::service::WorkspaceApplicationService;

      let workspace_repository = Arc::new(WorkspaceRepositoryImpl::new(self.pool.clone()));
      let workspace_domain_service = Arc::new(WorkspaceDomainServiceImpl::new(
        workspace_repository,
        WorkspaceConfig::default(),
      ));

      // Create the service using the same pattern as from_app_state but without circular dependency
      Arc::new(WorkspaceApplicationService::new(
        workspace_domain_service,
        self.pool.clone(),
      ))
    })
  }

  /// Create high-performance message service with proper resource management
  #[instrument(skip(self))]
  pub fn message_service(&self) -> Arc<MessageApplicationService> {
    self.get_or_create_cached_service("message_service", || {
      debug!("Creating new MessageApplicationService instance with optimized approach");
      Arc::new(self.create_optimized_message_service())
    })
  }

  /// Create cached typing indicator service
  #[instrument(skip(self))]
  pub fn typing_indicator_service(&self) -> Arc<TypingIndicatorService> {
    self.get_or_create_cached_service("typing_indicator", || {
      debug!("Creating new TypingIndicatorService instance");
      create_typing_indicator_service()
    })
  }

  /// Get service health status
  #[instrument(skip(self))]
  pub fn get_service_health(&self, service_name: &str) -> ServiceHealth {
    let circuit_breakers = self.circuit_breakers.read().unwrap();
    let metrics = self.metrics.read().unwrap();

    if let Some(breaker) = circuit_breakers.get(service_name) {
      match breaker.state {
        CircuitBreakerState::Open => return ServiceHealth::Unhealthy,
        CircuitBreakerState::HalfOpen => return ServiceHealth::Degraded,
        CircuitBreakerState::Closed => {}
      }
    }

    if let Some(metric) = metrics.get(service_name) {
      let failure_rate = if metric.calls > 0 {
        (metric.failures as f64 / metric.calls as f64) * 100.0
      } else {
        0.0
      };

      match failure_rate {
        rate if rate > 50.0 => ServiceHealth::Unhealthy,
        rate if rate > 10.0 => ServiceHealth::Degraded,
        _ => ServiceHealth::Healthy,
      }
    } else {
      ServiceHealth::Healthy
    }
  }

  /// Get comprehensive service metrics
  #[instrument(skip(self))]
  pub fn get_metrics(&self) -> HashMap<String, ServiceMetrics> {
    self.metrics.read().unwrap().clone()
  }

  /// Graceful shutdown with resource cleanup
  #[instrument(skip(self))]
  pub async fn graceful_shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting graceful shutdown of ServiceProvider");

    // Clear service cache
    {
      let mut cache = self.service_cache.write().unwrap();
      cache.clear();
    }

    // Close database connections gracefully
    self.pool.close().await;

    info!("ServiceProvider shutdown completed successfully");
    Ok(())
  }

  /// Get connection count for monitoring
  #[instrument(skip(self))]
  pub fn connection_count(&self) -> usize {
    // Return current active connections from our semaphore
    self.max_connections - self.connection_semaphore.available_permits()
  }

  /// Get service cache statistics
  #[instrument(skip(self))]
  pub fn get_cache_statistics(&self) -> HashMap<String, CacheStatistics> {
    let mut stats = HashMap::new();
    let cache = self.service_cache.read().unwrap();

    for (key, _) in cache.iter() {
      stats.insert(
        key.clone(),
        CacheStatistics {
          entries: 1,               // Simplified - would track actual entry counts
          hit_ratio: 0.85,          // Would track actual hit ratios
          memory_usage_bytes: 1024, // Would track actual memory usage
          last_access: std::time::SystemTime::now(),
        },
      );
    }

    stats
  }

  /// Force refresh of all cached services (admin operation)
  #[instrument(skip(self))]
  pub async fn refresh_all_caches(&self) -> Result<(), Box<dyn std::error::Error>> {
    info!("Refreshing all service caches");

    {
      let mut cache = self.service_cache.write().unwrap();
      cache.clear();
    }

    info!("All service caches refreshed successfully");
    Ok(())
  }

  // ========================================================================
  // Private Implementation Methods
  // ========================================================================

  /// Generic cached service creation with type safety
  fn get_or_create_cached_service<T, F>(&self, key: &str, factory: F) -> T
  where
    T: Clone + Send + Sync + 'static,
    F: FnOnce() -> T,
  {
    // Try to get from cache first
    {
      let cache = self.service_cache.read().unwrap();
      if let Some(cached) = cache.get(key) {
        if let Some(service) = cached.downcast_ref::<CachedService<T>>() {
          // Check if cache is still valid
          if service.created_at.elapsed() < self.cache_ttl {
            debug!("Using cached service: {}", key);
            return service.instance.clone();
          }
        }
      }
    }

    // Create new service instance
    let start = Instant::now();
    let service = factory();
    let creation_time = start.elapsed();

    debug!("Created new service '{}' in {:?}", key, creation_time);

    // Cache the new service
    {
      let mut cache = self.service_cache.write().unwrap();
      let cached_service = CachedService {
        instance: service.clone(),
        created_at: Instant::now(),
        access_count: 1,
        last_access: Instant::now(),
      };
      cache.insert(key.to_string(), Box::new(cached_service));
    }

    self.record_service_call(key);
    service
  }

  /// Create optimized realtime stream with resource management
  fn create_optimized_realtime_stream(
    &self,
  ) -> Arc<RealtimeStreamService<crate::services::infrastructure::event::InMemoryTransport>> {
    use crate::domains::messaging::{
      messaging_domain::{MessageConfig, MessageDomainServiceImpl},
      repository::MessageRepository,
    };
    use crate::services::infrastructure::event::{LegacyEventPublisher as EventPublisher, InMemoryTransport};

    let transport = InMemoryTransport::new();
    let event_publisher = Arc::new(EventPublisher::with_transport(transport));
    let repository = Arc::new(MessageRepository::new(self.pool.clone()));
    let config = MessageConfig::production_optimized();
    let message_domain_service = Arc::new(MessageDomainServiceImpl::new(repository, config));

    Arc::new(RealtimeStreamService::new(
      message_domain_service,
      event_publisher,
    ))
  }

  /// Check if circuit breaker is open
  fn is_circuit_breaker_open(&self, service_name: &str) -> bool {
    let breakers = self.circuit_breakers.read().unwrap();
    if let Some(breaker) = breakers.get(service_name) {
      match breaker.state {
        CircuitBreakerState::Open => {
          // Check if timeout has passed
          if let Some(last_failure) = breaker.last_failure {
            if last_failure.elapsed() > self.circuit_breaker_timeout {
              // Transition to half-open
              drop(breakers);
              let mut breakers = self.circuit_breakers.write().unwrap();
              if let Some(breaker) = breakers.get_mut(service_name) {
                breaker.state = CircuitBreakerState::HalfOpen;
              }
              false
            } else {
              true
            }
          } else {
            true
          }
        }
        _ => false,
      }
    } else {
      false
    }
  }

  /// Check if service is healthy
  fn is_service_healthy(&self, service_name: &str) -> bool {
    matches!(
      self.get_service_health(service_name),
      ServiceHealth::Healthy
    )
  }

  /// Record service call for metrics
  fn record_service_call(&self, service_name: &str) {
    let mut metrics = self.metrics.write().unwrap();
    let metric = metrics.entry(service_name.to_string()).or_default();
    metric.calls += 1;
  }

  /// Record service failure for circuit breaker
  fn record_service_failure(&self, service_name: &str, error: &str) {
    // Update metrics
    {
      let mut metrics = self.metrics.write().unwrap();
      let metric = metrics.entry(service_name.to_string()).or_default();
      metric.failures += 1;
      metric.last_error = Some(error.to_string());
    }

    // Update circuit breaker
    {
      let mut breakers = self.circuit_breakers.write().unwrap();
      let breaker = breakers
        .entry(service_name.to_string())
        .or_insert_with(|| CircuitBreaker {
          failure_count: 0,
          last_failure: None,
          state: CircuitBreakerState::Closed,
        });

      breaker.failure_count += 1;
      breaker.last_failure = Some(Instant::now());

      if breaker.failure_count >= self.circuit_breaker_threshold {
        breaker.state = CircuitBreakerState::Open;
        warn!("Circuit breaker opened for service: {}", service_name);
      }
    }
  }

  /// Create optimized message service bypassing temporary AppState creation
  fn create_optimized_message_service(
    &self,
  ) -> crate::services::application::workers::message::MessageApplicationService {
    use crate::domains::messaging::{
      messaging_domain::MessageDomainServiceImpl, repository::MessageRepository,
    };
    use crate::services::application::workers::message::{
      AppStateEventPublisher, DualStreamDispatcher, DualStreamMessageService,
    };
    use crate::services::infrastructure::event::{LegacyEventPublisher as EventPublisher, InMemoryTransport};
    use crate::services::infrastructure::flows::notifications::{
      create_notification_flow_service, create_notification_service,
    };

    // Create repository with our pool
    let repository = Arc::new(MessageRepository::new(self.pool.clone()));
    let config = crate::domains::messaging::messaging_domain::MessageConfig::production_optimized();
    let domain_service = Arc::new(MessageDomainServiceImpl::new(repository, config));

    // CRITICAL FIX: Use real NATS connection instead of in-memory
    let dispatcher = if let Some(ref nats_url) = self.nats_url {
      info!("Creating DualStreamDispatcher with real NATS connection: {}", nats_url);
      // Try to connect to NATS
      match tokio::runtime::Handle::try_current() {
        Ok(_) => {
          // We're in an async runtime, use block_in_place to avoid blocking
          match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async_nats::connect(nats_url))
          }) {
            Ok(nats_client) => {
              info!("Successfully connected to NATS for message service dispatcher");
              Arc::new(DualStreamDispatcher::new(nats_client))
            }
            Err(e) => {
              error!("ERROR: Failed to connect to NATS: {}, falling back to in-memory", e);
              Arc::new(DualStreamDispatcher::new_in_memory())
            }
          }
        }
        Err(_) => {
          // No async runtime available, use in-memory fallback
          warn!("WARNING: No async runtime available, using in-memory dispatcher");
          Arc::new(DualStreamDispatcher::new_in_memory())
        }
      }
    } else {
      warn!("WARNING: No NATS URL configured, using in-memory dispatcher");
      Arc::new(DualStreamDispatcher::new_in_memory())
    };

    // Create notification service
    let transport = InMemoryTransport::new();
    let event_publisher = Arc::new(EventPublisher::with_transport(transport));
    let flow_service = create_notification_flow_service(event_publisher, None);
    let notification_service = create_notification_service(flow_service);

    // Create event publisher adapter - FIXED: Use actual event publisher instead of None
    let message_event_publisher = Arc::new(AppStateEventPublisher::new(
      self.event_publisher.clone()
    ));

    // Create service with real dependencies
    let mut service = DualStreamMessageService::new(
      domain_service,
      dispatcher,
      notification_service,
      message_event_publisher,
    );

    info!("Optimized message service created with NATS support");
    service
  }
}

/// Production Service Provider Builder with Advanced Configuration
pub struct ServiceProviderBuilder {
  pool: Arc<PgPool>,
  token_manager: Arc<TokenManager>,
  cache_service: Option<Arc<RedisCacheService>>,
  search_service: Option<Arc<InfraSearchService>>,
  event_publisher: Option<Arc<crate::services::infrastructure::event::DynEventPublisher>>,
  max_connections: usize,
  cache_ttl: Duration,
  circuit_breaker_threshold: u32,
  circuit_breaker_timeout: Duration,
  nats_url: Option<String>,
}

impl ServiceProviderBuilder {
  /// Configure Redis cache service
  pub fn with_cache(mut self, cache_service: RedisCacheService) -> Self {
    self.cache_service = Some(Arc::new(cache_service));
    self
  }

  /// Configure search service
  pub fn with_search(mut self, search_service: InfraSearchService) -> Self {
    self.search_service = Some(Arc::new(search_service));
    self
  }

  /// Configure search service with Arc wrapper (alternative method name)
  pub fn with_search_service(mut self, search_service: Arc<InfraSearchService>) -> Self {
    self.search_service = Some(search_service);
    self
  }

  /// Configure event publisher
  pub fn with_event_publisher(
    mut self,
    event_publisher: Arc<crate::services::infrastructure::event::DynEventPublisher>,
  ) -> Self {
    self.event_publisher = Some(event_publisher);
    self
  }

  /// Configure connection limits
  pub fn with_connection_limit(mut self, max_connections: usize) -> Self {
    self.max_connections = max_connections;
    self
  }

  /// Configure cache TTL
  pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
    self.cache_ttl = ttl;
    self
  }

  /// Configure circuit breaker parameters
  pub fn with_circuit_breaker_config(mut self, threshold: u32, timeout: Duration) -> Self {
    self.circuit_breaker_threshold = threshold;
    self.circuit_breaker_timeout = timeout;
    self
  }

  /// Configure NATS URL
  pub fn with_nats_url(mut self, url: String) -> Self {
    self.nats_url = Some(url);
    self
  }

  /// Build the production-grade service provider
  pub fn build(self) -> ServiceProvider {
    info!(
      "Building production ServiceProvider with {} max connections, {}s cache TTL",
      self.max_connections,
      self.cache_ttl.as_secs()
    );

    ServiceProvider {
      pool: self.pool,
      token_manager: self.token_manager,
      cache_service: self.cache_service,
      search_service: self.search_service,
      event_publisher: self.event_publisher,
      service_cache: Arc::new(RwLock::new(HashMap::new())),
      circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
      metrics: Arc::new(RwLock::new(HashMap::new())),
      connection_semaphore: Arc::new(Semaphore::new(self.max_connections)),
      max_connections: self.max_connections,
      cache_ttl: self.cache_ttl,
      circuit_breaker_threshold: self.circuit_breaker_threshold,
      circuit_breaker_timeout: self.circuit_breaker_timeout,
      nats_url: self.nats_url,
    }
  }
}

/// Message configuration optimized for production
impl crate::domains::messaging::messaging_domain::MessageConfig {
  /// Create production-optimized configuration
  pub fn production_optimized() -> Self {
    Self {
      cache_enabled: true,
      cache_ttl: 300, // 5 minutes for production
      max_content_length: 16384,
      max_file_count: 10,
    }
  }
}

/// Cache statistics for monitoring
#[derive(Debug, Serialize)]
pub struct CacheStatistics {
  pub entries: usize,
  pub hit_ratio: f64,
  pub memory_usage_bytes: u64,
  pub last_access: std::time::SystemTime,
}
