use async_trait::async_trait;
use std::sync::Arc;

use fechatter_core::{ChatUser, UserId, Workspace, WorkspaceId, error::CoreError};

use crate::handlers::workspaces::UpdateWorkspaceRequest;

use super::repository::WorkspaceRepositoryImpl;

// Define WorkspaceChatStats locally since it's not defined elsewhere
#[derive(Debug, Clone)]
pub struct WorkspaceChatStats {
  pub chat_id: i64,
  pub chat_name: String,
  pub message_count: i64,
  pub last_activity: Option<chrono::NaiveDateTime>,
}

/// Workspace configuration
#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
  pub max_name_length: usize,
  pub min_name_length: usize,
  pub max_members: usize,
  pub allow_duplicate_names: bool,
}

impl Default for WorkspaceConfig {
  fn default() -> Self {
    Self {
      max_name_length: 50,
      min_name_length: 2,
      max_members: 1000,
      allow_duplicate_names: false,
    }
  }
}

/// Workspace validation rules
pub struct WorkspaceValidationRules {
  config: WorkspaceConfig,
}

impl WorkspaceValidationRules {
  pub fn new(config: WorkspaceConfig) -> Self {
    Self { config }
  }

  /// Validate workspace name
  pub fn validate_name(&self, name: &str) -> Result<(), CoreError> {
    let trimmed_name = name.trim();

    if trimmed_name.is_empty() {
      return Err(CoreError::Validation(
        "Workspace name cannot be empty".to_string(),
      ));
    }

    if trimmed_name.len() < self.config.min_name_length {
      return Err(CoreError::Validation(format!(
        "Workspace name must be at least {} characters",
        self.config.min_name_length
      )));
    }

    if trimmed_name.len() > self.config.max_name_length {
      return Err(CoreError::Validation(format!(
        "Workspace name cannot exceed {} characters",
        self.config.max_name_length
      )));
    }

    // Check special characters
    if !trimmed_name
      .chars()
      .all(|c| c.is_alphanumeric() || c.is_whitespace() || "-_".contains(c))
    {
      return Err(CoreError::Validation(
        "Workspace name can only contain letters, numbers, spaces, hyphens, and underscores"
          .to_string(),
      ));
    }

    Ok(())
  }

  /// Validate user permissions
  pub fn validate_user_permissions(
    &self,
    user_id: UserId,
    workspace: &Workspace,
  ) -> Result<(), CoreError> {
    if workspace.owner_id != user_id {
      return Err(CoreError::Unauthorized(
        "Only workspace owner can perform this action".to_string(),
      ));
    }
    Ok(())
  }
}

/// Workspace aggregate
pub struct WorkspaceAggregate {
  pub workspace: Workspace,
  pub members: Vec<ChatUser>,
  pub chat_stats: Vec<WorkspaceChatStats>,
}

impl WorkspaceAggregate {
  /// Create new workspace aggregate
  pub fn new(workspace: Workspace) -> Self {
    Self {
      workspace,
      members: Vec::new(),
      chat_stats: Vec::new(),
    }
  }

  /// Add member information
  pub fn with_members(mut self, members: Vec<ChatUser>) -> Self {
    self.members = members;
    self
  }

  /// Add chat statistics
  pub fn with_chat_stats(mut self, chat_stats: Vec<WorkspaceChatStats>) -> Self {
    self.chat_stats = chat_stats;
    self
  }

  /// Get active member count
  pub fn active_member_count(&self) -> usize {
    // Since ChatUser doesn't have is_active field, count all members
    self.members.len()
  }

  /// Get total chat count
  pub fn total_chat_count(&self) -> usize {
    self.chat_stats.len()
  }

  /// Get active chat count (messages in last 7 days)
  pub fn active_chat_count(&self) -> usize {
    use chrono::{Duration, Utc};
    let week_ago = Utc::now() - Duration::days(7);

    self
      .chat_stats
      .iter()
      .filter(|stat| {
        stat
          .last_activity
          .map(|activity| activity > week_ago.naive_utc())
          .unwrap_or(false)
      })
      .count()
  }
}

