# Fechatter Integration Tests

This directory contains comprehensive integration tests for the Fechatter application.

## Test Structure

```
fechatter_tests/
├── src/
│   ├── common/           # Shared test utilities
│   │   ├── mod.rs       # Module exports
│   │   ├── test_env.rs  # Test environment setup
│   │   ├── test_config.rs # Configuration management
│   │   ├── http_client.rs # HTTP client wrapper
│   │   ├── nats_utils.rs # NATS testing utilities
│   │   ├── test_fixtures.rs # Test data generators
│   │   └── test_context.rs # High-level test context
│   ├── api_tests.rs     # REST API tests
│   ├── auth_tests.rs    # Authentication tests
│   ├── database_tests.rs # Database operations tests
│   ├── file_tests.rs    # File upload/download tests
│   ├── nats_tests.rs    # NATS messaging tests
│   ├── notification_tests.rs # Real-time notification tests
│   ├── search_tests.rs  # Search functionality tests
│   ├── stress_tests.rs  # Performance and stress tests
│   └── workspace_tests.rs # Workspace management tests
├── tests/
│   ├── auth_integration_test.rs # Additional auth tests
│   └── server_integration_test.rs # Server integration tests
└── Cargo.toml
```

## Test Categories

### 1. API Tests (`api_tests.rs`)
- User registration and authentication
- Chat creation and management
- Message sending and retrieval
- File operations
- Chat member management
- Concurrent operations
- Edge cases and error handling

### 2. Authentication Tests (`auth_tests.rs`)
- User registration flow
- Login/logout functionality
- Token generation and refresh
- Password security
- Session management
- Duplicate registration prevention
- Invalid credential handling

### 3. Database Tests (`database_tests.rs`)
- Connection management
- Transaction handling
- Concurrent access
- Data integrity
- Performance benchmarks
- Error recovery

### 4. File Tests (`file_tests.rs`)
- File upload functionality
- Multiple file handling
- Large file uploads
- File download
- Invalid file handling
- Storage integration

### 5. NATS Tests (`nats_tests.rs`)
- Basic pub/sub operations
- JetStream integration
- Event publishing
- Message delivery guarantees
- Performance testing
- Error handling

### 6. Notification Tests (`notification_tests.rs`)
- Real-time message notifications
- User status updates
- Chat member events
- Event deduplication
- Cross-chat notifications
- Performance under load

### 7. Search Tests (`search_tests.rs`)
- Message content search
- Cross-chat search
- Search pagination
- Special character handling
- Empty result handling
- Search performance

### 8. Stress Tests (`stress_tests.rs`)
- Concurrent user creation
- High-volume message sending
- Large chat room handling
- Message history performance
- Resource limit testing
- Concurrent search operations

### 9. Workspace Tests (`workspace_tests.rs`)
- Workspace creation
- Member management
- Workspace isolation
- Cross-workspace restrictions
- Permission enforcement

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Category
```bash
# Run only API tests
cargo test api_tests

# Run only stress tests
cargo test stress_tests

# Run with logging
RUST_LOG=info cargo test

# Run with custom configuration
TEST_NATS_URL=nats://custom:4222 cargo test
```

### Run Single Test
```bash
cargo test test_create_chat_api -- --exact
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `TEST_NATS_URL` | NATS server URL | `nats://localhost:4222` |
| `TEST_NATS_TIMEOUT` | NATS operation timeout (seconds) | `5` |
| `TEST_CONCURRENT_MESSAGES` | Number of concurrent messages for stress tests | `10` |
| `TEST_PERFORMANCE_MESSAGES` | Number of messages for performance tests | `50` |
| `DATABASE_URL` | PostgreSQL connection string | (from workspace) |
| `RUST_LOG` | Logging level | `info` |

## Test Utilities

### TestEnvironment
Manages test database and optional NATS connection:
```rust
let mut env = TestEnvironment::new().await?;
let users = env.create_test_users(3).await?;
```

### HttpClient
HTTP client wrapper for API testing:
```rust
let client = HttpClient::new(&base_url);
client.set_auth_token(&token);
let response = client.post("/api/chat", &payload).await?;
```

### NatsTestUtils
NATS testing utilities:
```rust
let utils = NatsTestUtils::new(nats_client);
utils.publish("subject", payload).await?;
utils.check_jetstream().await?;
```

### TestFixtures
Generate test data:
```rust
let user_data = TestFixtures::create_user("test_user");
let chat_data = TestFixtures::create_chat("test_chat");
```

## Best Practices

1. **Isolation**: Each test creates its own data and cleans up after itself
2. **Unique Names**: Use timestamps or UUIDs to avoid conflicts
3. **Resource Management**: Always clean up connections and data
4. **Error Handling**: Tests should handle missing services gracefully
5. **Performance**: Use reasonable timeouts and limits
6. **Logging**: Use appropriate log levels for debugging

## Troubleshooting

### NATS Not Available
Tests will skip NATS-related functionality if NATS is not running:
```
WARN fechatter_tests::nats_tests: NATS not available, skipping test
```

### Database Connection Issues
Ensure PostgreSQL is running and DATABASE_URL is set correctly.

### Timeout Issues
Adjust timeouts via environment variables if tests are timing out.

### Port Conflicts
Tests use random ports for test servers to avoid conflicts.

## Contributing

When adding new tests:
1. Place them in the appropriate test file or create a new module
2. Use the common test utilities for consistency
3. Add documentation for new test categories
4. Ensure tests are independent and can run in parallel
5. Handle service availability gracefully
6. Add appropriate logging for debugging

## Performance Benchmarks

Expected performance metrics (on standard development machine):
- User creation: ~50 users/second
- Message sending: ~100 messages/second
- Search operations: ~20 searches/second
- File uploads: Depends on file size and network

## CI/CD Integration

Tests are designed to run in CI/CD pipelines:
- All tests can run in parallel
- External service dependencies are optional
- Configurable via environment variables
- Proper exit codes and error reporting 