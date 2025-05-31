//! # Workspace Application Service - Revolutionary Architecture
//!
//! **Responsibility**: Workspace use case orchestration and business logic
//! **Architecture**: Application Service Layer with Advanced Dependency Injection
//! **Design Principles**:
//! - Builder + Factory Pattern for Clean Service Creation
//! - Optimized Resource Management without Arc Proliferation  
//! - Unified Error Handling and Validation
//! - Clear Separation of Concerns

use async_trait::async_trait;
use fechatter_core::contracts::UserRepository;
use fechatter_core::{UserId, WorkspaceId};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

use crate::domains::user::repository::UserRepositoryImpl;
use crate::domains::workspace::repository::WorkspaceRepositoryImpl;
use crate::{AppError, AppState};

// ============================================================================
// OPTIMIZED SERVICE BUILDER PATTERN
// ============================================================================

/// Workspace Service Builder - Revolutionary dependency injection without Arc proliferation
pub struct WorkspaceServiceBuilder {
  workspace_repository: Option<Arc<WorkspaceRepositoryImpl>>,
  user_repository: Option<Arc<UserRepositoryImpl>>,
  app_state: Option<AppState>,
}

impl WorkspaceServiceBuilder {
  pub fn new() -> Self {
    Self {
      workspace_repository: None,
      user_repository: None,
      app_state: None,
    }
  }

  /// Inject optimized repositories from state
  pub fn with_repositories_from_state(mut self, state: &AppState) -> Self {
    // Optimized: Create Arc<PgPool> once, share across repositories
    let pool = Arc::new(state.pool().clone());

    self.workspace_repository = Some(Arc::new(WorkspaceRepositoryImpl::new(pool.clone())));
    self.user_repository = Some(Arc::new(UserRepositoryImpl::new(pool)));
    self.app_state = Some(state.clone());
    self
  }

  /// Build the complete service with validation
  pub fn build(self) -> Result<WorkspaceApplicationService, AppError> {
    let workspace_repository = self
      .workspace_repository
      .ok_or_else(|| AppError::InvalidInput("WorkspaceRepository is required".to_string()))?;

    let user_repository = self
      .user_repository
      .ok_or_else(|| AppError::InvalidInput("UserRepository is required".to_string()))?;

    let app_state = self
      .app_state
      .ok_or_else(|| AppError::InvalidInput("AppState is required".to_string()))?;

    Ok(WorkspaceApplicationService::new(
      workspace_repository,
      user_repository,
      app_state,
    ))
  }
}

// ============================================================================
// OPTIMIZED FACTORY PATTERN
// ============================================================================

/// Workspace Service Factory with Advanced Resource Management
pub struct WorkspaceServiceFactory;

impl WorkspaceServiceFactory {
  /// Create service with optimized resource sharing
  pub fn create_from_state(state: &AppState) -> Result<WorkspaceApplicationService, AppError> {
    WorkspaceServiceBuilder::new()
      .with_repositories_from_state(state)
      .build()
  }
}

// ============================================================================
// VIEW MODELS - Response Data Structures
// ============================================================================

