#[cfg(test)]
mod tests {
    use crate::{
        services::application::workers::auth::AuthUserService,
        tests::test_utils::{create_test_state, create_test_user},
        AppError,
    };
    use fechatter_core::{CreateUser, SigninUser, UserId};

    #[tokio::test]
    async fn test_change_password_success() {
        // Setup
        let state = create_test_state().await;
        let auth_service = AuthUserService::from_app_state(&state);

        // Create a test user
        let create_user = CreateUser {
            email: "test@example.com".to_string(),
            password: "oldpassword123".to_string(),
            fullname: "Test User".to_string(),
            workspace: "Test Workspace".to_string(),
        };

        let user = auth_service
            .signup(&create_user, None)
            .await
            .expect("Failed to create user");

        // Extract user ID from the token
        let user_id = state
            .token_manager()
            .verify_token(&user.access_token)
            .expect("Failed to verify token")
            .user_id;

        // Change password
        let result = auth_service
            .change_password(UserId(user_id), "oldpassword123", "newpassword456")
            .await;

        assert!(result.is_ok());

        // Verify new password works
        let signin_result = auth_service
            .signin(
                &SigninUser {
                    email: "test@example.com".to_string(),
                    password: "newpassword456".to_string(),
                },
                None,
            )
            .await
            .expect("Failed to signin");

        assert!(signin_result.is_some());

        // Verify old password doesn't work
        let old_signin_result = auth_service
            .signin(
                &SigninUser {
                    email: "test@example.com".to_string(),
                    password: "oldpassword123".to_string(),
                },
                None,
            )
            .await
            .expect("Failed to signin");

        assert!(old_signin_result.is_none());
    }

    #[tokio::test]
    async fn test_change_password_wrong_old_password() {
        // Setup
        let state = create_test_state().await;
        let auth_service = AuthUserService::from_app_state(&state);

        // Create a test user
        let create_user = CreateUser {
            email: "test2@example.com".to_string(),
            password: "password123".to_string(),
            fullname: "Test User 2".to_string(),
            workspace: "Test Workspace".to_string(),
        };

        let user = auth_service
            .signup(&create_user, None)
            .await
            .expect("Failed to create user");

        // Extract user ID from the token
        let user_id = state
            .token_manager()
            .verify_token(&user.access_token)
            .expect("Failed to verify token")
            .user_id;

        // Try to change password with wrong old password
        let result = auth_service
            .change_password(UserId(user_id), "wrongpassword", "newpassword456")
            .await;

        assert!(result.is_err());
        match result {
            Err(AppError::Unauthorized(msg)) => {
                assert_eq!(msg, "Invalid old password");
            }
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[tokio::test]
    async fn test_change_password_validation() {
        // Setup
        let state = create_test_state().await;
        let auth_service = AuthUserService::from_app_state(&state);

        // Create a test user
        let create_user = CreateUser {
            email: "test3@example.com".to_string(),
            password: "password123".to_string(),
            fullname: "Test User 3".to_string(),
            workspace: "Test Workspace".to_string(),
        };

        let user = auth_service
            .signup(&create_user, None)
            .await
            .expect("Failed to create user");

        // Extract user ID from the token
        let user_id = state
            .token_manager()
            .verify_token(&user.access_token)
            .expect("Failed to verify token")
            .user_id;

        // Test password too short
        let result = auth_service
            .change_password(UserId(user_id), "password123", "short")
            .await;

        assert!(result.is_err());
        match result {
            Err(AppError::ValidationError(msg)) => {
                assert_eq!(msg, "Password must be at least 8 characters long");
            }
            _ => panic!("Expected ValidationError for short password"),
        }

        // Test password too long
        let long_password = "a".repeat(129);
        let result = auth_service
            .change_password(UserId(user_id), "password123", &long_password)
            .await;

        assert!(result.is_err());
        match result {
            Err(AppError::ValidationError(msg)) => {
                assert_eq!(msg, "Password must be no more than 128 characters long");
            }
            _ => panic!("Expected ValidationError for long password"),
        }
    }

    #[tokio::test]
    async fn test_change_password_nonexistent_user() {
        // Setup
        let state = create_test_state().await;
        let auth_service = AuthUserService::from_app_state(&state);

        // Try to change password for non-existent user
        let result = auth_service
            .change_password(UserId(99999), "oldpassword", "newpassword")
            .await;

        assert!(result.is_err());
        match result {
            Err(AppError::NotFound(msgs)) => {
                assert!(msgs.contains(&"User not found".to_string()));
            }
            _ => panic!("Expected NotFound error"),
        }
    }
}
