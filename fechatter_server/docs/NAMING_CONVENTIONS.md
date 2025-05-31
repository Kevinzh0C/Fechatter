# Fechatter Domains 命名规则最佳实践

## 🎯 总体原则
- **一致性**: 所有模块使用统一的命名模式
- **简洁性**: 变量名简洁明了，避免冗余后缀
- **可读性**: 名称表达清晰的业务语义
- **层次性**: 通过命名体现架构层次

## 📋 命名规则详解

### 1. 结构体/枚举命名 (PascalCase)

#### Repository 层
```rust
// ✅ 正确
MessageRepository
ChatRepository
NotificationRepository

// ❌ 错误
FechatterUserRepository  // 不必要前缀
MessageRepo              // 简写不清晰
```

#### Domain Service 层
```rust
// ✅ 正确
MessageDomainService
ChatDomainService
NotificationDomainService

// ✅ 实现类
MessageDomainServiceImpl
ChatDomainServiceImpl
NotificationDomainServiceImpl
```

#### 事件类
```rust
// ✅ 正确 - [Domain][Action]
MessageSent
MessageEdited
MessageDeleted
ChatCreated
MemberAdded
MemberRemoved
```

#### 配置类
```rust
// ✅ 正确
MessageConfig
ChatConfig
NotificationConfig
```

### 2. 函数命名规范 (snake_case)

#### CRUD 操作
```rust
// ✅ 标准CRUD命名
create_message(input: &CreateMessage) -> Message
get_message_by_id(id: i64) -> Option<Message>
list_messages(params: &ListMessages) -> Vec<Message>
update_message(id: i64, input: &UpdateMessage) -> Message
delete_message(id: i64) -> Result<()>

// ✅ 查询操作
find_chat_by_id(id: i64)
exists_by_email(email: &str)
get_unread_count(user_id: i64)
```

#### 业务逻辑
```rust
// ✅ 权限检查 - can_[action]
can_edit(user_id: i64) -> bool
can_delete(user_id: i64) -> bool
can_modify(user_id: i64) -> bool

// ✅ 状态检查 - is_[state]
is_creator(user_id: i64) -> bool
is_member(user_id: i64) -> bool
is_public() -> bool

// ✅ 特征检查 - has_[feature]
has_attachments() -> bool
has_mentions() -> bool
has_description() -> bool

// ✅ 验证操作 - validate_[target]
validate_content(content: &str) -> Result<()>
validate_permissions(user_id: i64) -> Result<()>
```

### 3. 变量命名 (snake_case)

#### 简洁原则
```rust
// ✅ 正确 - 简洁明了
let message = repository.get_by_id(id)?;
let chat_id = message.chat_id;
let user_id = message.sender_id;

// ❌ 错误 - 过于verbose
let chat_id_val = i64::from(chat_id);
let user_id_val = i64::from(user_id);
let existing_message = query.fetch_one();  // 可简化为message
```

#### 临时变量
```rust
// ✅ 正确
let query = "SELECT * FROM messages WHERE chat_id = $1";
let result = sqlx::query(query).fetch_all(&pool).await?;
let count = messages.len();

// ✅ 类型转换
let chat_id = i64::from(chat_id);  // 而非chat_id_val
let user_id = i64::from(user_id);  // 而非user_id_val
```

### 4. 字段命名统一化

#### 时间字段
```rust
// ✅ 统一时间字段命名
pub created_at: DateTime<Utc>
pub updated_at: DateTime<Utc>
pub deleted_at: Option<DateTime<Utc>>

// ❌ 避免不一致
pub timestamp: DateTime<Utc>  // 应为created_at
```

#### ID字段
```rust
// ✅ 统一ID字段命名
pub id: i64
pub user_id: i64
pub chat_id: i64
pub message_id: i64
```

#### 配置字段
```rust
// ✅ 简洁的配置字段
pub max_length: usize     // 而非max_content_length
pub max_files: usize      // 而非max_file_count
pub cache_ttl: u64
pub enabled: bool         // 而非cache_enabled
```

### 5. 事件字段标准化

```rust
// ✅ 事件结构标准模式
pub struct MessageSent {
    pub message_id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,  // 统一使用created_at
}
```

## 🔧 重构实践

### 已完成的重命名
- `FechatterUserRepository` → `UserRepositoryImpl`
- `FechatterWorkspaceRepository` → `WorkspaceRepositoryImpl`
- `chat_id_val` → `chat_id`
- `user_id_val` → `user_id`
- `member_id_val` → `member_id`

### 命名冲突解决
当实现结构体与trait同名时，使用`Impl`后缀：
```rust
// ✅ 避免命名冲突
pub trait UserRepository { ... }
pub struct UserRepositoryImpl { ... }

impl UserRepository for UserRepositoryImpl { ... }
```

## 📊 质量检查清单

- [ ] 无不必要的前缀/后缀
- [ ] 函数名表达清晰的业务意图
- [ ] 变量名简洁且无冗余
- [ ] 时间字段统一使用`created_at`/`updated_at`
- [ ] CRUD操作使用标准动词
- [ ] 事件命名遵循`[Domain][Action]`模式
- [ ] 配置字段简洁明了

## 🚀 未来扩展

新增域模块时，严格遵循以上命名规则，确保整个domains层命名的一致性和可维护性。 