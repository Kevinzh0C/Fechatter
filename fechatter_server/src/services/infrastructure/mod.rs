//! # Infrastructure Services Layer - 基础设施服务层
//!
//! **职责**: 技术实现抽象 - 数据库、缓存、搜索、存储等
//! **层级**: Infrastructure Service Layer (Layer 4)

// Infrastructure services
pub mod cache;
pub mod search;
pub mod storage;
pub mod vector_db;

// Event services module
pub mod event;

// AI services - temporarily commented until path is fixed
// pub mod ai;

// Notification infrastructure
pub mod notification;

// Messaging infrastructure
pub mod messaging;

pub mod third_party_manager;

// Re-exports - 按职责导出核心基础设施服务
pub use event::event_publisher::NatsEventPublisher as EventPublisher;

// Cache services
pub use cache::{Cache, RedisCacheService};

// Search services
pub use search::{MeilisearchClient, SearchService};

// Storage services
pub use storage::LocalStorage;

// Vector database services - TODO: Re-enable when types are implemented
// pub use vector_db::{
//   MessageEmbedding, MessageVectorRepository, PgVectorDatabase, PineconeClient, VectorDatabase,
//   VectorSearchResult,
// };

// Temporary placeholder
pub use vector_db::PlaceholderVectorDb;
