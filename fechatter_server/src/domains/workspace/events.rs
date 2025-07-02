use fechatter_core::{UserId, WorkspaceId};
use serde::{Deserialize, Serialize};

/// Workspace domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceDomainEvent {
    /// Workspace created
    WorkspaceCreated {
        workspace_id: WorkspaceId,
        owner_id: UserId,
        name: String,
    },
    /// User joined workspace
    UserJoinedWorkspace {
        workspace_id: WorkspaceId,
        user_id: UserId,
        invited_by: Option<UserId>,
    },
    /// User left workspace
    UserLeftWorkspace {
        workspace_id: WorkspaceId,
        user_id: UserId,
        reason: String,
    },
    /// Workspace ownership transferred
    OwnershipTransferred {
        workspace_id: WorkspaceId,
        old_owner_id: UserId,
        new_owner_id: UserId,
    },
    /// Workspace settings updated
    WorkspaceSettingsUpdated {
        workspace_id: WorkspaceId,
        updated_by: UserId,
        changes: WorkspaceChanges,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceChanges {
    pub name: Option<String>,
    pub description: Option<String>,
    pub settings: Option<String>,
}
