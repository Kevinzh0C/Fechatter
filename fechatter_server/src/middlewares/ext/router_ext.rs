//! # Router Extension Layer - User-friendly Middleware API
//!
//! **Design Principles**:
//! - Provide intuitive and easy-to-use router extension methods
//! - Encapsulate complex middleware combination logic
//! - Support chaining and fluent API
//! - Single Responsibility: Each method adds only specific middleware functionality

use axum::{Router, middleware::from_fn, middleware::from_fn_with_state};

use crate::{
  AppState,
  middlewares::{
    chat::verify_chat_membership_middleware,
    composed::auth_flows::{
      auth_with_observability_middleware, auth_with_refresh_middleware, enhanced_auth_middleware,
      full_auth_middleware, standard_observability_middleware,
    },
    core::middlewares::{
      auth_middleware, chat_middleware, error_handling_middleware, request_tracking_middleware,
      server_time_middleware, token_refresh_middleware, workspace_middleware,
    },
  },
};

/// Extended Router functionality with middleware application
pub trait RouterExt {
  /// Apply standard authentication stack
  fn with_auth(self, state: AppState) -> Self;
  /// Apply chat membership verification  
  fn with_chat_middleware(self, state: AppState) -> Self;
  /// Apply workspace middleware
  fn with_workspace_middleware(self, state: AppState) -> Self;
}

impl RouterExt for Router<AppState> {
  fn with_auth(self, state: AppState) -> Self {
    self.layer(from_fn_with_state(state, auth_middleware))
  }

  fn with_chat_middleware(self, state: AppState) -> Self {
    self.layer(from_fn_with_state(state, verify_chat_membership_middleware))
  }

  fn with_workspace_middleware(self, state: AppState) -> Self {
    self.layer(from_fn_with_state(state, workspace_middleware))
  }
}

// =============================================================================
// Basic Middleware Extensions - Single Responsibility: Router extensions for individual middlewares
// =============================================================================

/// Router Middleware Extension Trait
///
/// **Single Responsibility**: Provides middleware extension methods for Router
/// **Design**: Each method adds only one specific middleware functionality
pub trait RouterMiddlewareExt<S>: Sized {
  // Authentication Extensions

  /// Add pure authentication middleware
  ///
  /// **Single Responsibility**: Only adds basic Bearer token authentication
  /// **Use Case**: Simple API authentication, no token refresh needed
  fn with_auth(self, state: AppState) -> Router<S>;

  /// Add token refresh middleware
  ///
  /// **Single Responsibility**: Only handles refresh token logic
  /// **Use Case**: Endpoints requiring automatic token refresh
  fn with_token_refresh(self, state: AppState) -> Router<S>;

  /// Add workspace validation middleware
  ///
  /// **Single Responsibility**: Only validates workspace permissions
  /// **Prerequisite**: Requires prior authentication
  fn with_workspace(self, state: AppState) -> Router<S>;

  /// Add chat validation middleware
  ///
  /// **Single Responsibility**: Only validates chat permissions
  /// **Prerequisite**: Requires prior authentication and workspace validation
  fn with_chat(self, state: AppState) -> Router<S>;

  // Observability Extensions

  /// Add request tracking middleware
  ///
  /// **Single Responsibility**: Only adds request ID tracking
  /// **Function**: Generates unique ID for each request
  fn with_request_tracking(self) -> Router<S>;

  /// Add server time middleware
  ///
  /// **Single Responsibility**: Only adds server time in response headers
  /// **Function**: Records request processing time
  fn with_server_time(self) -> Router<S>;

  /// Add error handling middleware
  ///
  /// **Single Responsibility**: Only handles error response standardization
  /// **Function**: Adds unified format for error responses
  fn with_error_handling(self) -> Router<S>;
}

/// Implement basic middleware extensions for Router
impl<S> RouterMiddlewareExt<S> for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn with_auth(self, state: AppState) -> Router<S> {
    self.layer(from_fn_with_state(state, auth_middleware))
  }

  fn with_token_refresh(self, state: AppState) -> Router<S> {
    self.layer(from_fn_with_state(state, token_refresh_middleware))
  }

  fn with_workspace(self, state: AppState) -> Router<S> {
    self.layer(from_fn_with_state(state, workspace_middleware))
  }

  fn with_chat(self, state: AppState) -> Router<S> {
    self.layer(from_fn_with_state(state, chat_middleware))
  }

  fn with_request_tracking(self) -> Router<S> {
    self.layer(from_fn(request_tracking_middleware))
  }

  fn with_server_time(self) -> Router<S> {
    self.layer(from_fn(server_time_middleware))
  }

  fn with_error_handling(self) -> Router<S> {
    self.layer(from_fn(error_handling_middleware))
  }
}

