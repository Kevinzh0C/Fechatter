//! # Services Layer - Main Entry Point
//!
//! **Design Philosophy**: Service layer architecture divided by responsibilities
//! **Layer**: Service Layer - Unified service provision layer

// =============================================================================
// APPLICATION SERVICES
// =============================================================================
pub mod application;

// =============================================================================
// INFRASTRUCTURE SERVICES
// =============================================================================
pub mod infrastructure;

// =============================================================================
// AI SERVICES
// =============================================================================
pub mod ai;

// =============================================================================
// PUBLIC EXPORTS
// =============================================================================

// Re-export core services
pub use application::builders::ServiceProvider;

// Re-export application services
pub use application::*;

// Re-export infrastructure services - focused by responsibility
pub use infrastructure::{EventPublisher, LocalStorage, RedisCacheService, SearchService};