#[derive(Serialize, Clone, Debug)]
pub struct UserSummaryView {
  pub id: i64,
  pub fullname: String,
  pub email: String,
  pub status: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct WorkspaceView {
  pub id: i64,
  pub name: String,
  pub owner_id: i64,
  pub member_count: i32,
  pub created_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// COMMAND MODELS - Input Data Structures
// ============================================================================

#[derive(Deserialize, Clone, Debug)]
pub struct UpdateWorkspaceCommand {
  pub name: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct InviteUserCommand {
  pub email: String,
}

// ============================================================================
// APPLICATION SERVICE TRAIT
// ============================================================================

#[async_trait]
pub trait WorkspaceApplicationServiceTrait: Send + Sync {
  /// Get workspace user list
  async fn list_workspace_users(
    &self,
    workspace_id: i64,
    user_id: i64,
  ) -> Result<Vec<UserSummaryView>, AppError>;

  /// Update workspace information
  async fn update_workspace(
    &self,
    workspace_id: i64,
    user_id: i64,
    command: UpdateWorkspaceCommand,
  ) -> Result<WorkspaceView, AppError>;

  /// Invite user to workspace
  async fn invite_user(
    &self,
    workspace_id: i64,
    inviter_id: i64,
    command: InviteUserCommand,
  ) -> Result<(), AppError>;

  /// Get workspace details
  async fn get_workspace_details(
    &self,
    workspace_id: i64,
    user_id: i64,
  ) -> Result<WorkspaceView, AppError>;

  /// Validate workspace access
  async fn validate_workspace_access(
    &self,
    user_id: i64,
    workspace_id: i64,
  ) -> Result<(), AppError>;

  /// Validate workspace owner permissions
  async fn validate_workspace_owner(&self, user_id: i64, workspace_id: i64)
  -> Result<(), AppError>;
}

// ============================================================================
// OPTIMIZED APPLICATION SERVICE IMPLEMENTATION
// ============================================================================

/// Workspace Application Service - Clean, optimized implementation
pub struct WorkspaceApplicationService {
  /// Workspace Repository - Optimized with shared pool
  workspace_repository: Arc<WorkspaceRepositoryImpl>,
  /// User Repository - Optimized with shared pool  
  user_repository: Arc<UserRepositoryImpl>,
  /// App State - For direct SQL queries when needed
  app_state: AppState,
}

impl WorkspaceApplicationService {
  /// Create a new workspace application service
  pub fn new(
    workspace_repository: Arc<WorkspaceRepositoryImpl>,
    user_repository: Arc<UserRepositoryImpl>,
    app_state: AppState,
  ) -> Self {
    Self {
      workspace_repository,
      user_repository,
      app_state,
    }
  }

  /// DEPRECATED: Use WorkspaceServiceFactory::create_from_state instead
  #[deprecated(
    note = "Use WorkspaceServiceFactory::create_from_state for better resource management"
  )]
  pub fn from_app_state(app_state: AppState) -> Self {
    WorkspaceServiceFactory::create_from_state(&app_state)
      .expect("Failed to create WorkspaceApplicationService")
  }

  /// Find workspace by ID with optimized repository access
  async fn find_workspace_by_id(
    &self,
    workspace_id: WorkspaceId,
  ) -> Result<Option<fechatter_core::Workspace>, AppError> {
    // WorkspaceRepositoryImpl has its own find_by_id method (not a trait method)
    self
      .workspace_repository
      .find_by_id(workspace_id)
      .await
      .map_err(|e| AppError::InvalidInput(format!("Failed to find workspace: {}", e)))
  }

  /// Find user by ID with optimized repository access
  async fn find_user_by_id(
    &self,
    user_id: UserId,
  ) -> Result<Option<fechatter_core::User>, AppError> {
    // Use explicit dereference to call trait method on Arc<UserRepositoryImpl>
    (&*self.user_repository)
      .find_by_id(user_id)
      .await
      .map_err(|e| AppError::InvalidInput(format!("Failed to find user: {}", e)))
  }

  /// Create workspace with optimized repository access
  pub async fn create_workspace(
    &self,
    name: &str,
    owner_id: UserId,
  ) -> Result<fechatter_core::Workspace, AppError> {
    // Use optimized repository for creation with explicit dereference
    let mut workspace = (&*self.workspace_repository)
      .find_or_create_by_name(name)
      .await
      .map_err(|e| AppError::InvalidInput(format!("Failed to create workspace: {}", e)))?;

    // Update owner if needed with explicit dereference
    if workspace.owner_id != owner_id {
      workspace = (&*self.workspace_repository)
        .update_owner(workspace.id, owner_id)
        .await
        .map_err(|e| AppError::InvalidInput(format!("Failed to update workspace owner: {}", e)))?;
    }

    Ok(workspace)
  }
}

impl fmt::Debug for WorkspaceApplicationService {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("WorkspaceApplicationService").finish()
  }
}