// =============================================================================
// Composite Middleware Extensions - Single Responsibility: Router extensions for complex flows
// =============================================================================

/// Advanced Router Middleware Extension Trait
///
/// **Single Responsibility**: Provides complex middleware combination methods for Router
/// **Design**: Each method represents a complete business flow
pub trait RouterAdvancedExt<S>: Sized {
  // Authentication Flow Extensions

  /// Add authentication + refresh flow
  ///
  /// **Single Responsibility**: Combines authentication and auto-refresh functionality
  /// **Use Case**: APIs requiring seamless user experience
  fn with_auth_and_refresh(self, state: AppState) -> Router<S>;

  /// Add complete permission validation flow
  ///
  /// **Single Responsibility**: Combines authentication, workspace, and chat permission validation
  /// **Use Case**: Chat-related API endpoints
  fn with_full_auth(self, state: AppState) -> Router<S>;

  /// Add enhanced authentication flow
  ///
  /// **Single Responsibility**: Combines authentication + refresh + workspace validation
  /// **Use Case**: Workspace-level APIs requiring auto-refresh
  fn with_enhanced_auth(self, state: AppState) -> Router<S>;

  // Observability Flow Extensions

  /// Add standard observability flow
  ///
  /// **Single Responsibility**: Combines request tracking and error handling
  /// **Use Case**: All APIs requiring monitoring
  fn with_observability(self) -> Router<S>;

  /// Add authentication + observability flow
  ///
  /// **Single Responsibility**: Combines authentication and observability functionality
  /// **Use Case**: Authentication APIs requiring monitoring
  fn with_auth_observability(self, state: AppState) -> Router<S>;
}

/// Implement advanced middleware extensions for Router
impl<S> RouterAdvancedExt<S> for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn with_auth_and_refresh(self, state: AppState) -> Router<S> {
    self.layer(from_fn_with_state(state, auth_with_refresh_middleware))
  }

  fn with_full_auth(self, state: AppState) -> Router<S> {
    self.layer(from_fn_with_state(state, full_auth_middleware))
  }

  fn with_enhanced_auth(self, state: AppState) -> Router<S> {
    self.layer(from_fn_with_state(state, enhanced_auth_middleware))
  }

  fn with_observability(self) -> Router<S> {
    self.layer(from_fn(standard_observability_middleware))
  }

  fn with_auth_observability(self, state: AppState) -> Router<S> {
    self.layer(from_fn_with_state(
      state,
      auth_with_observability_middleware,
    ))
  }
}

// =============================================================================
// Progressive Complexity Extensions - Single Responsibility: Provide different API levels by complexity
// =============================================================================

/// Progressive Middleware Extension Trait
///
/// **Single Responsibility**: Provides middleware combinations for different application complexity levels
/// **Design**: From simple to complex, meeting different application scenarios
pub trait RouterProgressiveExt<S>: Sized {
  /// P0 Level: Basic Security Authentication
  ///
  /// **Single Responsibility**: Provides most basic authentication functionality
  /// **Includes**: Authentication + Error Handling
  /// **Suitable**: Simple APIs, basic security needs
  fn with_basic_security(self, state: AppState) -> Router<S>;

  /// P1 Level: Enhanced User Experience
  ///
  /// **Single Responsibility**: Adds user experience on top of basic security
  /// **Includes**: Authentication+Refresh + Request Tracking + Error Handling
  /// **Suitable**: User APIs requiring good experience
  fn with_enhanced_ux(self, state: AppState) -> Router<S>;

  /// P2 Level: Complete Observability
  ///
  /// **Single Responsibility**: Provides complete monitoring and observability
  /// **Includes**: Authentication+Refresh + Complete Tracking + Error Handling + Server Time
  /// **Suitable**: Production environment requiring full monitoring
  fn with_full_observability(self, state: AppState) -> Router<S>;

  /// P3 Level: Enterprise Security
  ///
  /// **Single Responsibility**: Provides enterprise-grade complete security and monitoring
  /// **Includes**: Complete Permission Validation + Complete Observability
  /// **Suitable**: Enterprise applications, strict permission control
  fn with_enterprise_security(self, state: AppState) -> Router<S>;
}

/// Implement progressive extensions for Router
impl<S> RouterProgressiveExt<S> for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn with_basic_security(self, state: AppState) -> Router<S> {
    self.with_auth(state).with_error_handling()
  }

  fn with_enhanced_ux(self, state: AppState) -> Router<S> {
    self
      .with_auth_and_refresh(state)
      .with_request_tracking()
      .with_error_handling()
  }

  fn with_full_observability(self, state: AppState) -> Router<S> {
    self
      .with_auth_and_refresh(state)
      .with_request_tracking()
      .with_server_time()
      .with_error_handling()
  }

  fn with_enterprise_security(self, state: AppState) -> Router<S> {
    self
      .with_full_auth(state)
      .with_request_tracking()
      .with_server_time()
      .with_error_handling()
  }
}

