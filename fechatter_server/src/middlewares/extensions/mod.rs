//! # Extension Module
//!
//! **Responsibilities**: Provides Router extensions and convenient APIs
//! - router_ext: Router extension traits with chainable API
//! - Convenience: Configure complex middleware stacks in one line
//! - Scenarios: Pre-configured settings for common use cases

pub mod router_ext;


// Re-export all router extension traits
pub use router_ext::{
  // Configuration-driven extensions
  ConfigDrivenRouterExtensions,
  // Basic extensions
  RouterExtensions,

  // Scenario-based extensions
  ScenarioRouterExtensions,

  // Smart extensions
  SmartRouterExtensions,
};

// =============================================================================
// Convenient Re-exports - Users only need to import this module for all extensions
// =============================================================================

/// Extension prelude module
///
/// **Responsibility**: Provides one-stop importing of all common extensions
/// **Usage**: `use fechatter_server::middlewares::extensions::prelude::*;`
pub mod prelude {
  pub use super::router_ext::{
    ConfigDrivenRouterExtensions, RouterExtensions, ScenarioRouterExtensions, SmartRouterExtensions,
  };

  pub use crate::middlewares::core::{
    MiddlewareConfig, business_middleware_config, development_middleware_config,
    enterprise_middleware_config, standard_middleware_config,
  };
}
