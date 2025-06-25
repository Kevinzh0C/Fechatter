//! # Permission System - Centralized Authorization
//!
//! **Responsibility**: Define permission types and provide permission checking logic
//! **Principles**: Clear permission hierarchy, easy to extend

/// Permission enum - Defines all possible permissions in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
  // ========================================================================
  // Workspace Permissions - Increasing by level
  // ========================================================================
  /// Workspace Guest - Read-only permissions
  WorkspaceGuest,
  /// Workspace Member - Basic permissions (create chats, participate in discussions)
  WorkspaceMember,
  /// Workspace Moderator - Mid-level permissions (manage content, mute users)
  WorkspaceModerator,
  /// Workspace Admin - High-level permissions (manage users, settings, invites)
  WorkspaceAdmin,
  /// Workspace Owner - Highest permissions (delete workspace, transfer ownership)
  WorkspaceOwner,

  // ========================================================================
  // Chat Permissions - Divided by functionality
  // ========================================================================
  /// Chat Observer - Can only view messages, cannot send
  ChatObserver,
  /// Chat Member - Can view and send messages
  ChatMember,
  /// Chat Moderator - Can delete messages, mute members
  ChatModerator,
  /// Chat Admin - Can manage members and settings
  ChatAdmin,
  /// Chat Creator - Original creator of the chat
  ChatCreator,

  // ========================================================================
  // Message Permissions - Fine-grained control
  // ========================================================================
  /// Message Owner - Can edit and delete own messages
  MessageOwner,
  /// Message Edit - Can edit message content
  MessageEdit,
  /// Message Delete - Can delete messages
  MessageDelete,
  /// Message Pin - Can pin/unpin messages
  MessagePin,
  /// Message React - Can add emoji reactions
  MessageReact,
  /// Message Forward - Can forward messages to other chats
  MessageForward,

  // ========================================================================
  // File Permissions - File operation control
  // ========================================================================
  /// File Upload - Can upload files
  FileUpload,
  /// File Download - Can download files
  FileDownload,
  /// File Delete - Can delete files
  FileDelete,
  /// File Share - Can share file links
  FileShare,

  // ========================================================================
  // Search Permissions - Search functionality control
  // ========================================================================
  /// Global Search - Can search across entire workspace
  GlobalSearch,
  /// Chat Search - Can search within specific chats
  ChatSearch,
  /// User Search - Can search users
  UserSearch,
  /// Search Suggestions - Can get search suggestions
  SearchSuggestions,

  // ========================================================================
  // Invite Permissions - User invitation control
  // ========================================================================
  /// Invite users to workspace
  InviteToWorkspace,
  /// Invite users to chat
  InviteToChat,
  /// Create invite links
  CreateInviteLink,
  /// Manage invite links
  ManageInviteLinks,

  // ========================================================================
  // Monitoring Permissions - System monitoring functionality
  // ========================================================================
  /// View system status
  ViewSystemStatus,
  /// View cache statistics
  ViewCacheStats,
  /// Clear cache
  ClearCache,
  /// View online users
  ViewOnlineUsers,
  /// Rebuild search index
  RebuildSearchIndex,

  // ========================================================================
  // System Permissions - Highest level permissions
  // ========================================================================
  /// System Admin - System level management permissions
  SystemAdmin,
  /// System Maintenance - System maintenance permissions (backup, restore etc)
  SystemMaintenance,
  /// Audit View - View system audit logs
  AuditView,
}

