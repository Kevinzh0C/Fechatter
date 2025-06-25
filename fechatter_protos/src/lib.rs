// Fechatter Protocol Buffers
// 
// This crate handles protobuf compilation for both local and cross-compilation environments.
// For cross-compilation, it uses precompiled protobuf files to avoid protoc dependency issues.

// 重新导出 tonic 和 prost 类型
pub use prost;
pub use prost_types;
pub use tonic;

// 包含生成的协议代码
pub mod fechatter {
  pub mod v1 {
    // 条件编译：交叉编译时使用预编译文件，否则使用tonic生成的文件
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    include!("precompiled/fechatter.v1.rs");
    
    #[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
    tonic::include_proto!("fechatter.v1");

    // 重新导出服务相关类型
    pub use self::{
      // Analytics服务
      analytics_service_client::AnalyticsServiceClient,
      analytics_service_server::{AnalyticsService, AnalyticsServiceServer},
      
      // Auth服务
      auth_service_client::AuthServiceClient,
      auth_service_server::{AuthService, AuthServiceServer},
      
      // Bot服务
      bot_service_client::BotServiceClient,
      bot_service_server::{BotService, BotServiceServer},
      
      // Chat服务
      chat_service_client::ChatServiceClient,
      chat_service_server::{ChatService, ChatServiceServer},
      
      // Code Index服务
      code_index_service_client::CodeIndexServiceClient,
      code_index_service_server::{CodeIndexService, CodeIndexServiceServer},
      
      // File服务
      file_service_client::FileServiceClient,
      file_service_server::{FileService, FileServiceServer},
      
      // Message服务
      message_service_client::MessageServiceClient,
      message_service_server::{MessageService, MessageServiceServer},
      
      // Message Stream服务
      message_stream_service_client::MessageStreamServiceClient,
      message_stream_service_server::{MessageStreamService, MessageStreamServiceServer},
      
      // Notification服务
      notification_service_client::NotificationServiceClient,
      notification_service_server::{NotificationService, NotificationServiceServer},
      
      // User服务
      user_service_client::UserServiceClient,
      user_service_server::{UserService, UserServiceServer},
    };
  }
}

// 重新导出常用类型，方便使用
pub use fechatter::v1::*;