#[async_trait]
impl WorkspaceApplicationServiceTrait for WorkspaceApplicationService {
  async fn list_workspace_users(
    &self,
    workspace_id: i64,
    user_id: i64,
  ) -> Result<Vec<UserSummaryView>, AppError> {
    // 1. Validate workspace access
    self
      .validate_workspace_access(user_id, workspace_id)
      .await?;

    // 2. Query workspace user list using optimized pool access
    let users = sqlx::query!(
      r#"
            SELECT id, fullname, email, status::text as status
            FROM users 
            WHERE workspace_id = $1 
            ORDER BY fullname
            "#,
      workspace_id
    )
    .fetch_all(self.app_state.pool())
    .await?;

    // 3. Convert to view model
    let user_summaries = users
      .into_iter()
      .map(|row| UserSummaryView {
        id: row.id,
        fullname: row.fullname,
        email: row.email,
        status: row.status.unwrap_or_default(),
      })
      .collect();

    Ok(user_summaries)
  }

  async fn update_workspace(
    &self,
    workspace_id: i64,
    user_id: i64,
    command: UpdateWorkspaceCommand,
  ) -> Result<WorkspaceView, AppError> {
    // 1. Validate owner permissions
    self.validate_workspace_owner(user_id, workspace_id).await?;

    // 2. Execute update operation
    if let Some(name) = command.name {
      sqlx::query!(
        "UPDATE workspaces SET name = $1 WHERE id = $2",
        name,
        workspace_id
      )
      .execute(self.app_state.pool())
      .await?;
    }

    // 3. Get updated workspace details
    self.get_workspace_details(workspace_id, user_id).await
  }

  async fn invite_user(
    &self,
    workspace_id: i64,
    inviter_id: i64,
    command: InviteUserCommand,
  ) -> Result<(), AppError> {
    // 1. Validate owner permissions
    self
      .validate_workspace_owner(inviter_id, workspace_id)
      .await?;

    // 2. Check if user already exists
    let existing_user = sqlx::query!("SELECT id FROM users WHERE email = $1", command.email)
      .fetch_optional(self.app_state.pool())
      .await?;

    if existing_user.is_some() {
      return Err(AppError::InvalidInput("User already exists".to_string()));
    }

    // 3. Log invitation request (simplified implementation)
    tracing::info!(
      "User invitation requested for {} to workspace {} by user {}",
      command.email,
      workspace_id,
      inviter_id
    );

    // TODO: Implement email invitation system
    // In real implementation, this should:
    // - Create invitation record
    // - Send invitation email
    // - Set invitation expiration time

    Ok(())
  }

  async fn get_workspace_details(
    &self,
    workspace_id: i64,
    user_id: i64,
  ) -> Result<WorkspaceView, AppError> {
    // 1. Validate workspace access permissions
    self
      .validate_workspace_access(user_id, workspace_id)
      .await?;

    // 2. Get workspace information using optimized repository
    let workspace = self
      .find_workspace_by_id(WorkspaceId(workspace_id))
      .await?
      .ok_or_else(|| AppError::NotFound(vec!["Workspace not found".to_string()]))?;

    // 3. Get member count
    let member_count = sqlx::query_scalar!(
      "SELECT COUNT(*) FROM users WHERE workspace_id = $1",
      workspace_id
    )
    .fetch_one(self.app_state.pool())
    .await?
    .unwrap_or(0) as i32;

    // 4. Build view model
    Ok(WorkspaceView {
      id: i64::from(workspace.id),
      name: workspace.name,
      owner_id: i64::from(workspace.owner_id),
      member_count,
      created_at: workspace.created_at,
    })
  }

  async fn validate_workspace_access(
    &self,
    user_id: i64,
    workspace_id: i64,
  ) -> Result<(), AppError> {
    // Validate access through user's workspace ID using optimized repository
    let user = self
      .find_user_by_id(UserId(user_id))
      .await?
      .ok_or_else(|| AppError::NotFound(vec!["User not found".to_string()]))?;

    let user_workspace_id = i64::from(user.workspace_id);
    if workspace_id != user_workspace_id {
      return Err(AppError::ChatPermissionError(
        "Access denied to this workspace".to_string(),
      ));
    }

    Ok(())
  }

