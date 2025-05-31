# User模块使用指南 - 直觉性API设计

## 🎯 **设计原则：最少惊讶，最大直觉**

开发者应该能通过**直觉联想**快速找到需要的功能，而不需要深入了解内部架构。

## 📦 **极简引用 - 一个import解决所有需求**

```rust
// ✅ 99%的情况下，只需要这一行
use crate::domains::user::{UserService, hash_password, verify_password};
```

## 🔍 **函数分类 - 符合开发者思维模型**

### **查询操作** - `find_*`, `exists_*`
```rust
let user_service = UserService::new(repository);

// 开发者直觉：想找用户，就用find_by_xxx
let user = user_service.find_by_id(UserId(1)).await?;
let user = user_service.find_by_email("alice@example.com").await?;
let exists = user_service.exists_by_email("bob@example.com").await?;
```

### **创建操作** - `create_*`, `authenticate`
```rust
// 开发者直觉：创建用户就是create_user
let new_user = user_service.create_user(&create_input).await?;

// 开发者直觉：认证就是authenticate  
let auth_user = user_service.authenticate(&signin_input).await?;
```

### **更新操作** - `change_*`, `update_*`
```rust
// 开发者直觉：改密码就是change_password
user_service.change_password(
    UserId(1), 
    "old_password", 
    "new_password"
).await?;

// 开发者直觉：更新资料就是update_profile
let updated = user_service.update_profile(UserId(1), "New Name").await?;
```

### **验证操作** - `validate_*`
```rust
// 开发者直觉：验证就是validate_xxx
user_service.validate_users_exist(&[UserId(1), UserId(2)]).await?;
```

## 🛠️ **工具函数 - 直接可用**

```rust
// 开发者直觉：hash_password就是哈希密码
let hash = hash_password("my_password")?;

// 开发者直觉：verify_password就是验证密码
let is_valid = verify_password("input_password", &stored_hash)?;
```

## 🚀 **快速上手示例**

```rust
use crate::domains::user::{UserService, hash_password};

async fn user_operations_example() -> Result<(), CoreError> {
    // 1. 创建服务 - 一行代码
    let user_service = UserService::new(repository);
    
    // 2. 创建用户 - 直觉命名
    let user = user_service.create_user(&CreateUser {
        fullname: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        password: "secure_password".to_string(),
        workspace: "acme".to_string(),
    }).await?;
    
    // 3. 查找用户 - 直觉命名
    let found = user_service.find_by_email("alice@example.com").await?;
    
    // 4. 修改密码 - 直觉命名
    user_service.change_password(
        user.id, 
        "secure_password", 
        "new_secure_password"
    ).await?;
    
    // 5. 更新资料 - 直觉命名
    let updated = user_service.update_profile(user.id, "Alice Smith").await?;
    
    Ok(())
}
```

## 💡 **认知负荷对比**

### ❌ 重构前 - 认知负荷高
```rust
// 开发者需要记住4个不同的路径和概念
use crate::domains::user::repository::UserRepositoryImpl;
use crate::domains::user::user_domain::{UserDomainService, UserDomainServiceImpl};
use crate::services::application::UserAppService;
use crate::domains::user::password::{hashed_password, verify_password};

// 需要理解：Repository、DomainService、AppService的区别
let repo = UserRepositoryImpl::new(pool);
let domain_service = UserDomainServiceImpl::new(Arc::new(repo), config);
let app_service = UserAppService::new(/* 复杂的依赖注入 */);
```

### ✅ 重构后 - 认知负荷低
```rust
// 开发者只需要知道一个概念：UserService
use crate::domains::user::UserService;

// 一行代码创建，内部复杂性完全隐藏
let user_service = UserService::new(repository);
```

## 🎯 **直觉性测试**

问：**开发者想要修改用户密码，会期望调用什么函数？**
- ✅ `change_password()` - 符合直觉
- ❌ `update_password_hash()` - 过于技术化
- ❌ `modify_user_credentials()` - 过于冗长

问：**开发者想要验证邮箱是否存在，会期望调用什么函数？**  
- ✅ `exists_by_email()` - 符合直觉
- ❌ `check_email_existence()` - 过于冗长
- ❌ `find_by_email().is_some()` - 需要额外逻辑

问：**开发者想要哈希密码，会期望调用什么函数？**
- ✅ `hash_password()` - 符合直觉  
- ❌ `hashed_password()` - 过去分词，不如动词直观
- ❌ `generate_password_hash()` - 过于冗长

## 🏆 **设计成功指标**

1. **新开发者5分钟内上手** - ✅ 通过统一的UserService
2. **90%场景只需一行import** - ✅ 通过facade模式
3. **函数名可以自我解释** - ✅ 通过直觉命名
4. **符合开发者心智模型** - ✅ 通过动词分类

这种设计让**架构复杂性对开发者透明**，真正实现了"简单易用"的目标！ 