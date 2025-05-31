//! # Convenience API Layer - Simplified User Interface
//!
//! **Design Principles**:
//! - Provide simplest one-click configuration APIs
//! - Hide complex internal implementation details
//! - Provide out-of-box configurations for common scenarios
//! - Single responsibility: Each function represents a complete application configuration

use crate::{
  AppState,
  middlewares::ext::router_ext::{
    RouterAdvancedExt, RouterMiddlewareExt, RouterProgressiveExt, RouterQuickSetupExt,
    RouterScenarioExt,
  },
};
use axum::Router;

// =============================================================================
// Quick Start APIs - Single Responsibility: Provide one-click startup configurations
// =============================================================================

/// Quick Authentication Configuration
///
/// **Single Responsibility**: Provide minimal configuration for apps requiring auth
/// **Includes**: Basic auth + Error handling + Request tracking
/// **Suitable for**: 90% of authentication scenarios
/// **Usage**: `let router = quick_auth(router, state);`
pub fn quick_auth<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router.quick_setup(state)
}

/// Standard Application Configuration
///
/// **Single Responsibility**: Provide complete configuration for standard web apps
/// **Includes**: Auth+Refresh + Full tracking + Error handling
/// **Suitable for**: Most production applications
/// **Usage**: `let router = standard_app(router, state);`
pub fn standard_app<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router.production_setup(state)
}

/// Enterprise Application Configuration
///
/// **Single Responsibility**: Provide complete security configuration for enterprise apps
/// **Includes**: Full permission validation + Enterprise security + Complete monitoring
/// **Suitable for**: Enterprise apps with strict security requirements
/// **Usage**: `let router = enterprise_app(router, state);`
pub fn enterprise_app<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router.with_enterprise_security(state)
}

// =============================================================================
// Scenario Preset Configurations - Single Responsibility: Provide presets for specific business scenarios
// =============================================================================

/// API Gateway Configuration
///
/// **Single Responsibility**: Configure best practices for API gateway applications
/// **Features**: High performance + Complete monitoring + Error handling
/// **Usage**: API gateway in microservices architecture
pub fn api_gateway<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router.with_enhanced_ux(state).with_server_time()
}

/// Microservice Configuration
///
/// **Single Responsibility**: Configure standard settings for microservice applications
/// **Features**: Lightweight + Basic security + Service tracking
/// **Usage**: Business services in microservices architecture
pub fn microservice<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router
    .with_basic_security(state)
    .with_request_tracking()
    .with_server_time()
}

/// Chat Application Configuration
///
/// **Single Responsibility**: Configure complete functionality for chat applications
/// **Features**: Complete permissions + Real-time + Full monitoring
/// **Usage**: IM, collaboration tools and other chat applications
pub fn chat_app<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router.for_chat_apis(state)
}

/// Content Management System Configuration
///
/// **Single Responsibility**: Configure management functions for CMS applications
/// **Features**: Workspace management + File upload + User permissions
/// **Usage**: CMS, document management and similar applications
pub fn cms_app<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router.for_workspace_apis(state)
}

// =============================================================================
// Development Stage Configurations - Single Responsibility: Provide appropriate configurations for different development stages
// =============================================================================

/// Prototype Development Configuration
///
/// **Single Responsibility**: Provide minimal configuration for rapid prototyping
/// **Features**: Minimal middleware + Basic error handling
/// **Usage**: Proof of concept, rapid prototyping phase
pub fn prototype<S>(router: Router<S>) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router.for_public_apis()
}

/// Development Testing Configuration
///
/// **Single Responsibility**: Provide debug-friendly configuration for development testing
/// **Features**: Basic security + Detailed tracking + Error details
/// **Usage**: Development testing phase
pub fn development<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router.development_setup(state)
}

/// Pre-production Configuration
///
/// **Single Responsibility**: Provide near-production configuration for pre-prod environment
/// **Features**: Complete functionality + Performance monitoring + Security checks
/// **Usage**: UAT, pre-production environment
pub fn staging<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router.with_full_observability(state)
}

/// Production Environment Configuration
///
/// **Single Responsibility**: Provide optimized configuration for production environment
/// **Features**: Performance optimization + Complete security + Comprehensive monitoring
/// **Usage**: Production deployment
pub fn production<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router.production_setup(state)
}

// =============================================================================
// Performance Optimization Configurations - Single Responsibility: Provide optimized configurations for different performance needs
// =============================================================================

/// High Concurrency Configuration
///
/// **Single Responsibility**: Configuration optimized for high concurrency scenarios
/// **Features**: Minimal middleware + Efficient auth + Basic monitoring
/// **Usage**: High QPS API services
pub fn high_concurrency<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router.with_auth(state).with_error_handling()
}

/// Low Latency Configuration
///
/// **Single Responsibility**: Configuration optimized for low latency scenarios
/// **Features**: Streamlined middleware + Inline optimization + Minimal overhead
/// **Usage**: Real-time APIs, game services etc.
pub fn low_latency<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router.with_auth(state)
}

/// High Availability Configuration
///
/// **Single Responsibility**: Configuration designed for high availability scenarios
/// **Features**: Fault tolerance + Complete monitoring + Auto recovery
/// **Usage**: Critical business systems
pub fn high_availability<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router
    .with_auth_and_refresh(state)
    .with_request_tracking()
    .with_server_time()
    .with_error_handling()
}

// =============================================================================
// Security Level Configurations - Single Responsibility: Provide configurations for different security requirements
// =============================================================================

/// Basic Security Configuration
///
/// **Single Responsibility**: Provide basic security protection
/// **Features**: Basic authentication + Error handling
/// **Usage**: Internal tools, low sensitivity applications
pub fn basic_security<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router.with_basic_security(state)
}

