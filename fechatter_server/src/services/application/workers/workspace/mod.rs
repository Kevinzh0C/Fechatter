//! # Workspace Application Service
//!
//! **Single Responsibility**: Orchestrate workspace use cases through domain service
//! **Architecture Role**: Thin application layer between handlers and domain

pub mod service;

// Simplified exports - no unnecessary abstractions
pub use service::{
  AddMembersCommand, InviteUserCommand, UpdateWorkspaceCommand, UserSummaryView, WorkspaceService,
  WorkspaceView, create_workspace_application_service,
};
