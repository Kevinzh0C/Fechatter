# RFC: Fechatter - 实时聊天应用系统设计

## 1. 摘要

本文档描述了 Fechatter 实时聊天应用的设计和实现。Fechatter 是一个基于 Rust 的聊天平台，支持工作区内的不同类型聊天（单聊、群聊和频道）。系统由两个主要服务组成：`fechatter_server`（处理核心聊天功能）和 `notify_server`（管理通知）。本 RFC 详细说明了系统架构、数据模型、API 设计、安全考虑和性能优化策略。

## 2. 动机

现代工作环境需要高效、安全的实时通信工具。Fechatter 旨在提供一个类似 Slack 或 Discord 的聊天平台，但使用 Rust 语言实现以获得更好的性能和安全性。通过使用 Axum 框架和 WebSocket/SSE 技术，Fechatter 能够提供低延迟的实时消息传递，同时保持系统的可扩展性和可维护性。

## 3. 系统概述

### 3.1 核心功能

- 用户认证与注册
- 工作区创建与管理
- 多种聊天类型（点对点、群组、频道）
- 实时消息传递
- 聊天成员管理
- 文件共享

### 3.2 技术栈

- **编程语言**：Rust
- **Web 框架**：Axum
- **数据库交互**：SQLx
- **数据库**：PostgreSQL
- **认证**：JWT (JSON Web Tokens)
- **实时通信**：WebSocket/SSE (Server-Sent Events)
- **错误处理**：anyhow, thiserror
- **配置管理**：serde_yaml

## 4. 系统架构

### 4.1 组件结构

1. **fechatter_server**：主聊天应用服务器
   - 处理用户认证和注册
   - 管理聊天室、成员和消息
   - 实现工作区功能

2. **notify_server**：通知服务
   - 处理实时通知
   - 使用服务器发送事件 (SSE) 推送更新

3. **PostgreSQL 数据库**：存储所有应用数据
   - 用户、工作区、聊天和消息

### 4.2 架构图

```
+------------------+        +------------------+
|    客户端        |        |    客户端        |
+--------+---------+        +--------+---------+
         |                           |
         | HTTP API                  | SSE
         |                           |
+--------v---------+        +--------v---------+
|                  |        |                  |
| fechatter_server |<------>| notify_server   |
|                  |  事件   |                  |
+--------+---------+        +--------+---------+
         |                           |
         | 数据操作                   | 监听变更
         |                           |
+--------v---------+                 |
|                  |                 |
|   PostgreSQL     |<----------------+
|   数据库         |
+------------------+

+------------------+        +------------------+
|  认证流程        |        |  消息流程        |
+------------------+        +------------------+
|                  |        |                  |
| 1. 客户端发送    |        | 1. 客户端发送    |
|    登录请求      |        |    消息          |
|                  |        |                  |
| 2. 服务器验证    |        | 2. fechatter_    |
|    凭据          |        |    server 存储   |
|                  |        |    消息到数据库  |
| 3. 生成 JWT      |        |                  |
|    令牌          |        | 3. 数据库变更    |
|                  |        |    触发事件      |
| 4. 返回令牌      |        |                  |
|    给客户端      |        | 4. notify_server |
|                  |        |    监听到事件    |
| 5. 客户端使用    |        |                  |
|    令牌访问API   |        | 5. 通过 SSE 推送 |
|                  |        |    给相关客户端  |
+------------------+        +------------------+

+------------------------------------------+
|              数据模型关系                |
+------------------------------------------+
|                                          |
|    User <----> ChatMember <----> Chat    |
|      |                          |        |
|      |                          |        |
|      v                          v        |
|   Message <---------------> Workspace    |
|                                          |
+------------------------------------------+

+------------------------------------------+
|              系统组件结构                |
+------------------------------------------+
|                                          |
| fechatter_server:                        |
|   - AppState                             |
|   - 路由 (Router)                        |
|   - 处理器 (Handlers)                    |
|   - 模型 (Models)                        |
|   - 中间件 (Middlewares)                 |
|   - 工具 (Utils)                         |
|                                          |
| notify_server:                           |
|   - SSE 处理器                           |
|   - 事件监听器                           |
|                                          |
+------------------------------------------+
```

### 4.3 数据流