impl Permission {
  /// Get permission display name
  pub fn display_name(&self) -> &'static str {
    match self {
      // Workspace permissions
      Permission::WorkspaceGuest => "Workspace Guest",
      Permission::WorkspaceMember => "Workspace Member",
      Permission::WorkspaceModerator => "Workspace Moderator",
      Permission::WorkspaceAdmin => "Workspace Admin",
      Permission::WorkspaceOwner => "Workspace Owner",

      // Chat permissions
      Permission::ChatObserver => "Chat Observer",
      Permission::ChatMember => "Chat Member",
      Permission::ChatModerator => "Chat Moderator",
      Permission::ChatAdmin => "Chat Admin",
      Permission::ChatCreator => "Chat Creator",

      // Message permissions
      Permission::MessageOwner => "Message Owner",
      Permission::MessageEdit => "Message Edit",
      Permission::MessageDelete => "Message Delete",
      Permission::MessagePin => "Message Pin",
      Permission::MessageReact => "Message React",
      Permission::MessageForward => "Message Forward",

      // File permissions
      Permission::FileUpload => "File Upload",
      Permission::FileDownload => "File Download",
      Permission::FileDelete => "File Delete",
      Permission::FileShare => "File Share",

      // Search permissions
      Permission::GlobalSearch => "Global Search",
      Permission::ChatSearch => "Chat Search",
      Permission::UserSearch => "User Search",
      Permission::SearchSuggestions => "Search Suggestions",

      // Invite permissions
      Permission::InviteToWorkspace => "Invite to Workspace",
      Permission::InviteToChat => "Invite to Chat",
      Permission::CreateInviteLink => "Create Invite Link",
      Permission::ManageInviteLinks => "Manage Invite Links",

      // Monitoring permissions
      Permission::ViewSystemStatus => "View System Status",
      Permission::ViewCacheStats => "View Cache Stats",
      Permission::ClearCache => "Clear Cache",
      Permission::ViewOnlineUsers => "View Online Users",
      Permission::RebuildSearchIndex => "Rebuild Search Index",

      // System permissions
      Permission::SystemAdmin => "System Admin",
      Permission::SystemMaintenance => "System Maintenance",
      Permission::AuditView => "Audit View",
    }
  }

  /// Get permission description
  pub fn description(&self) -> &'static str {
    match self {
      // Workspace permissions
      Permission::WorkspaceGuest => "Can view basic workspace information",
      Permission::WorkspaceMember => "Can access basic workspace features",
      Permission::WorkspaceModerator => "Can manage content and moderator operations",
      Permission::WorkspaceAdmin => "Can manage workspace users and settings",
      Permission::WorkspaceOwner => "Has complete control over workspace",

      // Chat permissions
      Permission::ChatObserver => "Can view chat messages, cannot send",
      Permission::ChatMember => "Can view and send chat messages",
      Permission::ChatModerator => "Can manage chat content and members",
      Permission::ChatAdmin => "Can manage chat members and settings",
      Permission::ChatCreator => "Original chat creator with special privileges",

      // Message permissions
      Permission::MessageOwner => "Can edit and delete own messages",
      Permission::MessageEdit => "Can edit message content",
      Permission::MessageDelete => "Can delete messages",
      Permission::MessagePin => "Can pin and unpin messages",
      Permission::MessageReact => "Can add reactions to messages",
      Permission::MessageForward => "Can forward messages to other chats",

      // File permissions
      Permission::FileUpload => "Can upload files to chat",
      Permission::FileDownload => "Can download files",
      Permission::FileDelete => "Can delete files",
      Permission::FileShare => "Can share file links",

      // Search permissions
      Permission::GlobalSearch => "Can search across entire workspace",
      Permission::ChatSearch => "Can search within specific chats",
      Permission::UserSearch => "Can search users",
      Permission::SearchSuggestions => "Can get search suggestions",

      // Invite permissions
      Permission::InviteToWorkspace => "Can invite users to workspace",
      Permission::InviteToChat => "Can invite users to chat",
      Permission::CreateInviteLink => "Can create invite links",
      Permission::ManageInviteLinks => "Can manage invite links",

      // Monitoring permissions
      Permission::ViewSystemStatus => "Can view system status",
      Permission::ViewCacheStats => "Can view cache statistics",
      Permission::ClearCache => "Can clear cache",
      Permission::ViewOnlineUsers => "Can view online users",
      Permission::RebuildSearchIndex => "Can rebuild search index",

      // System permissions
      Permission::SystemAdmin => "Has system level management permissions",
      Permission::SystemMaintenance => "Can perform system maintenance operations",
      Permission::AuditView => "Can view system audit logs",
    }
  }

  /// Check if permission includes another permission (permission inheritance)
  pub fn includes(&self, other: Permission) -> bool {
    match (self, other) {
      // ====================================================================
      // Workspace permission hierarchy inheritance
      // ====================================================================
      (Permission::WorkspaceOwner, Permission::WorkspaceAdmin) => true,
      (Permission::WorkspaceOwner, Permission::WorkspaceModerator) => true,
      (Permission::WorkspaceOwner, Permission::WorkspaceMember) => true,
      (Permission::WorkspaceOwner, Permission::WorkspaceGuest) => true,

      (Permission::WorkspaceAdmin, Permission::WorkspaceModerator) => true,
      (Permission::WorkspaceAdmin, Permission::WorkspaceMember) => true,
      (Permission::WorkspaceAdmin, Permission::WorkspaceGuest) => true,

      (Permission::WorkspaceModerator, Permission::WorkspaceMember) => true,
      (Permission::WorkspaceModerator, Permission::WorkspaceGuest) => true,

      (Permission::WorkspaceMember, Permission::WorkspaceGuest) => true,

      // ====================================================================
      // Chat permission hierarchy inheritance
      // ====================================================================
      (Permission::ChatCreator, Permission::ChatAdmin) => true,
      (Permission::ChatCreator, Permission::ChatModerator) => true,
      (Permission::ChatCreator, Permission::ChatMember) => true,
      (Permission::ChatCreator, Permission::ChatObserver) => true,

      (Permission::ChatAdmin, Permission::ChatModerator) => true,
      (Permission::ChatAdmin, Permission::ChatMember) => true,
      (Permission::ChatAdmin, Permission::ChatObserver) => true,

      (Permission::ChatModerator, Permission::ChatMember) => true,
      (Permission::ChatModerator, Permission::ChatObserver) => true,

      (Permission::ChatMember, Permission::ChatObserver) => true,

      // ====================================================================
      // Message permission inheritance
      // ====================================================================
      (Permission::MessageOwner, Permission::MessageEdit) => true,
      (Permission::MessageOwner, Permission::MessageDelete) => true,
      (Permission::MessageOwner, Permission::MessageReact) => true,
      (Permission::MessageOwner, Permission::MessageForward) => true,

      (Permission::ChatAdmin, Permission::MessageDelete) => true,
      (Permission::ChatAdmin, Permission::MessagePin) => true,

      (Permission::ChatModerator, Permission::MessageDelete) => true,

      // ====================================================================
      // File permission inheritance
      // ====================================================================
      (Permission::WorkspaceAdmin, Permission::FileDelete) => true,
      (Permission::WorkspaceMember, Permission::FileUpload) => true,
      (Permission::WorkspaceMember, Permission::FileDownload) => true,
      (Permission::WorkspaceMember, Permission::FileShare) => true,

      // ====================================================================
      // Search permission inheritance
      // ====================================================================
      (Permission::WorkspaceMember, Permission::GlobalSearch) => true,
      (Permission::WorkspaceMember, Permission::UserSearch) => true,
      (Permission::WorkspaceMember, Permission::SearchSuggestions) => true,
      (Permission::ChatMember, Permission::ChatSearch) => true,

      // ====================================================================
      // Invite permission inheritance
      // ====================================================================
      (Permission::WorkspaceAdmin, Permission::InviteToWorkspace) => true,
      (Permission::WorkspaceAdmin, Permission::ManageInviteLinks) => true,
      (Permission::WorkspaceModerator, Permission::CreateInviteLink) => true,
      (Permission::ChatAdmin, Permission::InviteToChat) => true,

      // ====================================================================
      // Monitoring permission inheritance
      // ====================================================================
      (Permission::SystemAdmin, Permission::ViewSystemStatus) => true,
      (Permission::SystemAdmin, Permission::ViewCacheStats) => true,
      (Permission::SystemAdmin, Permission::ClearCache) => true,
      (Permission::SystemAdmin, Permission::ViewOnlineUsers) => true,
      (Permission::SystemAdmin, Permission::RebuildSearchIndex) => true,
      (Permission::SystemAdmin, Permission::AuditView) => true,

      (Permission::WorkspaceAdmin, Permission::ViewCacheStats) => true,
      (Permission::WorkspaceAdmin, Permission::ViewOnlineUsers) => true,

      // ====================================================================
      // Cross-domain permission inheritance
      // ====================================================================
      // Workspace admin inherits basic chat permissions
      (Permission::WorkspaceAdmin, Permission::ChatMember) => true,
      (Permission::WorkspaceAdmin, Permission::ChatObserver) => true,

      // Workspace moderator inherits basic message permissions
      (Permission::WorkspaceModerator, Permission::MessageDelete) => true,

      // ====================================================================
      // System level permissions
      // ====================================================================
      // System admin has all permissions
      (Permission::SystemAdmin, _) => true,

      // System maintenance permissions
      (Permission::SystemMaintenance, Permission::ViewSystemStatus) => true,
      (Permission::SystemMaintenance, Permission::ClearCache) => true,

      // ====================================================================
      // Same permissions
      // ====================================================================
      (a, b) if *a == b => true,

      // Other cases do not include
      _ => false,
    }
  }

  /// Get all workspace related permissions
  pub fn workspace_permissions() -> Vec<Permission> {
    vec![
      Permission::WorkspaceMember,
      Permission::WorkspaceAdmin,
      Permission::WorkspaceOwner,
    ]
  }

  /// Get all chat related permissions
  pub fn chat_permissions() -> Vec<Permission> {
    vec![
      Permission::ChatMember,
      Permission::ChatAdmin,
      Permission::ChatCreator,
    ]
  }

  /// Get all message related permissions
  pub fn message_permissions() -> Vec<Permission> {
    vec![
      Permission::MessageOwner,
      Permission::MessageEdit,
      Permission::MessageDelete,
    ]
  }
}

/// Permission Checker - Provides permission validation logic
pub struct PermissionChecker;

impl PermissionChecker {
  /// Check if user has specific permission
  pub fn has_permission(user_permissions: &[Permission], required: Permission) -> bool {
    user_permissions.iter().any(|p| p.includes(required))
  }

  /// Check if user has any of the required permissions
  pub fn has_any_permission(user_permissions: &[Permission], required: &[Permission]) -> bool {
    required
      .iter()
      .any(|req| Self::has_permission(user_permissions, *req))
  }

  /// Check if user has all required permissions
  pub fn has_all_permissions(user_permissions: &[Permission], required: &[Permission]) -> bool {
    required
      .iter()
      .all(|req| Self::has_permission(user_permissions, *req))
  }
}
