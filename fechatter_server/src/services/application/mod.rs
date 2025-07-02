//! # Application Services - 人体工程学架构
//!
//! ## 设计理念
//! 每个模块职责清晰，协调工作，互不重复，共同支撑整个软件的生命力。
//!
//! ## 📂 模块架构
//! - **builders** - 构建器：创建和管理服务实例  
//! - **workers** - 工作者：执行具体业务逻辑
//! - **flows** - 流动：事件传递和消息流转
//! - **stores** - 存储：缓存和数据管理
//! - **tools** - 工具：辅助功能和基础设施
//! - **legacy** - 兼容：向后兼容代码

// ============================================================================
// 人体工程学模块结构
// ============================================================================

/// 构建器 - 创建和管理服务实例
pub mod builders;

/// 工作者 - 执行具体业务逻辑  
pub mod workers;

// 流动 - 已迁移到 infrastructure/flows

/// 存储 - 缓存和数据管理
pub mod stores;

/// 工具 - 辅助功能和基础设施
pub mod tools;

/// 兼容 - 向后兼容代码
pub mod legacy;

// ============================================================================
// 统一导出 - 保持API简洁
// ============================================================================

// 工作者导出 - 简化版本，无traits
pub use workers::{
    // 认证工作者
    auth::AuthUserService,
    // 聊天工作者
    chat::{ChatDetailView, ChatService, CreateChatInput},
    // 消息工作者
    message::{MessageApplicationService, MessageView},
    // 搜索工作者
    search::{MessageSearchResults, SearchApplicationService},
    // 工作空间工作者
    workspace::{
        create_workspace_application_service, AddMembersCommand, InviteUserCommand,
        UpdateWorkspaceCommand, UserSummaryView, WorkspaceService, WorkspaceView,
    },
};

// 流动导出 - 从 infrastructure/flows 重新导出
pub use crate::services::infrastructure::flows::{
    ChatDomainEvent,
    DomainEvent,
    DomainEventService,
    MessageDomainEvent,
    // 通知流
    NotificationFlowService,
    NotificationPriority,
    NotificationService,
    NotificationType,
    RealtimeStreamEvent,
    // 实时流
    RealtimeStreamService,
    SimpleNotificationType,
    // 事件流 - 导出实际存在的类型
    SimplifiedEventPublisher,
    SystemDomainEvent,
    UserDomainEvent,
};

// 存储导出
pub use stores::{
    CacheDataType,
    // 缓存存储
    CacheStrategyService,
    InvalidationPattern,
};

// ============================================================================
// 类型别名 - 提供简洁引用
// ============================================================================

/// 应用缓存服务
pub type AppCacheService = CacheStrategyService;

/// 应用事件发布器
pub type AppEventPublisher = SimplifiedEventPublisher;

/// 应用消息流服务 (temporarily disabled)
// pub type AppMessageStreamService = DualStreamMessagingService;

// ============================================================================
// 兼容性导出（逐步废弃）
// ============================================================================

#[deprecated(note = "Use workers, flows, stores modules directly")]
pub use legacy::*;

// ============================================================================
// 向后兼容模块（保持现有导入路径工作）
// ============================================================================

/// 认证服务兼容导出
pub mod auth {
    pub use crate::services::application::workers::auth::*;
}

/// 聊天服务兼容导出
pub mod chat {
    pub use crate::services::application::workers::chat::*;
}

/// 消息服务兼容导出
pub mod message {
    pub use crate::services::application::workers::message::*;
    pub mod models {
        // Deprecated models - use fechatter_core models instead
        // pub use crate::services::application::stores::models::*;
    }
}

/// 搜索服务兼容导出
pub mod search {
    pub use crate::services::application::workers::search::*;
}

/// 工作空间服务兼容导出
pub mod workspace {
    pub use crate::services::application::workers::workspace::{
        create_workspace_application_service, WorkspaceService,
    };
}

/// 缓存服务兼容导出
pub mod cache {
    pub use crate::services::application::stores::*;
}

/// 事件服务兼容导出
pub mod events {
    pub use crate::services::infrastructure::flows::*;
}

/// 消息传递服务兼容导出
pub mod messaging {
    pub use crate::services::infrastructure::flows::*;
}

/// 核心服务兼容导出
pub mod core {
    pub mod service_provider {
        // 需要从实际位置重新导出
        // TODO: 实现服务提供者
    }

    // ApplicationServiceProvider 别名
    pub type ApplicationServiceProvider = (); // TODO: 实现具体类型
}

pub mod events_prelude {
    // ... existing code ...

    // Deprecated models - use fechatter_core models instead
    // pub use crate::services::application::stores::models::*;
}
