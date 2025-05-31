# User模块架构修正说明

## 🚨 **问题发现：services.rs位置错误**

### ❌ 修正前的错误架构
```
domains/user/
├── entities.rs          # ✅ Domain Layer
├── user_domain.rs       # ✅ Domain Layer  
├── repository.rs        # ✅ Infrastructure Layer
├── password.rs          # ✅ Domain Layer
└── services.rs          # ❌ Application Layer (位置错误!)
```

**问题分析**:
- `services.rs` 中的 `UserService` 是应用服务，不是领域服务
- 应用服务负责**用例编排**，不应该放在 domain 层
- 违反了 DDD 分层架构原则

## ✅ **修正后的正确架构**

### 📁 Domain Layer (domains/user/)
```rust
// 只包含纯领域概念
domains/user/
├── entities.rs          # 聚合根、实体、值对象
├── user_domain.rs       # 领域服务 (UserDomainService)
├── repository.rs        # 仓储接口实现
└── password.rs          # 领域工具函数
```

### 📁 Application Layer (services/application/)
```rust
// 应用服务协调领域服务
services/application/
└── user_app_service.rs  # 用例编排 (UserAppService)
```

## 🔧 **修正操作**

1. **删除**: `domains/user/services.rs`
2. **创建**: `services/application/user_app_service.rs`
3. **更新**: `domains/user/mod.rs` (移除services导出)
4. **更新**: `services/application/mod.rs` (添加user_app_service导出)

## 🎯 **架构层次职责明确**

### Domain Layer - 业务逻辑核心
```rust
// 域服务 - 纯业务逻辑
impl UserDomainService for UserDomainServiceImpl {
  async fn change_password(&self, user_id, current, new) -> Result<()> {
    // 业务规则验证
    self.validate_password(new)?;
    // 领域逻辑执行
  }
}
```

### Application Layer - 用例协调
```rust
// 应用服务 - 用例编排
impl UserAppService {
  async fn change_password_use_case(&self, user_id, current, new) -> Result<()> {
    // 可添加：权限检查、审计日志、事件发布
    self.domain_service.change_password(user_id, current, new).await
  }
}
```

## 🚀 **架构优势**

1. **职责清晰**: 每层只负责自己的关注点
2. **依赖正确**: Application → Domain → Infrastructure
3. **可测试性**: 层次分离便于单元测试
4. **扩展性**: 可独立扩展各层功能

## 📝 **使用指南**

```rust
// ✅ 正确使用方式
use crate::services::application::UserAppService;

let app_service = UserAppService::new(repository);
app_service.change_password_use_case(user_id, current, new).await?;
```

这次修正确保了架构的**层次纯洁性**和**职责清晰性**！ 