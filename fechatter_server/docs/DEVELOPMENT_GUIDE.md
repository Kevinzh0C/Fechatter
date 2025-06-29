# Fechatter Server Development Guide

## 📋 Table of Contents

1. [Getting Started](#getting-started)
2. [Project Structure](#project-structure)
3. [Development Workflow](#development-workflow)
4. [Code Style & Standards](#code-style--standards)
5. [Architecture Guidelines](#architecture-guidelines)
6. [Testing Strategy](#testing-strategy)
7. [Debugging Tips](#debugging-tips)
8. [Performance Optimization](#performance-optimization)
9. [Common Patterns](#common-patterns)
10. [Troubleshooting](#troubleshooting)

## 🚀 Getting Started

### Prerequisites

- **Rust**: 1.70.0 or higher
- **PostgreSQL**: 14.0 or higher
- **Redis**: 6.0 or higher
- **Git**: 2.25 or higher
- **Docker** (optional): For containerized development

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/fechatter.git
cd fechatter/fechatter_server

# Install Rust dependencies
cargo build

# Setup environment variables
cp .env.example .env
# Edit .env with your configuration

# Run database migrations
cargo run --bin migrate

# Run the development server
cargo run
```

### Recommended Tools

- **rust-analyzer**: IDE support
- **cargo-watch**: Auto-reload on changes
- **cargo-expand**: Macro expansion
- **cargo-flamegraph**: Performance profiling
- **sqlx-cli**: Database migrations

```bash
# Install development tools
cargo install cargo-watch cargo-expand cargo-flamegraph sqlx-cli
```

## 📁 Project Structure

```
fechatter_server/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library root
│   ├── config.rs            # Configuration management
│   ├── error.rs             # Error types and handling
│   ├── state.rs             # Application state
│   │
│   ├── domains/             # Business logic layer
│   │   ├── auth/            # Authentication domain
│   │   ├── chat/            # Chat management
│   │   ├── message/         # Message handling
│   │   └── workspace/       # Workspace logic
│   │
│   ├── services/            # Service layer
│   │   ├── application/     # Application services
│   │   │   ├── builders/    # Service construction
│   │   │   ├── workers/     # Business workers
│   │   │   ├── flows/       # Event flows
│   │   │   └── stores/      # Data stores
│   │   └── infrastructure/  # External services
│   │
│   ├── handlers/            # HTTP handlers
│   │   ├── auth.rs          # Auth endpoints
│   │   ├── chat.rs          # Chat endpoints
│   │   └── health.rs        # Health checks
│   │
│   ├── middlewares/         # Request middleware
│   │   ├── auth.rs          # Authentication
│   │   ├── cors.rs          # CORS handling
│   │   └── rate_limit.rs    # Rate limiting
│   │
│   └── interfaces/          # External interfaces
│       ├── dtos/            # Data transfer objects
│       └── repositories/    # Data access
│
├── tests/                   # Integration tests
├── migrations/              # Database migrations
└── docs/                    # Documentation
```

## 🔄 Development Workflow

### 1. Feature Development

```bash
# Create feature branch
git checkout -b feature/your-feature-name

# Develop with auto-reload
cargo watch -x run

# Run tests continuously
cargo watch -x test

# Format code
cargo fmt

# Check lints
cargo clippy
```

### 2. Database Changes

```bash
# Create new migration
sqlx migrate add your_migration_name

# Apply migrations
cargo run --bin migrate

# Revert migration
cargo run --bin migrate -- --revert
```

### 3. Adding New Endpoints

1. **Define DTO** in `src/interfaces/dtos/`
2. **Create Handler** in `src/handlers/`
3. **Add Route** in `src/lib.rs`
4. **Write Tests** in `tests/`

Example:
```rust
// src/interfaces/dtos/example.rs
#[derive(Deserialize, Validate)]
pub struct CreateExampleRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
}

// src/handlers/example.rs
pub async fn create_example(
    State(state): State<AppState>,
    Json(payload): Json<CreateExampleRequest>,
) -> Result<Json<ExampleResponse>, AppError> {
    payload.validate()?;
    // Implementation
}

// src/lib.rs (in router)
.route("/api/examples", post(handlers::example::create_example))
```

## 📝 Code Style & Standards

### Rust Style Guide

Follow the official [Rust Style Guide](https://doc.rust-lang.org/style-guide/) with these additions:

```rust
// ✅ Good: Descriptive names
pub async fn create_user_with_workspace(
    user_data: CreateUserRequest,
    workspace_id: i64,
) -> Result<User, CoreError> {
    // Implementation
}

// ❌ Bad: Unclear names
pub async fn cu(d: CUR, w: i64) -> Result<U, E> {
    // Implementation
}
```

### Error Handling

Always use proper error types:

```rust
// Define domain-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("Chat not found: {0}")]
    NotFound(i64),
    
    #[error("User not authorized")]
    Unauthorized,
    
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

// Use Result type alias
pub type ChatResult<T> = Result<T, ChatError>;
```

### Documentation

Document all public APIs:

```rust
/// Creates a new chat room in the specified workspace.
///
/// # Arguments
/// * `workspace_id` - The workspace to create the chat in
/// * `name` - The name of the chat room
/// * `is_private` - Whether the chat is private
///
/// # Returns
/// The newly created chat
///
/// # Errors
/// Returns `ChatError::Unauthorized` if the user lacks permissions
pub async fn create_chat(
    workspace_id: i64,
    name: String,
    is_private: bool,
) -> ChatResult<Chat> {
    // Implementation
}
```

## 🏗️ Architecture Guidelines

### Clean Architecture Principles

1. **Dependency Rule**: Dependencies point inward
2. **Domain Independence**: Business logic doesn't depend on frameworks
3. **Testability**: Each layer is independently testable

### Service Layer Pattern

```rust
// Service trait definition
#[async_trait]
pub trait ChatService: Send + Sync {
    async fn create_chat(&self, req: CreateChatRequest) -> Result<Chat>;
    async fn list_chats(&self, workspace_id: i64) -> Result<Vec<Chat>>;
}

// Implementation
pub struct ChatServiceImpl {
    repository: Arc<dyn ChatRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

#[async_trait]
impl ChatService for ChatServiceImpl {
    async fn create_chat(&self, req: CreateChatRequest) -> Result<Chat> {
        // Validate
        req.validate()?;
        
        // Create
        let chat = self.repository.create(req).await?;
        
        // Publish event
        self.event_publisher.publish(ChatCreated { chat: chat.clone() }).await?;
        
        Ok(chat)
    }
}
```

### Repository Pattern

```rust
#[async_trait]
pub trait ChatRepository: Send + Sync {
    async fn find_by_id(&self, id: i64) -> Result<Option<Chat>>;
    async fn create(&self, chat: CreateChatRequest) -> Result<Chat>;
    async fn update(&self, id: i64, chat: UpdateChatRequest) -> Result<Chat>;
    async fn delete(&self, id: i64) -> Result<()>;
}
```

## 🧪 Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_chat_validates_name() {
        let service = create_test_service();
        let req = CreateChatRequest {
            name: "".to_string(), // Invalid
            workspace_id: 1,
        };
        
        let result = service.create_chat(req).await;
        assert!(result.is_err());
    }
}
```

### Integration Tests

```rust
// tests/chat_integration_test.rs
#[tokio::test]
async fn test_chat_creation_flow() {
    let app = spawn_test_app().await;
    
    // Create user
    let user = create_test_user(&app).await;
    
    // Create chat
    let response = app.client
        .post("/api/chats")
        .json(&json!({
            "name": "test-chat",
            "workspace_id": 1
        }))
        .header("Authorization", &user.token)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::CREATED);
}
```

### Performance Tests

```rust
#[tokio::test]
async fn test_message_throughput() {
    let app = spawn_test_app().await;
    let start = Instant::now();
    
    // Send 1000 messages
    for i in 0..1000 {
        send_test_message(&app, i).await;
    }
    
    let duration = start.elapsed();
    assert!(duration < Duration::from_secs(10), "Messages took too long");
}
```

## 🐛 Debugging Tips

### Logging

```rust
use tracing::{debug, error, info, warn};

#[instrument(skip(pool))]
pub async fn complex_operation(
    pool: &PgPool,
    user_id: i64,
) -> Result<Data> {
    info!("Starting complex operation for user {}", user_id);
    
    let data = match fetch_data(pool, user_id).await {
        Ok(data) => {
            debug!("Fetched {} records", data.len());
            data
        }
        Err(e) => {
            error!("Failed to fetch data: {:?}", e);
            return Err(e);
        }
    };
    
    Ok(data)
}
```

### Environment Variables

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Enable specific module logging
RUST_LOG=fechatter_server::handlers=debug cargo run

# Enable SQL query logging
RUST_LOG=sqlx=debug cargo run
```

### Database Debugging

```rust
// Log SQL queries
let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect_with(
        PgConnectOptions::from_str(&database_url)?
            .log_statements(LevelFilter::Debug)
    )
    .await?;
```

## ⚡ Performance Optimization

### Query Optimization

```rust
// ❌ Bad: N+1 query
for chat_id in chat_ids {
    let messages = get_messages(chat_id).await?;
}

// ✅ Good: Batch query
let messages = sqlx::query_as!(
    Message,
    "SELECT * FROM messages WHERE chat_id = ANY($1)",
    &chat_ids
)
.fetch_all(&pool)
.await?;
```

### Caching Strategy

```rust
// Use caching for expensive operations
pub async fn get_user_stats(user_id: i64) -> Result<UserStats> {
    // Check cache first
    if let Some(stats) = cache.get(&format!("user_stats:{}", user_id)).await? {
        return Ok(stats);
    }
    
    // Calculate stats
    let stats = calculate_user_stats(user_id).await?;
    
    // Cache for 5 minutes
    cache.set(
        &format!("user_stats:{}", user_id),
        &stats,
        Duration::from_secs(300)
    ).await?;
    
    Ok(stats)
}
```

### Connection Pooling

```rust
// Configure optimal pool size
let pool = PgPoolOptions::new()
    .max_connections(100)
    .min_connections(10)
    .acquire_timeout(Duration::from_secs(3))
    .idle_timeout(Duration::from_secs(600))
    .connect(&database_url)
    .await?;
```

## 🎯 Common Patterns

### Builder Pattern

```rust
pub struct ServiceProviderBuilder {
    pool: Option<PgPool>,
    redis: Option<RedisClient>,
    config: ServiceConfig,
}

impl ServiceProviderBuilder {
    pub fn new() -> Self {
        Self {
            pool: None,
            redis: None,
            config: ServiceConfig::default(),
        }
    }
    
    pub fn with_pool(mut self, pool: PgPool) -> Self {
        self.pool = Some(pool);
        self
    }
    
    pub fn with_redis(mut self, redis: RedisClient) -> Self {
        self.redis = Some(redis);
        self
    }
    
    pub fn build(self) -> Result<ServiceProvider> {
        Ok(ServiceProvider {
            pool: self.pool.ok_or("Pool required")?,
            redis: self.redis.ok_or("Redis required")?,
            config: self.config,
        })
    }
}
```

### Factory Pattern

```rust
pub trait ServiceFactory: Send + Sync {
    type Service;
    
    fn create(&self) -> Self::Service;
}

pub struct ChatServiceFactory {
    pool: Arc<PgPool>,
}

impl ServiceFactory for ChatServiceFactory {
    type Service = Arc<dyn ChatService>;
    
    fn create(&self) -> Self::Service {
        Arc::new(ChatServiceImpl::new(self.pool.clone()))
    }
}
```

## 🔧 Troubleshooting

### Common Issues

#### 1. Database Connection Errors
```bash
# Check PostgreSQL is running
pg_isready

# Check connection string
psql $DATABASE_URL

# Check migrations
cargo run --bin migrate -- --status
```

#### 2. Redis Connection Errors
```bash
# Test Redis connection
redis-cli ping

# Check Redis config
redis-cli CONFIG GET bind
```

#### 3. Build Errors
```bash
# Clean build
cargo clean
cargo build

# Update dependencies
cargo update

# Check for outdated dependencies
cargo outdated
```

### Performance Issues

#### Slow Queries
```sql
-- Find slow queries
SELECT query, calls, mean_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;

-- Analyze query plan
EXPLAIN ANALYZE SELECT * FROM your_query;
```

#### High Memory Usage
```bash
# Profile memory usage
cargo flamegraph --bin fechatter_server

# Check for leaks
valgrind --leak-check=full target/debug/fechatter_server
```

## 📚 Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Axum Examples](https://github.com/tokio-rs/axum/tree/main/examples)

---

**Version**: 1.0.0  
**Last Updated**: December 2024  
**Status**: Active Development 🚧 