# 重构实施计划

## 当前问题分析

### 1. 重复和混乱的结构
```
❌ 当前混乱结构:
handlers/
├── messages/           # 混合版本处理器
│   ├── crud.rs        # 消息CRUD
│   ├── file.rs        # 文件处理
│   └── search.rs      # 搜索功能
├── v1/                # v1版本处理器
│   ├── auth.rs
│   ├── message_management.rs  # 与messages/重复
│   └── message_status.rs
└── health.rs          # 无版本处理器
```

### 2. 函数命名不统一
```rust
❌ 当前命名混乱:
// handlers/messages/crud.rs
pub async fn send_message_handler(...)     // send_开头
pub async fn list_messages_handler(...)    // list_开头

// handlers/v1/message_management.rs  
pub async fn edit_message_handler(...)     // edit_开头
pub async fn delete_message_handler(...)   // delete_开头

// handlers/v1/auth.rs
pub(crate) async fn signup_handler(...)    // 无前缀
pub(crate) async fn signin_handler(...)    // 无前缀
```

## 重构方案

### 1. 标准化目录结构

```
✅ 目标结构:
src/
├── api/
│   ├── v1/
│   │   ├── mod.rs              // 路由注册
│   │   ├── auth_routes.rs      // GET /auth/*, POST /auth/*
│   │   ├── message_routes.rs   // GET /messages/*, POST /messages/*
│   │   ├── chat_routes.rs      // GET /chats/*, POST /chats/*
│   │   └── file_routes.rs      // GET /files/*, POST /files/*
│   ├── websocket/
│   │   └── mod.rs              // WebSocket路由
│   └── common/
│       └── middleware.rs       // 通用中间件
├── handlers/
│   ├── v1/
│   │   ├── auth_handlers.rs    # create_user_handler, authenticate_user_handler
│   │   ├── message_handlers.rs # create_message_handler, list_messages_handler
│   │   ├── chat_handlers.rs    # create_chat_handler, list_chats_handler
│   │   └── file_handlers.rs    # upload_file_handler, download_file_handler
│   └── common/
│       ├── validation.rs       // 通用验证逻辑
│       ├── auth_utils.rs       // 认证工具函数
│       └── response_utils.rs   // 响应工具函数
└── services/                   // 保持现有结构
```

### 2. 函数命名标准化

#### 资源-动作命名模式
```rust
✅ 标准命名规范:

// 认证相关 (Resource: user/session)
pub async fn create_user_handler(...)        // POST /auth/signup
pub async fn authenticate_user_handler(...)  // POST /auth/signin  
pub async fn refresh_session_handler(...)    // POST /auth/refresh
pub async fn destroy_session_handler(...)    // DELETE /auth/logout

// 消息相关 (Resource: message)
pub async fn create_message_handler(...)     // POST /chats/{id}/messages
pub async fn list_messages_handler(...)      // GET /chats/{id}/messages
pub async fn get_message_handler(...)        // GET /messages/{id}
pub async fn update_message_handler(...)     // PUT /messages/{id}
pub async fn delete_message_handler(...)     // DELETE /messages/{id}

// 聊天相关 (Resource: chat)
pub async fn create_chat_handler(...)        // POST /chats
pub async fn list_chats_handler(...)         // GET /chats
pub async fn get_chat_handler(...)           // GET /chats/{id}
pub async fn update_chat_handler(...)        // PUT /chats/{id}
pub async fn delete_chat_handler(...)        // DELETE /chats/{id}

// 文件相关 (Resource: file)
pub async fn upload_file_handler(...)        // POST /files
pub async fn download_file_handler(...)      // GET /files/{id}
pub async fn delete_file_handler(...)        // DELETE /files/{id}
```

### 3. 重构实施步骤

#### Step 1: 创建新的标准化结构
```bash
# 创建新目录
mkdir -p src/api/v1
mkdir -p src/api/websocket  
mkdir -p src/api/common
mkdir -p src/handlers/v1
mkdir -p src/handlers/common
```