  async fn validate_workspace_owner(
    &self,
    user_id: i64,
    workspace_id: i64,
  ) -> Result<(), AppError> {
    // 1. Get workspace information using optimized repository
    let workspace = self
      .find_workspace_by_id(WorkspaceId(workspace_id))
      .await?
      .ok_or_else(|| AppError::NotFound(vec!["Workspace not found".to_string()]))?;

    // 2. Validate owner permissions
    if i64::from(workspace.owner_id) != user_id {
      return Err(AppError::ChatPermissionError(
        "Only workspace owner can perform this action".to_string(),
      ));
    }

    Ok(())
  }
}

// ============================================================================
// OPTIMIZED ADAPTER PATTERN
// ============================================================================

/// AppState Workspace Service Adapter - Clean resource management
pub struct AppStateWorkspaceServiceAdapter {
  service: WorkspaceApplicationService,
}

impl AppStateWorkspaceServiceAdapter {
  /// Create adapter with optimized resource management
  pub fn new(state: &AppState) -> Result<Self, AppError> {
    let service = WorkspaceServiceFactory::create_from_state(state)?;
    Ok(Self { service })
  }

  /// DEPRECATED: Use new() instead
  #[deprecated(note = "Use new() for better error handling")]
  pub fn from_state_unchecked(state: AppState) -> Self {
    let service = WorkspaceServiceFactory::create_from_state(&state)
      .expect("Failed to create WorkspaceApplicationService");
    Self { service }
  }
}

#[async_trait]
impl WorkspaceApplicationServiceTrait for AppStateWorkspaceServiceAdapter {
  async fn list_workspace_users(
    &self,
    workspace_id: i64,
    user_id: i64,
  ) -> Result<Vec<UserSummaryView>, AppError> {
    self
      .service
      .list_workspace_users(workspace_id, user_id)
      .await
  }

  async fn update_workspace(
    &self,
    workspace_id: i64,
    user_id: i64,
    command: UpdateWorkspaceCommand,
  ) -> Result<WorkspaceView, AppError> {
    self
      .service
      .update_workspace(workspace_id, user_id, command)
      .await
  }

  async fn invite_user(
    &self,
    workspace_id: i64,
    inviter_id: i64,
    command: InviteUserCommand,
  ) -> Result<(), AppError> {
    self
      .service
      .invite_user(workspace_id, inviter_id, command)
      .await
  }

  async fn get_workspace_details(
    &self,
    workspace_id: i64,
    user_id: i64,
  ) -> Result<WorkspaceView, AppError> {
    self
      .service
      .get_workspace_details(workspace_id, user_id)
      .await
  }

  async fn validate_workspace_access(
    &self,
    user_id: i64,
    workspace_id: i64,
  ) -> Result<(), AppError> {
    self
      .service
      .validate_workspace_access(user_id, workspace_id)
      .await
  }

  async fn validate_workspace_owner(
    &self,
    user_id: i64,
    workspace_id: i64,
  ) -> Result<(), AppError> {
    self
      .service
      .validate_workspace_owner(user_id, workspace_id)
      .await
  }
}

impl fmt::Debug for AppStateWorkspaceServiceAdapter {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("AppStateWorkspaceServiceAdapter").finish()
  }
}

// ============================================================================
// OPTIMIZED FACTORY FUNCTIONS
// ============================================================================

/// Create workspace application service with optimized resource management
pub fn create_workspace_application_service(
  state: &AppState,
) -> Result<WorkspaceApplicationService, AppError> {
  WorkspaceServiceFactory::create_from_state(state)
}

/// Create workspace service adapter with proper error handling
pub fn create_workspace_service_adapter(
  state: &AppState,
) -> Result<AppStateWorkspaceServiceAdapter, AppError> {
  AppStateWorkspaceServiceAdapter::new(state)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_workspace_view_conversion() {
    // TODO: Add WorkspaceView conversion tests
  }

  #[test]
  fn test_builder_pattern() {
    // TODO: Add Builder pattern tests
  }

  #[test]
  fn test_factory_pattern() {
    // TODO: Add Factory pattern tests
  }
}
