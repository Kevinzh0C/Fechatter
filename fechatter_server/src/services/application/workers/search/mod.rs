//! # Search Domain
//!
//! **Responsibility**: All search-related use cases
//! **Architecture**: Layered design with clear responsibilities

// ================================================================================================
// Core Search Components
// ================================================================================================

/// Query Processor - Handles query preprocessing and optimization
pub mod query_processor;

/// Search Service - Application layer search logic
pub mod service;

/// Search Service Adapter - Adapts infrastructure search to application interface
pub mod search_adapter;

// ================================================================================================
// Cache Components (Reorganized)
// ================================================================================================

/// Search Cache Strategy - Cache optimization specifically for search
pub mod cache;

// ================================================================================================
// Re-export Core Types
// ================================================================================================

pub use query_processor::{
  OptimizedQuery, QueryIntent, QueryProcessor, QueryType, SearchSuggestionGenerator,
  create_legacy_query_processor, create_optimized_query_processor,
  create_query_processor_with_config,
};

pub use service::{
  MessageSearchResults, SearchApplicationService, SearchApplicationServiceTrait, SearchPage,
  SearchableMessage, create_search_application_service,
  create_search_application_service_from_config,
};

pub use cache::{
  SearchCacheConfig, SearchCacheService, SearchCacheStats, create_high_performance_search_cache,
  create_search_cache_service,
};

pub use search_adapter::SearchServiceAdapter;