/// Workspace Domain Service trait
#[async_trait]
pub trait WorkspaceDomainService: Send + Sync {
  async fn create_workspace(&self, name: &str, owner_id: UserId) -> Result<Workspace, CoreError>;
  async fn update_workspace(
    &self,
    workspace_id: WorkspaceId,
    request: &UpdateWorkspaceRequest,
    user_id: UserId,
  ) -> Result<Workspace, CoreError>;
  async fn get_workspace_aggregate(
    &self,
    workspace_id: WorkspaceId,
  ) -> Result<WorkspaceAggregate, CoreError>;
  async fn add_member_to_workspace(
    &self,
    workspace_id: WorkspaceId,
    user_id: UserId,
    admin_user_id: UserId,
  ) -> Result<Workspace, CoreError>;
  async fn transfer_ownership(
    &self,
    workspace_id: WorkspaceId,
    new_owner_id: UserId,
    current_owner_id: UserId,
  ) -> Result<Workspace, CoreError>;
  async fn get_workspace_stats(
    &self,
    workspace_id: WorkspaceId,
  ) -> Result<Vec<WorkspaceChatStats>, CoreError>;
}

/// Workspace Domain Service implementation
pub struct WorkspaceDomainServiceImpl {
  repository: Arc<WorkspaceRepositoryImpl>,
  validator: WorkspaceValidationRules,
}

impl WorkspaceDomainServiceImpl {
  pub fn new(repository: Arc<WorkspaceRepositoryImpl>, config: WorkspaceConfig) -> Self {
    Self {
      repository,
      validator: WorkspaceValidationRules::new(config),
    }
  }
}

#[async_trait]
impl WorkspaceDomainService for WorkspaceDomainServiceImpl {
  async fn create_workspace(&self, name: &str, owner_id: UserId) -> Result<Workspace, CoreError> {
    // Validate name
    self.validator.validate_name(name)?;

    // Check name uniqueness (if required)
    if !self.validator.config.allow_duplicate_names {
      if let Some(_existing) = self.repository.find_by_name(name).await? {
        return Err(CoreError::Validation(
          "Workspace name already exists".to_string(),
        ));
      }
    }

    // Create workspace using find_or_create_by_name method
    let mut workspace = self.repository.find_or_create_by_name(name).await?;

    // Update owner if necessary (new workspaces have owner_id = 0)
    if workspace.owner_id == UserId(0) || workspace.owner_id != owner_id {
      workspace = self.repository.update_owner(workspace.id, owner_id).await?;
    }

    Ok(workspace)
  }

  async fn update_workspace(
    &self,
    workspace_id: WorkspaceId,
    request: &UpdateWorkspaceRequest,
    user_id: UserId,
  ) -> Result<Workspace, CoreError> {
    // Verify workspace exists
    let workspace = self
      .repository
      .find_by_id(workspace_id)
      .await?
      .ok_or_else(|| CoreError::NotFound("Workspace not found".to_string()))?;

    // Validate user permissions
    self
      .validator
      .validate_user_permissions(user_id, &workspace)?;

    // Check if name is provided
    if let Some(ref new_name) = request.name {
      // Validate new name
      self.validator.validate_name(new_name)?;

      // Check name uniqueness (if required and name changed)
      if !self.validator.config.allow_duplicate_names && workspace.name != *new_name {
        if let Some(_existing) = self.repository.find_by_name(new_name).await? {
          return Err(CoreError::Validation(
            "Workspace name already exists".to_string(),
          ));
        }
      }

      // Since repository doesn't have update_workspace method,
      // we'll need to implement this differently or add the method
      // For now, return an error indicating this needs implementation
      return Err(CoreError::Unimplemented(
        "Workspace name update not yet implemented in repository".to_string(),
      ));
    }

    // If no name provided, just return the existing workspace
    Ok(workspace)
  }

  async fn get_workspace_aggregate(
    &self,
    workspace_id: WorkspaceId,
  ) -> Result<WorkspaceAggregate, CoreError> {
    // Get workspace basic info
    let workspace = self
      .repository
      .find_by_id(workspace_id)
      .await?
      .ok_or_else(|| CoreError::NotFound("Workspace not found".to_string()))?;

    let aggregate = WorkspaceAggregate::new(workspace);

    // Note: WorkspaceRepositoryImpl doesn't have methods for fetching members or stats
    // These would need to be implemented in the repository layer
    // For now, return aggregate with empty members and stats

    Ok(aggregate)
  }