#### Step 2: 迁移消息处理器
```rust
// 新文件: src/handlers/v1/message_handlers.rs
use crate::{AppState, AppError, AuthUser};
use axum::{extract::*, response::IntoResponse, Json};
use fechatter_core::models::*;

/// 创建消息 - POST /chats/{chat_id}/messages
pub async fn create_message_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(chat_id): Path<i64>,
    Json(payload): Json<CreateMessage>,
) -> Result<impl IntoResponse, AppError> {
    // 从 handlers/messages/crud.rs 迁移逻辑
    // 重命名: send_message_handler -> create_message_handler
}

/// 获取消息列表 - GET /chats/{chat_id}/messages
pub async fn list_messages_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(chat_id): Path<i64>,
    Query(query): Query<ListMessages>,
) -> Result<impl IntoResponse, AppError> {
    // 保持现有逻辑，统一命名
}

/// 更新消息 - PUT /messages/{message_id}
pub async fn update_message_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(message_id): Path<i64>,
    Json(payload): Json<UpdateMessage>,
) -> Result<impl IntoResponse, AppError> {
    // 从 handlers/v1/message_management.rs 迁移
    // 重命名: edit_message_handler -> update_message_handler
}

/// 删除消息 - DELETE /messages/{message_id}
pub async fn delete_message_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(message_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    // 保持现有逻辑
}

/// 搜索消息 - GET /chats/{chat_id}/messages/search
pub async fn search_messages_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(chat_id): Path<i64>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
    // 从 handlers/messages/search.rs 迁移逻辑
}
```

#### Step 3: 迁移认证处理器
```rust
// 新文件: src/handlers/v1/auth_handlers.rs

/// 用户注册 - POST /auth/signup
pub async fn create_user_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    // 从 handlers/v1/auth.rs 迁移
    // 重命名: signup_handler -> create_user_handler
}

/// 用户登录 - POST /auth/signin
pub async fn authenticate_user_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    // 重命名: signin_handler -> authenticate_user_handler
}

/// 刷新令牌 - POST /auth/refresh
pub async fn refresh_session_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    cookies: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    // 重命名: refresh_token_handler -> refresh_session_handler
}

/// 登出 - DELETE /auth/logout  
pub async fn destroy_session_handler(
    State(state): State<AppState>,
    cookies: CookieJar,
    headers: HeaderMap,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    // 重命名: logout_handler -> destroy_session_handler
}
```

#### Step 4: 创建路由注册
```rust
// 新文件: src/api/v1/message_routes.rs
use crate::handlers::v1::message_handlers::*;
use axum::{routing::*, Router};

pub fn message_routes() -> Router<AppState> {
    Router::new()
        // 聊天消息
        .route("/chats/:chat_id/messages", post(create_message_handler))
        .route("/chats/:chat_id/messages", get(list_messages_handler))
        .route("/chats/:chat_id/messages/search", get(search_messages_handler))
        
        // 单个消息操作
        .route("/messages/:message_id", get(get_message_handler))
        .route("/messages/:message_id", put(update_message_handler))
        .route("/messages/:message_id", delete(delete_message_handler))
}

// 新文件: src/api/v1/auth_routes.rs
use crate::handlers::v1::auth_handlers::*;

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/signup", post(create_user_handler))
        .route("/auth/signin", post(authenticate_user_handler))
        .route("/auth/refresh", post(refresh_session_handler))
        .route("/auth/logout", delete(destroy_session_handler))
}

// 更新: src/api/v1/mod.rs
pub mod auth_routes;
pub mod message_routes;
pub mod chat_routes;
pub mod file_routes;

use axum::Router;
use crate::AppState;

pub fn v1_routes() -> Router<AppState> {
    Router::new()
        .merge(auth_routes::auth_routes())
        .merge(message_routes::message_routes())
        .merge(chat_routes::chat_routes())
        .merge(file_routes::file_routes())
        .nest("/v1", self::routes())
}
```

### 4. 验证重构效果

#### 重构前后对比
```
重构前:
❌ handlers/messages/crud.rs::send_message_handler
❌ handlers/v1/message_management.rs::edit_message_handler
❌ handlers/v1/auth.rs::signup_handler

重构后:  
✅ handlers/v1/message_handlers.rs::create_message_handler
✅ handlers/v1/message_handlers.rs::update_message_handler
✅ handlers/v1/auth_handlers.rs::create_user_handler
```

#### 全局一致性检查清单
- [ ] 所有handler函数遵循 `{action}_{resource}_handler` 命名
- [ ] 所有路由按资源分组 (`auth_routes`, `message_routes`)
- [ ] 版本隔离清晰 (`v1/`, `v2/`)
- [ ] HTTP方法与动作对应 (`POST->create`, `GET->list/get`, `PUT->update`, `DELETE->delete`)
- [ ] 权限检查一致性 (统一使用 `Extension<AuthUser>`)
- [ ] 错误处理一致性 (统一返回 `Result<impl IntoResponse, AppError>`)

### 5. 迁移时间线

```
Week 1: 创建新结构，迁移消息处理器
Week 2: 迁移认证和聊天处理器  
Week 3: 创建路由注册，更新导入
Week 4: 删除旧文件，更新测试
Week 5: 文档更新，团队培训
```

这个重构方案将消除当前的混乱结构，建立清晰的职责划分和命名一致性。 