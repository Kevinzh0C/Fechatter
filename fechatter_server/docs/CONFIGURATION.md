# Fechatter Server Configuration Guide

## 📋 Table of Contents

1. [Configuration Overview](#configuration-overview)
2. [Environment Variables](#environment-variables)
3. [Configuration Files](#configuration-files)
4. [Database Configuration](#database-configuration)
5. [Redis Configuration](#redis-configuration)
6. [Security Configuration](#security-configuration)
7. [Performance Tuning](#performance-tuning)
8. [Feature Flags](#feature-flags)
9. [Logging Configuration](#logging-configuration)
10. [Advanced Configuration](#advanced-configuration)

## 🎯 Configuration Overview

Fechatter Server uses a hierarchical configuration system that supports:

1. **Environment Variables** (highest priority)
2. **Configuration Files** (YAML/TOML)
3. **Default Values** (lowest priority)

### Configuration Loading Order

```rust
// 1. Default configuration
let config = Config::default();

// 2. Load from file
config.merge(File::with_name("config/default"))?;

// 3. Load environment-specific file
config.merge(File::with_name(&format!("config/{}", env)))?;

// 4. Override with environment variables
config.merge(Environment::with_prefix("FECHATTER"))?;
```

## 🔧 Environment Variables

### Core Application Settings

```bash
# Application
FECHATTER_ENV=production              # Environment: development, staging, production
FECHATTER_HOST=0.0.0.0               # Server bind address
FECHATTER_PORT=8080                  # Server port
FECHATTER_WORKERS=8                  # Number of worker threads (default: CPU cores)

# Database
DATABASE_URL=postgresql://user:pass@localhost:5432/fechatter
DATABASE_MAX_CONNECTIONS=100         # Maximum pool connections
DATABASE_MIN_CONNECTIONS=10          # Minimum pool connections
DATABASE_ACQUIRE_TIMEOUT=3           # Connection acquire timeout (seconds)
DATABASE_IDLE_TIMEOUT=600            # Idle connection timeout (seconds)
DATABASE_MAX_LIFETIME=1800           # Maximum connection lifetime (seconds)

# Redis
REDIS_URL=redis://localhost:6379
REDIS_POOL_SIZE=50                   # Connection pool size
REDIS_CONNECTION_TIMEOUT=5           # Connection timeout (seconds)
REDIS_COMMAND_TIMEOUT=5              # Command timeout (seconds)

# Security
JWT_SECRET=your-256-bit-secret-key   # JWT signing key (required)
JWT_EXPIRY_SECONDS=3600             # Token expiration time
BCRYPT_COST=12                      # Password hashing cost (10-15)
ENCRYPTION_KEY=your-32-byte-key     # Data encryption key

# Search (Optional)
MEILISEARCH_URL=http://localhost:7700
MEILISEARCH_KEY=masterKey            # Master key for Meilisearch
MEILISEARCH_INDEX_PREFIX=fechatter   # Index name prefix

# Monitoring
ENABLE_METRICS=true                  # Enable Prometheus metrics
METRICS_PORT=9090                   # Metrics endpoint port
ENABLE_TRACING=true                 # Enable distributed tracing
JAEGER_AGENT_ENDPOINT=localhost:6831 # Jaeger agent endpoint
```

### Feature Flags

```bash
# Features
ENABLE_CIRCUIT_BREAKER=true         # Circuit breaker for services
ENABLE_RATE_LIMITING=true           # API rate limiting
ENABLE_CACHE=true                   # Caching layer
ENABLE_SEARCH=true                  # Full-text search
ENABLE_WEBSOCKET=true               # WebSocket support
ENABLE_SSE=true                     # Server-Sent Events

# Performance
CONNECTION_LIMIT=1000               # Max concurrent connections
REQUEST_TIMEOUT_SECONDS=30          # Request timeout
CACHE_TTL_SECONDS=300              # Default cache TTL
CACHE_MAX_SIZE=10000               # Maximum cache entries
```

## 📄 Configuration Files

### Default Configuration (`config/default.yml`)

```yaml
server:
  host: 0.0.0.0
  port: 8080
  workers: 0  # 0 = number of CPU cores
  keep_alive: 75
  request_timeout: 30
  body_limit: 10485760  # 10MB

database:
  url: ${DATABASE_URL}
  max_connections: 100
  min_connections: 10
  acquire_timeout: 3
  idle_timeout: 600
  max_lifetime: 1800
  statement_cache_capacity: 100
  
redis:
  url: ${REDIS_URL}
  pool:
    max_size: 50
    min_idle: 10
    connection_timeout: 5
    idle_timeout: 300
    max_lifetime: 900

security:
  jwt:
    secret: ${JWT_SECRET}
    expiry_seconds: 3600
    algorithm: HS256
  bcrypt:
    cost: 12
  cors:
    allowed_origins:
      - http://localhost:3000
      - https://app.fechatter.com
    allowed_methods:
      - GET
      - POST
      - PUT
      - DELETE
      - OPTIONS
    allowed_headers:
      - Authorization
      - Content-Type
    max_age: 3600

cache:
  provider: redis  # redis or memory
  default_ttl: 300
  max_size: 10000
  eviction_policy: lru  # lru, lfu, or ttl

logging:
  level: info  # trace, debug, info, warn, error
  format: json  # json or pretty
  targets:
    - stdout
    - file
  file:
    path: logs/fechatter.log
    rotation: daily
    max_files: 7

features:
  circuit_breaker:
    enabled: true
    failure_threshold: 5
    recovery_timeout: 60
    half_open_requests: 3
  
  rate_limiting:
    enabled: true
    window_seconds: 60
    max_requests: 60
    burst_size: 10
    
  search:
    enabled: true
    provider: meilisearch
    auto_index: true
    batch_size: 100
```

### Environment-Specific Configuration

#### Development (`config/development.yml`)

```yaml
server:
  host: localhost
  port: 3000

database:
  url: postgresql://postgres:postgres@localhost:5432/fechatter_dev

redis:
  url: redis://localhost:6379/0

security:
  jwt:
    expiry_seconds: 86400  # 24 hours for development
  cors:
    allowed_origins:
      - http://localhost:*
      - http://127.0.0.1:*

logging:
  level: debug
  format: pretty

features:
  rate_limiting:
    enabled: false  # Disabled for development
```

#### Production (`config/production.yml`)

```yaml
server:
  workers: ${CPU_COUNT}
  keep_alive: 75
  
database:
  max_connections: 200
  statement_cache_capacity: 1000

redis:
  pool:
    max_size: 100

security:
  jwt:
    expiry_seconds: 3600  # 1 hour
  bcrypt:
    cost: 13  # Higher for production
  cors:
    allowed_origins:
      - https://app.fechatter.com
      - https://api.fechatter.com

cache:
  default_ttl: 600  # 10 minutes
  max_size: 100000  # 100k entries

logging:
  level: warn
  format: json
  
monitoring:
  metrics:
    enabled: true
    port: 9090
  tracing:
    enabled: true
    sampling_rate: 0.1  # 10% sampling
```

## 💾 Database Configuration

### Connection Pool Settings

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
    pub statement_cache_capacity: usize,
}

impl DatabaseConfig {
    pub fn create_pool(&self) -> PgPool {
        PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(Duration::from_secs(self.acquire_timeout))
            .idle_timeout(Duration::from_secs(self.idle_timeout))
            .max_lifetime(Duration::from_secs(self.max_lifetime))
            .statement_cache_capacity(self.statement_cache_capacity)
            .connect_lazy(&self.url)
            .expect("Failed to create pool")
    }
}
```

### PostgreSQL Tuning

```sql
-- Recommended PostgreSQL configuration for production
-- postgresql.conf

# Memory
shared_buffers = 4GB                # 25% of system memory
effective_cache_size = 12GB         # 75% of system memory
work_mem = 50MB                     # RAM per operation
maintenance_work_mem = 1GB          # RAM for maintenance

# Connections
max_connections = 200               # Match application pool size

# Write Performance
checkpoint_completion_target = 0.9
wal_buffers = 16MB
max_wal_size = 4GB
min_wal_size = 1GB

# Query Planner
random_page_cost = 1.1              # For SSD storage
effective_io_concurrency = 200      # For SSD storage

# Logging
log_min_duration_statement = 100    # Log queries > 100ms
log_checkpoints = on
log_connections = on
log_disconnections = on
log_lock_waits = on
log_temp_files = 0

# Statistics
track_activities = on
track_counts = on
track_io_timing = on
track_functions = all
```

## 🔴 Redis Configuration

### Redis Connection Settings

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub pool_max_size: u32,
    pub pool_min_idle: u32,
    pub connection_timeout: u64,
    pub command_timeout: u64,
}

impl RedisConfig {
    pub async fn create_client(&self) -> Result<RedisClient> {
        let client = redis::Client::open(self.url.as_str())?;
        let manager = RedisConnectionManager::new(client);
        
        let pool = Pool::builder()
            .max_size(self.pool_max_size)
            .min_idle(Some(self.pool_min_idle))
            .connection_timeout(Duration::from_secs(self.connection_timeout))
            .build(manager)
            .await?;
            
        Ok(RedisClient::new(pool))
    }
}
```

### Redis Server Configuration

```conf
# redis.conf

# Memory
maxmemory 4gb
maxmemory-policy allkeys-lru

# Persistence
save 900 1
save 300 10
save 60 10000
appendonly yes
appendfsync everysec

# Performance
tcp-keepalive 300
timeout 0
tcp-backlog 511

# Security
requirepass your-redis-password
rename-command FLUSHDB ""
rename-command FLUSHALL ""
rename-command CONFIG ""
```

## 🔒 Security Configuration

### JWT Configuration

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiry_seconds: i64,
    pub algorithm: Algorithm,
    pub issuer: String,
    pub audience: Vec<String>,
}

impl JwtConfig {
    pub fn validate(&self) -> Result<()> {
        if self.secret.len() < 32 {
            return Err("JWT secret must be at least 32 bytes");
        }
        Ok(())
    }
}
```

### CORS Configuration

```rust
use tower_http::cors::{CorsLayer, Any};

pub fn configure_cors(config: &CorsConfig) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(
            config.allowed_origins
                .iter()
                .map(|o| o.parse().unwrap())
                .collect::<Vec<_>>()
        )
        .allow_methods(
            config.allowed_methods
                .iter()
                .map(|m| m.parse().unwrap())
                .collect::<Vec<_>>()
        )
        .allow_headers(
            config.allowed_headers
                .iter()
                .map(|h| h.parse().unwrap())
                .collect::<Vec<_>>()
        )
        .max_age(Duration::from_secs(config.max_age))
}
```

### Rate Limiting Configuration

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub window_seconds: u64,
    pub max_requests: u32,
    pub burst_size: u32,
    pub key_by: KeyBy,  // ip, user, api_key
}

pub enum KeyBy {
    IpAddress,
    UserId,
    ApiKey,
}
```

## ⚡ Performance Tuning

### Application Performance Settings

```yaml
performance:
  # Connection limits
  max_connections: 10000
  connection_timeout: 30
  keep_alive_timeout: 75
  
  # Thread pool
  worker_threads: 0  # 0 = CPU cores
  blocking_threads: 512
  
  # Request handling
  request_timeout: 30
  body_limit: 10485760  # 10MB
  multipart_limit: 52428800  # 50MB
  
  # Database
  db_query_timeout: 5
  db_slow_query_threshold: 0.1  # 100ms
  
  # Cache
  cache_size: 100000
  cache_ttl: 600
  cache_compression: true
  
  # Circuit breaker
  circuit_breaker_threshold: 5
  circuit_breaker_timeout: 60
  circuit_breaker_half_open: 3
```

### System Tuning

```bash
# /etc/sysctl.conf

# Network
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.ip_local_port_range = 1024 65535
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 15
net.core.netdev_max_backlog = 65535

# File descriptors
fs.file-max = 1000000

# Memory
vm.swappiness = 10
vm.dirty_ratio = 15
vm.dirty_background_ratio = 5
```

## 🚩 Feature Flags

### Feature Flag Configuration

```yaml
features:
  # Core features
  authentication:
    enabled: true
    providers:
      - local
      - oauth2
      - saml
  
  messaging:
    enabled: true
    max_message_length: 4096
    typing_indicators: true
    read_receipts: true
    reactions: true
  
  search:
    enabled: true
    provider: meilisearch
    auto_index: true
    languages:
      - english
      - chinese
      - japanese
  
  # Beta features
  beta:
    voice_messages: false
    video_calls: false
    screen_sharing: false
    
  # Experimental
  experimental:
    ai_moderation: false
    end_to_end_encryption: false
```

### Runtime Feature Toggle

```rust
pub struct FeatureFlags {
    flags: Arc<RwLock<HashMap<String, bool>>>,
}

impl FeatureFlags {
    pub async fn is_enabled(&self, feature: &str) -> bool {
        self.flags.read().await
            .get(feature)
            .copied()
            .unwrap_or(false)
    }
    
    pub async fn set(&self, feature: &str, enabled: bool) {
        self.flags.write().await
            .insert(feature.to_string(), enabled);
    }
}
```

## 📝 Logging Configuration

### Log Levels and Targets

```yaml
logging:
  # Global log level
  level: info  # trace, debug, info, warn, error
  
  # Per-module configuration
  modules:
    fechatter_server: debug
    sqlx: warn
    tower_http: info
    hyper: warn
    tokio: warn
  
  # Output configuration
  outputs:
    - type: stdout
      format: json
      filter: "level >= info"
      
    - type: file
      path: logs/fechatter.log
      format: json
      rotation:
        size: 100MB
        count: 10
        compress: true
        
    - type: syslog
      host: localhost
      port: 514
      facility: local0
```

### Structured Logging

```rust
use tracing::{info, warn, error, instrument};
use serde_json::json;

#[instrument(skip(pool))]
pub async fn process_request(
    request_id: &str,
    user_id: i64,
    pool: &PgPool,
) -> Result<Response> {
    info!(
        request_id = %request_id,
        user_id = %user_id,
        "Processing request"
    );
    
    // Structured logging with JSON
    info!(
        target: "audit",
        event = "request_processed",
        request_id = %request_id,
        user_id = %user_id,
        metadata = %json!({
            "ip": "192.168.1.1",
            "user_agent": "Mozilla/5.0",
            "duration_ms": 45
        })
    );
}
```

## 🔧 Advanced Configuration

### Custom Configuration Provider

```rust
pub trait ConfigProvider: Send + Sync {
    async fn get(&self, key: &str) -> Option<String>;
    async fn set(&self, key: &str, value: String) -> Result<()>;
    async fn watch(&self, key: &str) -> ConfigWatcher;
}

// Implementation for different backends
pub struct EnvConfigProvider;
pub struct FileConfigProvider { path: PathBuf }
pub struct ConsulConfigProvider { client: ConsulClient }
pub struct EtcdConfigProvider { client: EtcdClient }
```

### Dynamic Configuration Reload

```rust
pub struct DynamicConfig {
    provider: Arc<dyn ConfigProvider>,
    cache: Arc<RwLock<HashMap<String, Value>>>,
    watchers: Arc<RwLock<HashMap<String, Vec<ConfigCallback>>>>,
}

impl DynamicConfig {
    pub async fn watch<F>(&self, key: &str, callback: F)
    where
        F: Fn(Value) + Send + Sync + 'static,
    {
        let mut watchers = self.watchers.write().await;
        watchers.entry(key.to_string())
            .or_default()
            .push(Box::new(callback));
            
        // Start watching for changes
        let watcher = self.provider.watch(key).await;
        tokio::spawn(async move {
            while let Some(value) = watcher.next().await {
                // Notify all callbacks
                for callback in &callbacks {
                    callback(value.clone());
                }
            }
        });
    }
}
```

### Configuration Validation

```rust
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct AppConfig {
    #[validate(range(min = 1024, max = 65535))]
    pub port: u16,
    
    #[validate(url)]
    pub database_url: String,
    
    #[validate(email)]
    pub admin_email: String,
    
    #[validate(length(min = 32))]
    pub jwt_secret: String,
    
    #[validate]
    pub redis: RedisConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config = Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::Environment::with_prefix("FECHATTER"))
            .build()?;
            
        let app_config: AppConfig = config.try_deserialize()?;
        app_config.validate()?;
        
        Ok(app_config)
    }
}
```

## 📋 Configuration Best Practices

1. **Use Environment Variables for Secrets**
   - Never commit secrets to version control
   - Use `.env` files for local development
   - Use secret management services in production

2. **Validate Configuration at Startup**
   - Check required fields are present
   - Validate data types and ranges
   - Test external service connections

3. **Provide Sensible Defaults**
   - Make the application work out-of-the-box
   - Document all configuration options
   - Use progressive disclosure

4. **Support Multiple Environments**
   - Development, staging, production
   - Environment-specific overrides
   - Feature flags for gradual rollout

5. **Monitor Configuration Changes**
   - Log configuration values (except secrets)
   - Track configuration drift
   - Alert on critical changes

---

**Version**: 1.0.0  
**Last Updated**: December 2024  
**Status**: Production Ready ✅ 