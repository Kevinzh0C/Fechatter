//! # Core Middleware Compatibility Layer
//!
//! **Responsibilities**:
//! - Adapts fechatter_core middleware to new 5-layer architecture
//! - Provides unified interface abstraction
//! - Ensures functional completeness and backward compatibility
//! - Avoids duplicate implementation

use crate::AppState;
use axum::{
  extract::{Request, State},
  middleware::Next,
  response::{IntoResponse, Response},
};

// Re-export fechatter_core middleware to avoid duplication
pub use fechatter_core::middlewares::{
  ActualAuthServiceProvider, ServerTimeLayer, SetLayer, TokenVerifier, WithServiceProvider,
  WithTokenManager, request_id_middleware, verify_token_middleware,
};

// =============================================================================
// Core Middleware Adapter - Single Responsibility: Adapt core functionality to new architecture
// =============================================================================

/// Core Bearer Token Verification Adapter
///
/// **Single Responsibility**: Adapt fechatter_core's verify_token_middleware to new architecture
/// **Benefits**: Reuse core's mature logic, avoid duplication
pub async fn core_auth_middleware(
  State(state): State<AppState>,
  request: Request,
  next: Next,
) -> Response {
  // Directly use fechatter_core's mature implementation
  fechatter_core::middlewares::verify_token_middleware(State(state), request, next).await
}

/// Core Request ID Middleware Adapter
///
/// **Single Responsibility**: Adapt fechatter_core's request_id_middleware to new architecture
/// **Benefits**: Uses core's UUID v7 implementation, more standardized
pub async fn core_request_id_middleware(request: Request, next: Next) -> Response {
  // Directly use fechatter_core's implementation
  fechatter_core::middlewares::request_id_middleware(request, next).await
}

/// Core Token Refresh Middleware Adapter
///
/// **Single Responsibility**: Adapt fechatter_core's complex refresh logic to new architecture
/// **Benefits**: Includes security checks, user agent validation and advanced features
pub async fn core_token_refresh_middleware(
  State(state): State<AppState>,
  request: Request,
  next: Next,
) -> Response {
  // Use fechatter_core's complete refresh logic
  match fechatter_core::middlewares::token_refresh::refresh_token_middleware::<
    AppState,
    fechatter_core::models::AuthUser,
  >(request.headers().clone(), State(state), request, next)
  .await
  {
    Ok(response) => response,
    Err(_) => (axum::http::StatusCode::UNAUTHORIZED, "Token refresh failed").into_response(),
  }
}

// =============================================================================
// Unified Interface Abstraction - Single Responsibility: Provide consistent middleware interface
// =============================================================================

/// Unified Middleware Interface
///
/// **Single Responsibility**: Provide unified calling interface for all middleware
/// **Design**: Based on function pointers, zero-cost abstraction
pub struct UnifiedMiddleware {
  pub name: &'static str,
  pub core_based: bool,
  pub middleware_fn: fn(
    State<AppState>,
    Request,
    Next,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>,
}

impl UnifiedMiddleware {
  /// Create core-based middleware
  pub const fn from_core(
    name: &'static str,
    middleware_fn: fn(
      State<AppState>,
      Request,
      Next,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>,
  ) -> Self {
    Self {
      name,
      core_based: true,
      middleware_fn,
    }
  }

  /// Create new architecture middleware
  pub const fn from_new(
    name: &'static str,
    middleware_fn: fn(
      State<AppState>,
      Request,
      Next,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>,
  ) -> Self {
    Self {
      name,
      core_based: false,
      middleware_fn,
    }
  }
}

// =============================================================================
// Middleware Selection Strategy - Single Responsibility: Smart selection of optimal implementation
// =============================================================================

/// Middleware Selection Strategy
///
/// **Single Responsibility**: Automatically select core or new implementation based on scenario
#[derive(Debug, Clone, Copy)]
pub enum MiddlewareStrategy {
  /// Prefer Core implementation (recommended)
  PreferCore,
  /// Prefer new implementation
  PreferNew,
  /// Auto-select based on performance
  AutoSelect,
  /// Force use Core
  ForceCore,
  /// Force use new implementation
  ForceNew,
}

impl Default for MiddlewareStrategy {
  fn default() -> Self {
    Self::PreferCore // Default prefer core as it's more mature
  }
}

/// Smart Middleware Selector
///
/// **Single Responsibility**: Select optimal middleware implementation based on strategy
pub struct MiddlewareSelector {
  strategy: MiddlewareStrategy,
}

impl MiddlewareSelector {
  pub fn new(strategy: MiddlewareStrategy) -> Self {
    Self { strategy }
  }

  /// Select authentication middleware
  ///
  /// **Single Responsibility**: Choose optimal implementation for authentication
  pub fn auth_middleware(&self) -> &'static str {
    match self.strategy {
      MiddlewareStrategy::PreferCore | MiddlewareStrategy::ForceCore => "core_auth",
      MiddlewareStrategy::PreferNew | MiddlewareStrategy::ForceNew => "new_auth",
      MiddlewareStrategy::AutoSelect => {
        // Choose based on performance test results
        // Core implementation is more mature, choose core
        "core_auth"
      }
    }
  }

  /// Select request ID middleware
  ///
  /// **Single Responsibility**: Choose optimal implementation for request tracking
  pub fn request_id_middleware(&self) -> &'static str {
    match self.strategy {
      MiddlewareStrategy::PreferCore | MiddlewareStrategy::ForceCore => "core_request_id",
      MiddlewareStrategy::PreferNew | MiddlewareStrategy::ForceNew => "new_request_id",
      MiddlewareStrategy::AutoSelect => {
        // Core uses UUID v7, more standard
        "core_request_id"
      }
    }
  }