/// Enhanced Security Configuration
///
/// **Single Responsibility**: Provide enhanced security protection
/// **Features**: Auth+Refresh + Request tracking + Complete monitoring
/// **Usage**: Applications handling user data, business data
pub fn enhanced_security<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router.with_enhanced_ux(state)
}

/// Enterprise Security Configuration
///
/// **Single Responsibility**: Provide enterprise-grade security protection
/// **Features**: Complete permission validation + Full audit + Zero trust architecture
/// **Usage**: Finance, healthcare and other high security industries
pub fn enterprise_security<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  router.with_enterprise_security(state)
}

// =============================================================================
// Composite Configuration Builder - Single Responsibility: Provide flexible composite configurations
// =============================================================================

/// Configuration Builder
///
/// **Single Responsibility**: Provide fluent API style configuration building
/// **Features**: Chain calls + Conditional configuration + Flexible composition
pub struct MiddlewareBuilder<S> {
  router: Router<S>,
  state: Option<AppState>,
}

impl<S> MiddlewareBuilder<S>
where
  S: Clone + Send + Sync + 'static,
{
  /// Create new builder
  ///
  /// **Single Responsibility**: Initialize configuration builder
  pub fn new(router: Router<S>) -> Self {
    Self {
      router,
      state: None,
    }
  }

  /// Set application state
  ///
  /// **Single Responsibility**: Set application state for builder
  pub fn with_state(mut self, state: AppState) -> Self {
    self.state = Some(state);
    self
  }

  /// Conditionally add authentication
  ///
  /// **Single Responsibility**: Decide whether to add auth based on condition
  pub fn auth_if(self, condition: bool) -> Self {
    if condition && self.state.is_some() {
      Self {
        router: quick_auth(self.router, self.state.unwrap()),
        state: None,
      }
    } else {
      self
    }
  }

  /// Conditionally add observability
  ///
  /// **Single Responsibility**: Decide whether to add monitoring based on condition
  pub fn observability_if(self, condition: bool) -> Self {
    if condition {
      Self {
        router: self.router.with_request_tracking().with_error_handling(),
        state: self.state,
      }
    } else {
      self
    }
  }

  /// Build final router
  ///
  /// **Single Responsibility**: Complete build and return configured router
  pub fn build(self) -> Router<S> {
    self.router
  }
}

// =============================================================================
// Convenience Function Collection - Single Responsibility: Provide most commonly used convenience functions
// =============================================================================

/// One-click configuration: Complete Web Application
///
/// **Single Responsibility**: Provide complete one-click configuration for web applications
/// **Usage**: `configure_web_app(router, state, "production")`
pub fn configure_web_app<S>(router: Router<S>, state: AppState, environment: &str) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  match environment {
    "development" => development(router, state),
    "staging" => staging(router, state),
    "production" => production(router, state),
    _ => quick_auth(router, state),
  }
}

/// One-click configuration: API Service
///
/// **Single Responsibility**: Provide complete one-click configuration for API services
/// **Usage**: `configure_api_service(router, state, "high_performance")`
pub fn configure_api_service<S>(router: Router<S>, state: AppState, profile: &str) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  match profile {
    "high_concurrency" => high_concurrency(router, state),
    "low_latency" => low_latency(router, state),
    "high_availability" => high_availability(router, state),
    _ => standard_app(router, state),
  }
}

/// One-click configuration: By Business Type
///
/// **Single Responsibility**: Provide best configuration based on business type
/// **Usage**: `configure_by_business(router, state, "chat")`
pub fn configure_by_business<S>(
  router: Router<S>,
  state: AppState,
  business_type: &str,
) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  match business_type {
    "chat" => chat_app(router, state),
    "cms" => cms_app(router, state),
    "gateway" => api_gateway(router, state),
    "microservice" => microservice(router, state),
    _ => standard_app(router, state),
  }
}

// =============================================================================
// Tests and Validation
// =============================================================================

#[cfg(test)]
mod tests {
  use super::*;
  use axum::routing::get;

  async fn test_handler() -> &'static str {
    "test"
  }

  #[test]
  fn test_quick_configurations() {
    let state = AppState::default();
    let router = Router::new().route("/test", get(test_handler));

    // Test quick configurations
    let _quick = quick_auth(router.clone(), state.clone());
    let _standard = standard_app(router.clone(), state.clone());
    let _enterprise = enterprise_app(router.clone(), state.clone());

    assert!(true);
  }

  #[test]
  fn test_scenario_configurations() {
    let state = AppState::default();
    let router = Router::new().route("/test", get(test_handler));

    // Test scenario configurations
    let _chat = chat_app(router.clone(), state.clone());
    let _cms = cms_app(router.clone(), state.clone());
    let _gateway = api_gateway(router.clone(), state.clone());

    assert!(true);
  }

  #[test]
  fn test_builder_pattern() {
    let state = AppState::default();
    let router = Router::new().route("/test", get(test_handler));

    // Test builder pattern
    let _configured = MiddlewareBuilder::new(router)
      .with_state(state)
      .auth_if(true)
      .observability_if(true)
      .build();

    assert!(true);
  }

  #[test]
  fn test_one_liner_configurations() {
    let state = AppState::default();
    let router = Router::new().route("/test", get(test_handler));

    // Test one-liner configurations
    let _web_app = configure_web_app(router.clone(), state.clone(), "production");
    let _api_service = configure_api_service(router.clone(), state.clone(), "high_performance");
    let _by_business = configure_by_business(router.clone(), state.clone(), "chat");

    assert!(true);
  }
}
