# Repository Implementation Guide

This document provides guidance on implementing the repository interfaces defined in the `fechatter_core` crate.

## Overview

The repository pattern abstracts data access operations behind interfaces (traits). This allows the core business logic to remain independent of specific database technologies or storage mechanisms.

## Repository Implementation

To implement a repository, you need to:

1. Create a concrete struct in the `fechatter_server` crate
2. Implement the corresponding trait from `fechatter_core` for your struct
3. Implement the required methods with database operations

## Example Implementation

Here's an example of implementing the `UserRepository` trait:

```rust
use fechatter_core::{
    error::CoreError,
    models::{AuthUser, CreateUser, SigninUser, User, UserRepository, UserStatus},
};
use sqlx::PgPool;

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, input: &CreateUser) -> Result<User, CoreError> {
        // Implementation with SQL queries
        let password_hash = crate::utils::hashed_password(&input.password)?;
        
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (fullname, email, password_hash, status, workspace_id) 
             VALUES ($1, $2, $3, $4, $5) 
             RETURNING id, fullname, email, status, created_at, workspace_id",
        )
        .bind(&input.fullname)
        .bind(&input.email)
        .bind(&password_hash)
        .bind(UserStatus::Active)
        .bind(workspace_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return CoreError::Conflict(format!("Email {} already exists", input.email));
                }
            }
            CoreError::Database(e)
        })?;

        Ok(user)
    }

    // Implement other required methods...
}
```

## Using Repositories in Middleware

Middleware often needs to access repositories to perform data validation or authentication. Here's how to use repositories in middleware components:

### Access Through Application State

When implementing middleware, access repositories through the application state:

```rust
async fn workspace_middleware<S, AppState>(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> 
where
    AppState: WithServiceProvider,
{
    // Extract workspace_id from path parameters or query params
    let workspace_id = extract_workspace_id(&request)?;
    
    // Get the authenticated user from request extensions
    let auth_user = request.extensions().get::<AuthUser>().ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Access the workspace repository through the service provider
    let workspace_repo = state.service_provider().workspace_repository();
    
    // Validate workspace access
    let has_access = workspace_repo.check_user_access(auth_user.id, workspace_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !has_access {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Proceed to the next middleware/handler
    Ok(next.run(request).await)
}
```

### Using WithServiceProvider Trait

The `WithServiceProvider` trait provides access to all repositories:

```rust
pub trait WithServiceProvider {
    type ServiceProviderType;
    
    fn service_provider(&self) -> &Self::ServiceProviderType;
}
```

Implement this trait for your application state:

```rust
impl WithServiceProvider for AppState {
    type ServiceProviderType = ServiceProvider;
    
    fn service_provider(&self) -> &ServiceProvider {
        &self.service_provider
    }
}
```

### State Management in Middleware Chain

The type-state builder pattern ensures that middleware is applied in the correct order, which matters for repository access:

1. **Authentication middleware** must run first to populate the `AuthUser` in request extensions
2. **Workspace middleware** can then use the authenticated user to validate workspace access
3. **Chat membership middleware** can check if the user has access to a specific chat

Example middleware chain:

```rust
let router = Router::new()
    .route("/api/chat/:id/messages", get(list_messages))
    .with_middlewares(app_state)
    .with_auth()                // Add AuthUser to request
    .with_token_refresh()       // Refresh tokens if needed
    .with_workspace()           // Validate workspace access
    .with_chat_membership()     // Validate chat membership
    .build();
```

## Testing Repositories

You should test your repository implementations to ensure they correctly implement the interfaces:

```rust
#[tokio::test]
async fn test_user_repository() -> Result<()> {
    let pool = initialize_test_db().await?;
    let repo = PgUserRepository::new(pool);
    
    let input = CreateUser::new("Test User", "test@example.com", "TestWorkspace", "password123");
    let user = repo.create(&input).await?;
    
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.fullname, "Test User");
    
    let found_user = repo.find_by_id(user.id).await?;
    assert!(found_user.is_some());
    
    Ok(())
}
```

## Alternative Implementations

You can create multiple implementations of the same repository interface:

1. **Production implementation**: Uses a real database
2. **In-memory implementation**: Uses hash maps for testing
3. **Cached implementation**: Adds caching to improve performance

## Common Patterns

1. **Error handling**: Convert database errors to appropriate `CoreError` variants
2. **Transactions**: Use transactions for operations that affect multiple tables
3. **Logging**: Add logging for debugging and monitoring
4. **Pagination**: Implement efficient pagination for list operations 