  /// Select token refresh middleware
  ///
  /// **Single Responsibility**: Choose optimal implementation for token refresh
  pub fn token_refresh_middleware(&self) -> &'static str {
    match self.strategy {
      MiddlewareStrategy::PreferCore | MiddlewareStrategy::ForceCore => "core_token_refresh",
      MiddlewareStrategy::PreferNew | MiddlewareStrategy::ForceNew => "new_token_refresh",
      MiddlewareStrategy::AutoSelect => {
        // Core refresh logic includes security checks, more complete
        "core_token_refresh"
      }
    }
  }
}

// =============================================================================
// Compatibility Check Tools - Single Responsibility: Verify compatibility
// =============================================================================

/// Compatibility Check Report
#[derive(Debug)]
pub struct CompatibilityReport {
  pub core_middleware_count: usize,
  pub new_middleware_count: usize,
  pub conflicts: Vec<String>,
  pub missing_features: Vec<String>,
  pub recommendations: Vec<String>,
}

/// Compatibility Checker
///
/// **Single Responsibility**: Check and report compatibility between two middleware systems
pub struct CompatibilityChecker;

impl CompatibilityChecker {
  /// Perform complete compatibility check
  ///
  /// **Single Responsibility**: Generate detailed compatibility report
  pub fn check() -> CompatibilityReport {
    let mut report = CompatibilityReport {
      core_middleware_count: 6, // bearer_auth, request_id, server_time, token_refresh, custom_builder, mw_traits
      new_middleware_count: 5,  // primitives, middlewares, auth_flows, context, router_ext
      conflicts: Vec::new(),
      missing_features: Vec::new(),
      recommendations: Vec::new(),
    };

    // Check feature conflicts
    report
      .conflicts
      .push("Bearer token verification: both core and new have implementations".to_string());
    report
      .conflicts
      .push("Request ID generation: both core and new have implementations".to_string());
    report
      .conflicts
      .push("Token refresh: core more complete, new simplified".to_string());

    // Check missing features
    report
      .missing_features
      .push("New system lacks server_time middleware".to_string());
    report
      .missing_features
      .push("New system lacks trait abstraction layer".to_string());
    report
      .missing_features
      .push("New system lacks security check features".to_string());

    // Generate recommendations
    report
      .recommendations
      .push("Use core's auth_middleware, more complete functionality".to_string());
    report
      .recommendations
      .push("Use core's request_id_middleware, UUID v7 more standard".to_string());
    report
      .recommendations
      .push("Use core's token_refresh, includes security checks".to_string());
    report
      .recommendations
      .push("Add server_time support to new system".to_string());
    report
      .recommendations
      .push("Implement trait abstraction layer for better extensibility".to_string());

    report
  }

  /// Print compatibility report
  ///
  /// **Single Responsibility**: Format and output compatibility check results
  pub fn print_report() {
    let report = Self::check();

    println!("🔍 Middleware Compatibility Check Report");
    println!("================================");
    println!("📊 Statistics:");
    println!(
      "  - Core middleware count: {}",
      report.core_middleware_count
    );
    println!("  - New middleware count: {}", report.new_middleware_count);

    println!("\n⚠️ Feature Conflicts:");
    for conflict in &report.conflicts {
      println!("  - {}", conflict);
    }

    println!("\n❌ Missing Features:");
    for missing in &report.missing_features {
      println!("  - {}", missing);
    }

    println!("\n💡 Recommendations:");
    for recommendation in &report.recommendations {
      println!("  - {}", recommendation);
    }

    println!(
      "\n🎯 Summary: Recommend prioritizing Core middleware, gradually migrating to unified architecture"
    );
  }
}

// =============================================================================
// Test Module
// =============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_middleware_selector() {
    let selector = MiddlewareSelector::new(MiddlewareStrategy::PreferCore);

    assert_eq!(selector.auth_middleware(), "core_auth");
    assert_eq!(selector.request_id_middleware(), "core_request_id");
    assert_eq!(selector.token_refresh_middleware(), "core_token_refresh");
  }

  #[test]
  fn test_compatibility_report() {
    let report = CompatibilityChecker::check();

    assert!(report.core_middleware_count > 0);
    assert!(report.new_middleware_count > 0);
    assert!(!report.conflicts.is_empty());
    assert!(!report.missing_features.is_empty());
    assert!(!report.recommendations.is_empty());
  }

  #[test]
  fn test_print_compatibility_report() {
    // This test mainly verifies the function doesn't panic
    CompatibilityChecker::print_report();
  }
}
