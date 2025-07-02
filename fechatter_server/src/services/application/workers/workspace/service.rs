//! # Workspace Application Service
//!
//! **Single Responsibility**: Orchestrate workspace use cases by delegating to domain service
//! **NOT responsible for**:
//! - Authentication/Authorization (handled by middleware)
//! - Direct data access (handled by domain/repository)
//! - Business rules validation (handled by domain service)
//!
//! **Architecture Role**: Application Service Layer - thin orchestration layer
//! **Design Principle**: Keep it simple - no unnecessary patterns

use fechatter_core::{UserId, WorkspaceId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domains::workspace::repository::WorkspaceRepositoryImpl;
use crate::domains::workspace::workspace_domain::{
    WorkspaceConfig, WorkspaceDomainService, WorkspaceDomainServiceImpl,
};
use crate::handlers::workspaces::UpdateWorkspaceRequest;
use crate::{AppError, AppState};

// ============================================================================
// VIEW MODELS - API Response DTOs
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
// COMMAND MODELS - API Request DTOs
// ============================================================================

#[derive(Deserialize, Clone, Debug)]
pub struct UpdateWorkspaceCommand {
    pub name: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct InviteUserCommand {
    pub email: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AddMembersCommand {
    pub member_ids: Vec<i64>,
}

// ============================================================================
// APPLICATION SERVICE - Simple, focused implementation
// ============================================================================

/// Workspace Application Service - Orchestrates workspace use cases
///
/// This is a thin layer that:
/// 1. Receives commands from handlers
/// 2. Delegates to domain service for business logic
/// 3. Converts domain objects to view models
///
/// It does NOT handle auth, validation, or direct data access
pub struct WorkspaceApplicationService {
    workspace_domain_service: Arc<dyn WorkspaceDomainService>,
    pool: Arc<sqlx::PgPool>, // Only for user list query (TODO: move to domain)
}

impl WorkspaceApplicationService {
    /// Create service from AppState - simple factory method
    pub fn from_app_state(state: &AppState) -> Self {
        let pool = state.pool().clone();
        let workspace_repository = Arc::new(WorkspaceRepositoryImpl::new(pool.clone()));
        let workspace_domain_service = Arc::new(WorkspaceDomainServiceImpl::new(
            workspace_repository,
            WorkspaceConfig::default(),
        )) as Arc<dyn WorkspaceDomainService>;

        Self {
            workspace_domain_service,
            pool,
        }
    }

    /// Create service with direct dependencies - used by ServiceProvider
    /// This avoids circular dependency with AppState
    pub fn new(
        workspace_domain_service: Arc<dyn WorkspaceDomainService>,
        pool: Arc<sqlx::PgPool>,
    ) -> Self {
        Self {
            workspace_domain_service,
            pool,
        }
    }

    /// Create workspace - delegates to domain service
    pub async fn create_workspace(
        &self,
        name: &str,
        owner_id: UserId,
    ) -> Result<WorkspaceView, AppError> {
        let workspace = self
            .workspace_domain_service
            .create_workspace(name, owner_id)
            .await?;

        // Convert to view model
        Ok(WorkspaceView {
            id: i64::from(workspace.id),
            name: workspace.name,
            owner_id: i64::from(workspace.owner_id),
            member_count: 1, // Owner is the first member
            created_at: workspace.created_at,
        })
    }

    /// Get workspace details - delegates to domain service
    pub async fn get_workspace_details(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<WorkspaceView, AppError> {
        let aggregate = self
            .workspace_domain_service
            .get_workspace_aggregate(workspace_id)
            .await?;

        // Convert aggregate to view model
        Ok(WorkspaceView {
            id: i64::from(aggregate.workspace.id),
            name: aggregate.workspace.name.clone(),
            owner_id: i64::from(aggregate.workspace.owner_id),
            member_count: aggregate.active_member_count() as i32,
            created_at: aggregate.workspace.created_at,
        })
    }

    /// Update workspace - delegates to domain service
    pub async fn update_workspace(
        &self,
        workspace_id: WorkspaceId,
        user_id: UserId,
        command: UpdateWorkspaceCommand,
    ) -> Result<WorkspaceView, AppError> {
        let update_request = UpdateWorkspaceRequest { name: command.name };

        let workspace = self
            .workspace_domain_service
            .update_workspace(workspace_id, &update_request, user_id)
            .await?;

        // Get full details for response
        self.get_workspace_details(workspace_id).await
    }

    /// List workspace users
    /// Delegates to domain service for data retrieval
    pub async fn list_workspace_users(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Vec<UserSummaryView>, AppError> {
        // Delegate to domain service
        let users = self
            .workspace_domain_service
            .list_users(workspace_id)
            .await?;

        // Transform WorkspaceUser to UserSummaryView
        Ok(users
            .into_iter()
            .map(|user| UserSummaryView {
                id: i64::from(user.id),
                fullname: user.fullname,
                email: user.email,
                status: match user.status {
                    fechatter_core::UserStatus::Active => "Active".to_string(),
                    fechatter_core::UserStatus::Suspended => "Suspended".to_string(),
                },
            })
            .collect())
    }

    /// Invite user to workspace
    /// TODO: Implement actual invitation logic in domain service
    pub async fn invite_user(
        &self,
        workspace_id: WorkspaceId,
        inviter_id: UserId,
        command: InviteUserCommand,
    ) -> Result<(), AppError> {
        // For now, just log the invitation
        tracing::info!(
            "User invitation requested for {} to workspace {} by user {}",
            command.email,
            workspace_id,
            inviter_id
        );

        // TODO: Implement in domain service:
        // 1. Check if user exists
        // 2. Create invitation record
        // 3. Send invitation email

        Ok(())
    }

    /// Add members to workspace
    /// Delegates to domain service for business logic and validation
    pub async fn add_members(
        &self,
        workspace_id: WorkspaceId,
        admin_user_id: UserId,
        command: AddMembersCommand,
    ) -> Result<Vec<UserSummaryView>, AppError> {
        // Convert member IDs to UserId type
        let member_ids: Vec<UserId> = command.member_ids.into_iter().map(UserId::from).collect();

        // Delegate to domain service
        let users = self
            .workspace_domain_service
            .add_members(workspace_id, member_ids, admin_user_id)
            .await?;

        // Transform to view models
        Ok(users
            .into_iter()
            .map(|user| UserSummaryView {
                id: i64::from(user.id),
                fullname: user.fullname,
                email: user.email,
                status: match user.status {
                    fechatter_core::UserStatus::Active => "Active".to_string(),
                    fechatter_core::UserStatus::Suspended => "Suspended".to_string(),
                },
            })
            .collect())
    }
}

// ============================================================================
// PUBLIC API - Simplified exports
// ============================================================================

/// Create workspace application service - single factory function
pub fn create_workspace_application_service(
    state: &AppState,
) -> Result<WorkspaceApplicationService, AppError> {
    Ok(WorkspaceApplicationService::from_app_state(state))
}

// Re-export service for convenience
pub use WorkspaceApplicationService as WorkspaceService;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_view_creation() {
        // Test view model creation
    }
}
