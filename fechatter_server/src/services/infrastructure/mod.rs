//! # Infrastructure Services Layer - 基础设施服务层
//!
//! **职责**: 技术实现抽象 - 数据库、缓存、搜索、存储等
//! **层级**: Infrastructure Service Layer (Layer 4)
//!
//! ## 🔄 重要架构调整：Flows 迁移到 Infrastructure
//!
//! **决策原因**:
//! - Flows 主要提供技术基础设施服务 (消息传输、事件发布、通知投递)
//! - 专注于性能优化和系统集成，而非业务逻辑
//! - 更符合 Infrastructure Layer 的职责定义
//!
//! **迁移后的分层**:
//! - **Application Layer**: 业务用例编排、权限验证、跨领域协调
//! - **Infrastructure Layer**: 消息流、事件流、通知流等技术基础设施

// Infrastructure services
pub mod cache;
pub mod event;
pub mod event_publisher;
pub mod events;
pub mod flows;
pub mod notification;
pub mod observability;
pub mod search;
pub mod storage;
pub mod third_party_manager;
pub mod vector_db;

// Re-exports - 按职责导出核心基础设施服务
pub use event::LegacyEventPublisher as EventPublisher;

// Cache services
pub use cache::{Cache, RedisCacheService};

// Search services
pub use search::{InfraSearchService as SearchService, MeilisearchClient, SearchServiceBuilder};

// Storage services
pub use storage::LocalStorage;

// Vector database services - TODO: Re-enable when types are implemented
// pub use vector_db::{
//   MessageEmbedding, MessageVectorRepository, PgVectorDatabase, PineconeClient, VectorDatabase,
//   VectorSearchResult,
// };

// Temporary placeholder
pub use vector_db::PlaceholderVectorDb;

// Enhanced event publisher exports from unified event system
pub use event::{
  CompleteMessageData, EnhancedEventPublisher, NotifyMessageEvent, NotifyChatMemberEvent, 
  NotifyReadReceiptEvent, message_to_complete_data, create_enhanced_publisher_for_notify_server,
};

// Legacy flows (complex business workflows)
pub use flows::{
  RealtimeStreamService, DomainEventService, NotificationFlowService, InstantMessagingService
};

// Legacy event publisher (DEPRECATED - use event::EnhancedEventPublisher instead)
#[deprecated(since = "2.0.0", note = "Use event::EnhancedEventPublisher instead")]
pub use events::EventEnvelope;
