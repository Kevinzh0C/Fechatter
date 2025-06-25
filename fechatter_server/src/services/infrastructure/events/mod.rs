//! # DEPRECATED: events module
//!
//! **🚨 This module is DEPRECATED - use `event` module instead**
//!
//! **Migration:**
//! - `events::UnifiedEventPublisher` → `event::EnhancedEventPublisher`
//! - New publisher provides notify_server compatibility
//! - Complete message content delivery for SSE broadcasting

#[deprecated(
    since = "2.0.0", 
    note = "Use event::EnhancedEventPublisher instead for notify_server compatibility"
)]
pub mod unified_publisher;

// Re-export only what actually exists
#[deprecated(since = "2.0.0", note = "Use event::EnhancedEventPublisher instead")]
pub use unified_publisher::EventEnvelope; 