  async fn add_member_to_workspace(
    &self,
    workspace_id: WorkspaceId,
    _user_id: UserId,
    admin_user_id: UserId,
  ) -> Result<Workspace, CoreError> {
    // Verify workspace exists
    let workspace = self
      .repository
      .find_by_id(workspace_id)
      .await?
      .ok_or_else(|| CoreError::NotFound("Workspace not found".to_string()))?;

    // Validate admin permissions
    self
      .validator
      .validate_user_permissions(admin_user_id, &workspace)?;

    // WorkspaceRepositoryImpl doesn't have add_user_to_workspace method
    // This functionality needs to be implemented in the repository layer
    Err(CoreError::Unimplemented(
      "Adding users to workspace not yet implemented in repository".to_string(),
    ))
  }

  async fn transfer_ownership(
    &self,
    workspace_id: WorkspaceId,
    new_owner_id: UserId,
    current_owner_id: UserId,
  ) -> Result<Workspace, CoreError> {
    // Verify workspace exists
    let workspace = self
      .repository
      .find_by_id(workspace_id)
      .await?
      .ok_or_else(|| CoreError::NotFound("Workspace not found".to_string()))?;

    // Validate current user permissions
    self
      .validator
      .validate_user_permissions(current_owner_id, &workspace)?;

    // Transfer ownership
    self
      .repository
      .update_owner(workspace_id, new_owner_id)
      .await
  }

  async fn get_workspace_stats(
    &self,
    workspace_id: WorkspaceId,
  ) -> Result<Vec<WorkspaceChatStats>, CoreError> {
    // Verify workspace exists
    self
      .repository
      .find_by_id(workspace_id)
      .await?
      .ok_or_else(|| CoreError::NotFound("Workspace not found".to_string()))?;

    // WorkspaceRepositoryImpl doesn't have get_workspace_chat_stats method
    // For now, return empty stats
    Ok(Vec::new())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;

  fn create_test_workspace() -> Workspace {
    Workspace {
      id: WorkspaceId(1),
      name: "Test Workspace".to_string(),
      owner_id: UserId(1),
      created_at: Utc::now().naive_utc(),
    }
  }

  #[tokio::test]
  async fn validate_name_should_enforce_length_limits() {
    let config = WorkspaceConfig::default();
    let validator = WorkspaceValidationRules::new(config);

    // Test minimum length
    let short_name = "a"; // 1 char, below min of 2
    let result = validator.validate_name(short_name);
    assert!(result.is_err());
    assert!(
      result
        .unwrap_err()
        .to_string()
        .contains("at least 2 characters")
    );

    // Test valid length
    let valid_name = "ab"; // 2 chars, exactly min
    assert!(validator.validate_name(valid_name).is_ok());

    // Test maximum length
    let long_name = "a".repeat(51); // 51 chars, above max of 50
    let result = validator.validate_name(&long_name);
    assert!(result.is_err());
    assert!(
      result
        .unwrap_err()
        .to_string()
        .contains("cannot exceed 50 characters")
    );
  }

  #[tokio::test]
  async fn validate_name_should_handle_whitespace() {
    let config = WorkspaceConfig::default();
    let validator = WorkspaceValidationRules::new(config);

    // Empty name should fail
    let result = validator.validate_name("");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));