1. 客户端通过 HTTP API 与 `fechatter_server` 交互进行认证和数据操作
2. 客户端通过 SSE 连接到 `notify_server` 接收实时更新
3. `fechatter_server` 处理数据库操作并触发通知事件
4. `notify_server` 监听数据库事件并向相关客户端推送通知

## 5. 数据模型

### 5.1 核心实体

1. **User**：系统用户
   ```rust
   pub struct User {
       pub id: i64,
       pub email: String,
       pub password: String,
       pub fullname: String,
       pub avatar: Option<String>,
       pub status: UserStatus,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
   }
   
   pub enum UserStatus {
       Suspended,
       Active,
   }
   ```

2. **Chat**：聊天实体
   ```rust
   pub struct Chat {
       pub id: i64,
       pub name: String,
       pub type_: ChatType,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
   }
   
   pub enum ChatType {
       Single,
       Group,
       PrivateChannel,
       PublicChannel,
   }
   ```

3. **Message**：聊天消息
   ```rust
   pub struct Message {
       pub id: i64,
       pub chat_id: i64,
       pub sender_id: i64,
       pub content: String,
       pub created_at: DateTime<Utc>,
   }
   ```

4. **ChatMember**：聊天成员关系
   ```rust
   pub struct ChatMember {
       pub user_id: i64,
       pub chat_id: i64,
       pub role: String,
       pub created_at: DateTime<Utc>,
   }
   ```

5. **Workspace**：组织单位
   ```rust
   pub struct Workspace {
       pub id: i64,
       pub name: String,
       pub description: Option<String>,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
   }
   ```

### 5.2 数据库关系

- 用户与聊天：多对多关系（通过 ChatMember）
- 消息与聊天：一对多关系
- 用户与消息：一对多关系
- 工作区与聊天：一对多关系

### 5.3 数据库索引

- 用户表：为 email 创建唯一索引
- 消息表：为 chat_id 和 created_at 创建联合索引
- 聊天成员表：为 user_id 和 chat_id 创建联合索引

## 6. API 设计

### 6.1 认证 API

- `POST /api/auth/signup`：用户注册
  ```json
  // 请求
  {
    "fullname": "用户名",
    "email": "user@example.com",
    "password": "password123"
  }
  
  // 响应
  {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
  ```

- `POST /api/auth/signin`：用户登录
  ```json
  // 请求
  {
    "email": "user@example.com",
    "password": "password123"
  }
  
  // 响应
  {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
  ```

### 6.2 聊天管理 API

- `GET /api/chats`：获取聊天列表
  ```json
  // 响应
  [
    {
      "id": 1,
      "name": "聊天名称",
      "type": "Group",
      "created_at": "2023-01-01T00:00:00Z",
      "updated_at": "2023-01-01T00:00:00Z"
    }
  ]
  ```

- `POST /api/chats`：创建新聊天
  ```json
  // 请求
  {
    "name": "新聊天",
    "type": "Group",
    "members": [1, 2, 3]
  }
  
  // 响应
  {
    "id": 2,
    "name": "新聊天",
    "type": "Group",
    "created_at": "2023-01-02T00:00:00Z",
    "updated_at": "2023-01-02T00:00:00Z"
  }
  ```

- `PUT /api/chats/:id`：更新聊天
  ```json
  // 请求
  {
    "name": "更新的聊天名称"
  }
  
  // 响应
  {
    "id": 2,
    "name": "更新的聊天名称",
    "type": "Group",
    "created_at": "2023-01-02T00:00:00Z",
    "updated_at": "2023-01-03T00:00:00Z"
  }
  ```

- `DELETE /api/chats/:id`：删除聊天
  ```
  // 响应状态码：204 No Content
  ```

### 6.3 聊天成员 API

- `GET /api/chats/:id/members`：获取聊天成员
- `POST /api/chats/:id/members`：添加聊天成员
- `DELETE /api/chats/:id/members/:user_id`：移除聊天成员

### 6.4 消息 API

- `GET /api/chats/:id/messages`：获取聊天消息
- `POST /api/chats/:id/messages`：发送消息

### 6.5 工作区 API

- `GET /api/workspaces`：获取工作区列表
- `POST /api/workspaces`：创建工作区
- `PUT /api/workspaces/:id`：更新工作区
- `DELETE /api/workspaces/:id`：删除工作区

### 6.6 通知 API

- `GET /api/notifications/events`：SSE 通知端点

## 7. 安全设计

