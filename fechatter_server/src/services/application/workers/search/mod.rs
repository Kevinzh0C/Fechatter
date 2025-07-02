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
    create_legacy_query_processor, create_optimized_query_processor,
    create_query_processor_with_config, OptimizedQuery, QueryIntent, QueryProcessor, QueryType,
    SearchSuggestionGenerator,
};

pub use service::{
    create_search_application_service, create_search_application_service_from_config,
    MessageSearchResults, SearchApplicationService, SearchApplicationServiceTrait, SearchPage,
    SearchableMessage,
};

pub use cache::{
    create_high_performance_search_cache, create_search_cache_service, SearchCacheConfig,
    SearchCacheService, SearchCacheStats,
};

pub use search_adapter::SearchServiceAdapter;