// =============================================================================
// Specific Scenario Extensions - Single Responsibility: Middleware combinations customized for specific use cases
// =============================================================================

/// Scenario-specific Middleware Extension Trait
///
/// **Single Responsibility**: Provides customized middleware combinations for common use cases
/// **Design**: Pre-configured best practices based on business scenarios
pub trait RouterScenarioExt<S>: Sized {
  /// User API Scenario
  ///
  /// **Single Responsibility**: Standard configuration for user-related APIs
  /// **Includes**: Authentication+Refresh + Tracking + Error Handling
  /// **Scenario**: User profile, settings APIs
  fn for_user_apis(self, state: AppState) -> Router<S>;

  /// Workspace API Scenario
  ///
  /// **Single Responsibility**: Standard configuration for workspace-related APIs
  /// **Includes**: Enhanced Authentication + Tracking + Error Handling
  /// **Scenario**: Workspace management, file management APIs
  fn for_workspace_apis(self, state: AppState) -> Router<S>;

  /// Chat API Scenario
  ///
  /// **Single Responsibility**: Standard configuration for chat-related APIs
  /// **Includes**: Complete Permission Validation + Complete Observability
  /// **Scenario**: Message sending, chat management APIs
  fn for_chat_apis(self, state: AppState) -> Router<S>;

  /// Public API Scenario
  ///
  /// **Single Responsibility**: Standard configuration for public APIs
  /// **Includes**: Only basic tracking and error handling, no authentication needed
  /// **Scenario**: Health checks, public information APIs
  fn for_public_apis(self) -> Router<S>;

  /// Admin API Scenario
  ///
  /// **Single Responsibility**: Standard configuration for admin backend APIs
  /// **Includes**: Enterprise Security + Complete Monitoring
  /// **Scenario**: System management, user management APIs
  fn for_admin_apis(self, state: AppState) -> Router<S>;
}

/// Implement scenario-specific extensions for Router
impl<S> RouterScenarioExt<S> for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn for_user_apis(self, state: AppState) -> Router<S> {
    self.with_enhanced_ux(state)
  }

  fn for_workspace_apis(self, state: AppState) -> Router<S> {
    self
      .with_enhanced_auth(state)
      .with_request_tracking()
      .with_error_handling()
  }

  fn for_chat_apis(self, state: AppState) -> Router<S> {
    self.with_enterprise_security(state)
  }

  fn for_public_apis(self) -> Router<S> {
    self
      .with_request_tracking()
      .with_server_time()
      .with_error_handling()
  }

  fn for_admin_apis(self, state: AppState) -> Router<S> {
    self.with_enterprise_security(state)
  }
}

// =============================================================================
// One-click Configuration API - Single Responsibility: Provide simplest configuration method
// =============================================================================

/// Quick Setup Extension Trait
///
/// **Single Responsibility**: Provides simplest direct configuration methods
/// **Design**: Complete configuration in one method, suitable for rapid development
pub trait RouterQuickSetupExt<S>: Sized {
  /// Quick Development Configuration
  ///
  /// **Single Responsibility**: One-click configuration for quick development environment
  /// **Feature**: Complete functionality but simple configuration
  fn quick_setup(self, state: AppState) -> Router<S>;

  /// Production Environment Configuration
  ///
  /// **Single Responsibility**: One-click configuration for production environment
  /// **Feature**: Complete security and monitoring configuration
  fn production_setup(self, state: AppState) -> Router<S>;

  /// Development Environment Configuration
  ///
  /// **Single Responsibility**: One-click configuration for development environment
  /// **Feature**: Basic functionality, easy to debug
  fn development_setup(self, state: AppState) -> Router<S>;
}

/// Implement quick setup extensions for Router
impl<S> RouterQuickSetupExt<S> for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn quick_setup(self, state: AppState) -> Router<S> {
    self.with_enhanced_ux(state)
  }

  fn production_setup(self, state: AppState) -> Router<S> {
    self.with_enterprise_security(state)
  }

  fn development_setup(self, state: AppState) -> Router<S> {
    self.with_basic_security(state).with_request_tracking()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::routing::get;

  // Mock a simple handler for testing
  async fn test_handler() -> &'static str {
    "test"
  }

  #[test]
  fn test_trait_compilation() {
    // This test ensures the traits compile correctly
    // We can't test actual functionality without a real AppState
    assert!(true);
  }
}
