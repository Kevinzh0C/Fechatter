# Testing Strategy

This document outlines the testing strategy for the Fechatter application, which follows a clean architecture pattern. The two main components, `fechatter_core` and `fechatter_server`, require different testing approaches due to their separation of concerns.

## Core Library Testing

The `fechatter_core` library tests focus on pure algorithmic functions and business logic without database dependencies.

### Unit Tests

Unit tests in the core library should:

1. Test pure functions directly
2. Use mock implementations of repository interfaces
3. Test business logic in isolation

Example of testing a pure function:

```rust
#[test]
fn validate_chat_name_should_work() {
    // Test valid name
    let result = validate_chat_name("My Chat");
    assert!(result.is_ok());
    
    // Test empty name
    let result = validate_chat_name("");
    assert!(result.is_err());
    
    // Test too long name
    let result = validate_chat_name("a".repeat(129));
    assert!(result.is_err());
}
```

### Mock Repositories

For testing components that depend on repositories, create mock implementations:

```rust
struct MockUserRepository {
    users: std::sync::Mutex<Vec<User>>,
}

#[async_trait::async_trait]
impl UserRepository for MockUserRepository {
    async fn create(&self, input: &CreateUser) -> Result<User, CoreError> {
        let user = User {
            id: 1,
            fullname: input.fullname.clone(),
            email: input.email.clone(),
            password_hash: Some("hashed".to_string()),
            status: UserStatus::Active,
            created_at: chrono::Utc::now(),
            workspace_id: 1,
        };
        
        self.users.lock().unwrap().push(user.clone());
        Ok(user)
    }
    
    // Implement other methods...
}
```

### Test Helpers

The core library provides test helper functions in `utils::test_helpers` to create test data:

```rust
// Create a test user
let user = create_test_user(1, 10);

// Create a token manager for testing
let token_manager = create_test_token_manager()?;
```

## Server Implementation Testing

The `fechatter_server` includes database-dependent tests using real repositories.

### Integration Tests

The server crate includes the `setup_test_users!` macro for creating test databases and users:

```rust
#[tokio::test]
async fn test_user_repository() {
    let (pool, _state, users) = setup_test_users!(2);
    let repo = PgUserRepository::new(pool);
    
    // Test with real database
    let user = repo.find_by_id(users[0].id).await.unwrap();
    assert!(user.is_some());
}
```

### Repository Tests

Test each repository implementation to ensure it correctly implements the interface:

```rust
#[tokio::test]
async fn test_refresh_token_repository() {
    let (pool, _state, users) = setup_test_users!(1);
    let repo = PgRefreshTokenRepository::new(pool);
    
    let token = generate_refresh_token();
    let token_info = repo.create(users[0].id, &token, None, None).await.unwrap();
    
    let found = repo.find_by_token(&token).await.unwrap();
    assert!(found.is_some());
}
```

## Testing Middleware

The type-state based middleware system requires special testing approaches to verify correctness.

### Unit Testing Middleware Functions

Individual middleware functions should be tested in isolation:

```rust
#[tokio::test]
async fn test_auth_middleware() {
    // Create a mock token verifier
    let token_verifier = MockTokenVerifier::new();
    token_verifier.set_should_succeed(true);
    
    // Create a test request with a valid token
    let request = Request::builder()
        .uri("/test")
        .header("Authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap();
    
    // Test the middleware directly
    let response = verify_token_middleware(State(token_verifier), request, next_mock())
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
}
```

### Testing Middleware Chains

To test chains of middleware, use the middleware builder with mock state:

```rust
#[tokio::test]
async fn test_middleware_chain() {
    // Create a mock state with required implementations
    let mock_state = MockAppState::new();
    
    // Create a test router with middleware chain
    let app = Router::new()
        .route("/test", get(test_handler))
        .with_middlewares(mock_state.clone())
        .with_auth()
        .with_token_refresh()
        .build();
    
    // Create a test server
    let server = MockServer::new(app);
    
    // Test with valid credentials
    let request = Request::builder()
        .uri("/test")
        .header("Authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap();
    
    let response = server.send_request(request).await;
    assert_eq!(response.status(), StatusCode::OK);
}
```

### Testing Middleware Order

The type-state pattern should prevent incorrect middleware ordering at compile time, but the behavior can be verified with tests:

```rust
#[tokio::test]
async fn test_middleware_execution_order() {
    // Create a tracker to monitor middleware call order
    let call_tracker = MiddlewareCallTracker::new();
    
    // Create a mock state with the tracker
    let mock_state = MockAppState::new_with_tracker(call_tracker.clone());
    
    // Create a test router with middleware chain
    let app = Router::new()
        .route("/test", get(test_handler))
        .with_middlewares(mock_state.clone())
        .with_auth()
        .with_token_refresh()
        .build();
    
    // Create a test server and send a request
    let server = MockServer::new(app);
    let request = Request::builder()
        .uri("/test")
        .header("Authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap();
    
    let _ = server.send_request(request).await;
    
    // Verify middleware was called in the correct order
    let calls = call_tracker.get_calls();
    assert_eq!(calls[0], "auth");
    assert_eq!(calls[1], "refresh");
}
```

## Testing Procedural Macros

The middleware builder procedural macro should be tested separately:

```rust
#[test]
fn test_middleware_builder_macro() {
    // This is a compile-time test - the code should compile
    // without any type errors if the macro is working correctly
    
    #[middleware_builder(
        state(TestState1) -> with_test1 uses "crate::test::test_middleware1" => TestState2,
        state(TestState2) -> with_test2 uses "crate::test::test_middleware2" => TestState3
    )]
    struct TestBuilder;
    
    // The macro should generate the required structs and methods
    // so that this code compiles without errors
    let _router = Router::new()
        .route("/test", get(|| async { "test" }))
        .with_middlewares(MockState {})
        .with_test1()
        .with_test2()
        .build();
}
```

## Test Environment

The server tests use:

1. A test database (`fechatter_test`)
2. Tables created fresh for each test
3. Test data that is isolated between tests

## Mocking Cross-Crate Dependencies

When testing code that depends on components from both crates:

1. In the core crate: Use mock implementations of repository traits
2. In the server crate: Use real implementations with test databases

## Continuous Integration

Tests are run as part of the CI pipeline to ensure:

1. All pure functions work correctly
2. Repository implementations satisfy their contracts
3. Server components integrate properly with the database
4. Middleware behaves correctly in various scenarios 