    // Whitespace only should fail
    let result = validator.validate_name("   ");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));

    // Leading/trailing whitespace should be handled (trimmed)
    assert!(validator.validate_name("  valid name  ").is_ok());

    // Test minimum after trimming
    let result = validator.validate_name(" a "); // Trims to "a" (1 char)
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn validate_name_should_check_special_characters() {
    let config = WorkspaceConfig::default();
    let validator = WorkspaceValidationRules::new(config);

    // Valid characters should pass
    assert!(validator.validate_name("Valid Name 123").is_ok());
    assert!(validator.validate_name("Test-Workspace_1").is_ok());

    // Invalid special characters should fail
    let invalid_names = vec![
      "Name@Domain",   // @ symbol
      "Name#Tag",      // # symbol
      "Name$Value",    // $ symbol
      "Name%Complete", // % symbol
      "Name&Company",  // & symbol
      "Name*Star",     // * symbol
      "Name+Plus",     // + symbol
      "Name=Equal",    // = symbol
    ];

    for invalid_name in invalid_names {
      let result = validator.validate_name(invalid_name);
      assert!(result.is_err(), "Name '{}' should be invalid", invalid_name);
      assert!(result.unwrap_err().to_string().contains("can only contain"));
    }
  }

  #[tokio::test]
  async fn validate_name_with_custom_config() {
    let config = WorkspaceConfig {
      min_name_length: 5,
      max_name_length: 20,
      ..Default::default()
    };
    let validator = WorkspaceValidationRules::new(config);

    // Test below custom minimum
    let result = validator.validate_name("Test"); // 4 chars, below min of 5
    assert!(result.is_err());
    assert!(
      result
        .unwrap_err()
        .to_string()
        .contains("at least 5 characters")
    );

    // Test valid with custom config
    assert!(validator.validate_name("Valid").is_ok()); // 5 chars, exactly min

    // Test above custom maximum
    let long_name = "a".repeat(21); // 21 chars, above max of 20
    let result = validator.validate_name(&long_name);
    assert!(result.is_err());
    assert!(
      result
        .unwrap_err()
        .to_string()
        .contains("cannot exceed 20 characters")
    );
  }

  #[tokio::test]
  async fn validate_user_permissions_should_check_ownership() {
    let config = WorkspaceConfig::default();
    let validator = WorkspaceValidationRules::new(config);
    let workspace = create_test_workspace();

    // Owner should have permissions
    let owner_id = workspace.owner_id;
    assert!(
      validator
        .validate_user_permissions(owner_id, &workspace)
        .is_ok()
    );

    // Non-owner should not have permissions
    let non_owner_id = UserId(999);
    let result = validator.validate_user_permissions(non_owner_id, &workspace);
    assert!(result.is_err());
    assert!(
      result
        .unwrap_err()
        .to_string()
        .contains("Only workspace owner")
    );
  }

  #[tokio::test]
  async fn workspace_config_should_have_reasonable_defaults() {
    let config = WorkspaceConfig::default();

    assert_eq!(config.max_name_length, 50);
    assert_eq!(config.min_name_length, 2);
    assert_eq!(config.max_members, 1000);
    assert!(!config.allow_duplicate_names);
  }

  #[tokio::test]
  async fn workspace_aggregate_should_initialize_correctly() {
    let workspace = create_test_workspace();
    let aggregate = WorkspaceAggregate::new(workspace.clone());

    assert_eq!(aggregate.workspace.id, workspace.id);
    assert_eq!(aggregate.workspace.name, workspace.name);
    assert!(aggregate.members.is_empty());
    assert!(aggregate.chat_stats.is_empty());
  }

  #[tokio::test]
  async fn workspace_aggregate_should_calculate_active_member_count() {
    let workspace = create_test_workspace();
    let mut aggregate = WorkspaceAggregate::new(workspace);

    // No members initially
    assert_eq!(aggregate.active_member_count(), 0);

    // Add mock members (simplified ChatUser without avatar and is_active fields)
    let active_user = fechatter_core::ChatUser {
      id: UserId(1),
      email: "active@test.com".to_string(),
      fullname: "Active User".to_string(),
    };

    let inactive_user = fechatter_core::ChatUser {
      id: UserId(2),
      email: "inactive@test.com".to_string(),
      fullname: "Inactive User".to_string(),
    };

    aggregate = aggregate.with_members(vec![active_user, inactive_user]);
    assert_eq!(aggregate.active_member_count(), 2); // Both users counted (no is_active field)
  }

  #[tokio::test]
  async fn workspace_aggregate_should_calculate_chat_counts() {
    let workspace = create_test_workspace();
    let aggregate = WorkspaceAggregate::new(workspace);

    // No chats initially
    assert_eq!(aggregate.total_chat_count(), 0);
    assert_eq!(aggregate.active_chat_count(), 0);

    // Note: Testing with real WorkspaceChatStats would require more complex setup
    // This test verifies the counting logic exists
  }
}
