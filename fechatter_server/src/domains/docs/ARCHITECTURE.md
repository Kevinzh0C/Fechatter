# API & Handlers 架构规范

## 职责划分原则

### API层职责 (`api/`)
- **路由定义**: 定义URL路径和HTTP方法映射
- **中间件组装**: 应用认证、授权、限流等中间件
- **版本管理**: 管理不同API版本的路由
- **OpenAPI文档**: 生成API文档和规范

**函数命名规范**:
```rust
// 路由注册函数
pub fn register_v1_routes() -> Router<AppState>
pub fn register_websocket_routes() -> Router<AppState>

// 中间件组装函数  
pub fn auth_middleware_stack() -> MiddlewareStack
pub fn rate_limit_middleware(limit: u32) -> Middleware
```

### Handler层职责 (`handlers/`)
- **请求解析**: 提取路径参数、查询参数、请求体
- **输入验证**: 验证请求参数的合法性
- **权限检查**: 执行细粒度的权限验证
- **服务调用**: 调用Application Service执行业务逻辑
- **响应组装**: 将服务结果转换为HTTP响应
- **错误处理**: 捕获并转换业务错误为HTTP错误

**函数命名规范**:
```rust
// Handler函数 - 按照资源_动作_handler命名
pub async fn create_message_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(chat_id): Path<i64>,
    Json(payload): Json<CreateMessage>,
) -> Result<impl IntoResponse, AppError>

pub async fn list_messages_handler(...) -> Result<impl IntoResponse, AppError>
pub async fn update_message_handler(...) -> Result<impl IntoResponse, AppError>
pub async fn delete_message_handler(...) -> Result<impl IntoResponse, AppError>
```

### Service层职责 (`services/`)
- **业务逻辑**: 实现核心业务规则
- **事务管理**: 管理数据库事务边界
- **领域协调**: 协调多个领域服务
- **外部集成**: 调用外部服务和API

## 全局一致性规范

### 1. 目录结构标准化

```
src/
├── api/
│   ├── v1/
│   │   ├── mod.rs              // 路由注册
│   │   ├── auth.rs             // 认证路由
│   │   ├── messages.rs         // 消息路由
│   │   └── chats.rs            // 聊天路由
│   ├── v2/                     // 新版本
│   └── websocket/
├── handlers/
│   ├── v1/
│   │   ├── auth_handlers.rs    // 认证处理器
│   │   ├── message_handlers.rs // 消息处理器
│   │   └── chat_handlers.rs    // 聊天处理器
│   ├── v2/                     // 新版本处理器
│   └── common/                 // 跨版本通用
└── services/                   // 业务服务层
```

### 2. 函数签名标准化

#### Handler函数标准签名
```rust
pub async fn {resource}_{action}_handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,      // 可选: 需要认证时
    Path(params): Path<PathParams>,           // 可选: 路径参数
    Query(query): Query<QueryParams>,         // 可选: 查询参数  
    Json(payload): Json<RequestPayload>,      // 可选: 请求体
) -> Result<impl IntoResponse, AppError>
```

#### Service函数标准签名
```rust
pub async fn {action}_{resource}(
    &self,
    params: ActionParams,
    context: ServiceContext,
) -> Result<ActionResult, ServiceError>
```

### 3. 错误处理标准化

#### Handler层错误处理
```rust
pub async fn create_message_handler(...) -> Result<impl IntoResponse, AppError> {
    // 1. 参数验证
    validate_input(&payload)?;
    
    // 2. 权限检查
    authorize_action(&user, &chat_id, Permission::SendMessage)?;
    
    // 3. 服务调用
    let result = state.message_service()
        .create_message(payload, user.id)
        .await
        .map_err(AppError::from)?;
    
    // 4. 响应组装
    Ok((StatusCode::CREATED, Json(result)))
}
```

### 4. 版本管理策略

#### 向后兼容原则
- **新增字段**: 可选字段，默认值
- **字段重命名**: 保留旧字段，标记为deprecated  
- **行为变更**: 创建新版本API

#### 版本间代码复用
```rust
// 跨版本共享的核心逻辑
mod common {
    pub async fn extract_user_from_token(token: &str) -> Result<AuthUser, AppError> {
        // 通用认证逻辑
    }
    
    pub fn validate_message_content(content: &str) -> Result<(), AppError> {
        // 通用验证逻辑
    }
}

// v1处理器
mod v1 {
    use super::common::*;
    
    pub async fn create_message_handler(...) -> Result<impl IntoResponse, AppError> {
        validate_message_content(&payload.content)?;
        // v1特定逻辑
    }
}

// v2处理器  
mod v2 {
    use super::common::*;
    
    pub async fn create_message_handler(...) -> Result<impl IntoResponse, AppError> {
        validate_message_content(&payload.content)?;
        // v2特定逻辑
    }
}
```

### 5. 测试策略标准化

#### Handler测试模板
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_{action}_{resource}_success() {
        // 准备测试数据
        let state = create_test_state().await;
        let user = create_test_user();
        let payload = create_valid_payload();
        
        // 执行测试
        let result = {action}_{resource}_handler(
            State(state),
            Extension(user),
            Json(payload),
        ).await;
        
        // 验证结果
        assert!(result.is_ok());
    }
    
    #[tokio::test] 
    async fn test_{action}_{resource}_unauthorized() {
        // 测试未授权场景
    }
    
    #[tokio::test]
    async fn test_{action}_{resource}_invalid_input() {
        // 测试无效输入场景
    }
}
```

## 迁移指导

### 当前问题修复
1. **整合重复模块**: 合并 `handlers/messages/` 到 `handlers/v1/`
2. **规范命名**: 统一函数命名为 `{resource}_{action}_handler`
3. **分离关注点**: API层只管路由，Handler层处理逻辑

### 实施步骤
1. 创建新的标准化目录结构
2. 迁移现有处理器到新结构
3. 统一函数签名和错误处理
4. 添加全面的测试覆盖
5. 生成标准化的API文档

这样可以确保代码的可维护性、可扩展性和团队协作效率。 