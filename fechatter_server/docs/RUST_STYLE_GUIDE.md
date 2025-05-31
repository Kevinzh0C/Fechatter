# Rust风格的API设计 - 真实工程实践

## 🎯 **真实优秀工程的做法**

### ❌ **不使用Facade模式**
```rust
// Java风格 - Rust生态很少这样做
pub struct UserFacade {
    // 复杂的内部状态
}
```

### ✅ **Rust生态的标准做法**

## 📦 **1. 智能重导出 - tokio风格**

```rust
// 参考：tokio/src/lib.rs
pub use tokio::fs;
pub use tokio::net; 
pub use tokio::sync;

// 我们的实现：
pub use repository::UserRepositoryImpl;
pub use user_domain::{UserDomainService, UserDomainServiceImpl};
pub use password::{hashed_password as hash_password, verify_password};
```

## 🏗️ **2. 模块构造器 - serde风格**

```rust
// 参考：serde的模块组织
use serde::{Deserialize, Serialize}; // 一行搞定

// 我们的实现：
use crate::domains::user::{UserModule, hash_password, verify_password};

let user_module = UserModule::new(pool);
// 直接访问内部组件
user_module.repository.find_by_id(id).await?;
user_module.domain_service.change_password(id, old, new).await?;
```

## 🔧 **3. 便捷函数 - std风格**

```rust
// 参考：std::fs::read_to_string()
let content = std::fs::read_to_string("file.txt")?;

// 我们的实现：
let hash = user::hash_password_quick("password")?;
let valid = user::verify_password_quick("input", &hash)?;
```

## 💡 **真实使用场景**

### **场景1：Handler中使用**
```rust
use crate::domains::user::UserModule;

async fn create_user_handler(state: AppState) -> Result<Json<User>, AppError> {
    let user_module = UserModule::new(state.pool());
    
    // 直接使用，无需复杂的依赖注入
    let user = user_module.repository.create(&input).await?;
    Ok(Json(user))
}
```

### **场景2：Service中使用**
```rust
use crate::domains::user::{UserModule, hash_password};

pub struct AuthService {
    user_module: UserModule,
}

impl AuthService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            user_module: UserModule::new(pool),
        }
    }
    
    pub async fn register(&self, email: &str, password: &str) -> Result<User> {
        let hash = hash_password(password)?;
        // 使用具体的组件
        self.user_module.repository.create(&CreateUser { 
            email: email.to_string(),
            password_hash: hash,
            // ...
        }).await
    }
}
```

### **场景3：测试中使用**
```rust
#[tokio::test]
async fn test_user_operations() {
    let pool = setup_test_db().await;
    let user_module = UserModule::new(pool);
    
    // 清晰的测试代码
    let user = user_module.repository.create(&test_input).await?;
    user_module.domain_service.change_password(user.id, "old", "new").await?;
}
```

## 🏆 **为什么这样更好？**

### **1. 符合Rust生态习惯**
- 重导出而非封装
- 组合而非继承
- 明确而非隐藏

### **2. 零成本抽象**
- 编译时优化
- 无运行时开销
- 直接访问底层组件

### **3. 灵活性更高**
```rust
// 可以直接访问任何层次
user_module.repository.find_by_id(id).await?;           // 数据层
user_module.domain_service.change_password(...).await?; // 业务层

// 也可以使用便捷函数
hash_password_quick("password")?;                       // 工具层
```

### **4. 测试友好**
```rust
// 可以单独测试任何组件
let repo = UserRepositoryImpl::new(pool);
let domain = UserDomainServiceImpl::new(Arc::new(repo), config);

// 或者测试整个模块
let module = UserModule::new(pool);
```

## 📚 **参考的优秀Rust项目**

1. **tokio** - 模块重导出
2. **serde** - 智能API设计  
3. **sqlx** - 便捷构造器
4. **tracing** - 顶层便捷函数
5. **reqwest** - 模块化组织

## 🎯 **设计原则总结**

1. **重导出 > Facade** - 让用户直接访问需要的类型
2. **组合 > 封装** - 提供构造器而非黑盒
3. **明确 > 隐藏** - 让架构对用户可见
4. **便捷 > 复杂** - 提供简单的默认路径

这才是**真正的Rust风格**！ 