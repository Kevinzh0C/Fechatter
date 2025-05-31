use fechatter_core::{ChatId, UserId, WorkspaceId};
use serde::{Deserialize, Serialize};

/// Chat domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatDomainEvent {
  /// Chat created
  ChatCreated {
    chat_id: ChatId,
    creator_id: UserId,
    workspace_id: WorkspaceId,
    chat_type: String,
    initial_members: Vec<UserId>,
  },
  /// Chat updated
  ChatUpdated {
    chat_id: ChatId,
    updated_by: UserId,
    changes: ChatChanges,
  },
  /// Chat deleted
  ChatDeleted { chat_id: ChatId, deleted_by: UserId },
  /// Member added
  MemberAdded {
    chat_id: ChatId,
    added_by: UserId,
    new_member: UserId,
  },
  /// Member removed
  MemberRemoved {
    chat_id: ChatId,
    removed_by: UserId,
    removed_member: UserId,
  },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChanges {
  pub name: Option<String>,
  pub description: Option<String>,
}