### 7.1 认证机制

- JWT 令牌认证
  ```rust
  pub struct TokenManager {
      secret: String,
      issuer: String,
  }
  
  impl TokenManager {
      pub fn generate_token(&self, user: &User) -> Result<String>;
      pub fn verify_token(&self, token: &str) -> Result<Claims>;
  }
  
  pub struct Claims {
      pub sub: i64,  // 用户 ID
      pub exp: u64,  // 过期时间
      pub iat: u64,  // 签发时间
      pub iss: String,  // 签发者
  }
  ```

- 密码使用 Argon2 哈希存储
  ```rust
  fn hashed_password(password: &str) -> Result<String> {
      let salt = SaltString::generate(&mut OsRng);
      let argon2 = Argon2::default();
      let password_hash = argon2
          .hash_password(password.as_bytes(), &salt)?
          .to_string();
      Ok(password_hash)
  }
  ```

- 令牌包含用户 ID 和过期时间

### 7.2 授权控制

- 中间件验证用户对资源的访问权限
  ```rust
  pub async fn verify_token_middleware<B>(
      State(state): State<AppState>,
      TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
      request: Request<B>,
      next: Next<B>,
  ) -> Result<Response, AppError> {
      // 验证令牌并提取用户信息
      // ...
  }
  ```

- 聊天成员关系验证
  ```rust
  async fn is_member_in_chat(
      user_id: i64,
      chat_id: i64,
      pool: &PgPool,
  ) -> Result<bool> {
      // 验证用户是否为聊天成员
      // ...
  }
  ```

- 工作区成员关系验证

## 8. 性能考虑

### 8.1 缓存策略

- 聊天列表缓存（30秒 TTL）
  ```rust
  pub struct AppState {
      config: AppConfig,
      token_manager: TokenManager,
      pool: PgPool,
      chat_list_cache: DashMap<i64, (Arc<Vec<ChatSidebar>>, Instant)>,
  }
  ```

- 消息分页加载
  ```rust
  async fn get_messages(
      chat_id: i64,
      limit: i64,
      before_id: Option<i64>,
      pool: &PgPool,
  ) -> Result<Vec<Message>> {
      // 分页加载消息
      // ...
  }
  ```

### 8.2 查询优化

- 为 email 创建索引以便快速查找用户
- 为 chat_id 和 created_at 创建联合索引以便高效查询消息

## 9. 配置管理

### 9.1 配置文件

- `app.yaml` 配置文件
  ```yaml
  server:
    port: 6688
  ```

- 环境变量支持
  ```rust
  fn load_config() -> Result<AppConfig> {
      // 尝试从多个位置加载配置
      match fs::File::open("app.yaml")
          .or_else(|_| fs::File::open("/etc/config/app.yaml"))
          .or_else(|_| {
              env::var("CHAT_CONFIG").map(|path| fs::File::open(path)).unwrap_or(Err(io::Error::new(
                  io::ErrorKind::NotFound,
                  "Config not found",
              )))
          }) {
          Ok(reader) => {
              let config: AppConfig = serde_yaml::from_reader(reader)?;
              Ok(config)
          }
          Err(_) => Err(anyhow!("Failed to load config")),
      }
  }
  ```

### 9.2 工作区设置

- 多个 crate 共享依赖
  ```toml
  [workspace]
  members = [
      "fechatter_server",
      "notify_server",
  ]
  
  [workspace.dependencies]
  tokio = { version = "1", features = ["full"] }
  axum = { version = "0.6", features = ["headers"] }
  sqlx = { version = "0.6", features = ["runtime-tokio-rustls", "postgres", "chrono", "json"] }
  # 其他共享依赖...
  ```

- 主服务器和通知服务器分离

## 10. 未来工作

### 10.1 短期计划

- 完善文件共享功能
- 添加用户状态管理（在线、离线、忙碌等）
- 实现消息已读状态

### 10.2 长期计划

- 添加端到端加密
- 实现消息搜索功能
- 支持语音和视频聊天
- 移动客户端开发

## 11. 结论

Fechatter 提供了一个基于 Rust 的高性能实时聊天平台，通过使用现代 Web 技术和最佳实践，实现了安全、可扩展的聊天服务。系统的模块化设计允许未来功能的灵活扩展，而 Rust 的性能和安全特性确保了系统的稳定性和可靠性。
