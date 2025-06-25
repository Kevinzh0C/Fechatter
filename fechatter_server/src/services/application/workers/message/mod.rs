//! # Message Worker Module
//!
//! **Responsibilities**: Message processing work unit
//! **Principle**: Use unified models from fechatter_core

pub mod consistency_monitor;
mod service;

// Re-export service components
pub use service::{
  AppStateEventPublisher, AsyncIndexEvent, DualStreamDispatcher, DualStreamMessageService,
  IndexOperation, MessageApplicationService, RealtimeEvent, create_message_service,
};

// Re-export models from fechatter_core for backward compatibility
pub use fechatter_core::models::message::MessageView;

pub use consistency_monitor::{
  ConsistencyCheckResult, MessageConsistencyMonitor, MessageConsistencyMonitorImpl,
  create_consistency_monitor,
};
