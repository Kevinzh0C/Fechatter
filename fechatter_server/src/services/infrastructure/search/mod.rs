//! # Search Infrastructure
//!
//! **Responsibility**: Concrete implementations of search engines (Meilisearch, Elasticsearch etc.)
//! **Layer**: Infrastructure Layer - Technical Implementation Layer
//!
//! ## Architecture Refactoring Notes
//!
//! **Problems**:
//! - Naming conflicts: Infrastructure layer's `SearchService` conflicts with application layer's `SearchApplicationService`
//! - Layer confusion: Application layer depends on infrastructure layer, but naming suggests reverse dependency
//!
//! **Solutions**:
//! - **Infrastructure Layer**: Provides concrete search engine implementations (`MeilisearchBackend`, `ElasticsearchBackend`)
//! - **Application Layer**: Provides business logic encapsulation (`SearchApplicationService`)
//! - **Core Layer**: Provides abstract interfaces (`SearchService` trait)

// Search engine implementation modules
pub mod meilisearch;
pub mod search_service;

// ── Explicit exports: Infrastructure concrete implementations ────────────────────────────────────────

/// Meilisearch search engine client
pub use meilisearch::MeilisearchClient;

/// Infrastructure layer search service implementation (avoiding naming conflicts with application layer)
pub use search_service::{
    create_search_service,
    create_search_service_with_batch_size,
    MeilisearchBackend,
    SearchBackend,
    SearchFilters,
    SearchService as InfraSearchService, // Renamed to avoid conflicts
    SearchServiceBuilder,
};
