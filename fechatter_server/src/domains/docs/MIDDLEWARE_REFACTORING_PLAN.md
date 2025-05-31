# Middleware架构重构计划

## 🎯 重构目标
解决职责重叠和边界模糊问题，建立清晰的分层架构

## 🚨 当前问题
1. **构建器重复**：Core和Server都有builder，职责重叠
2. **认证授权混合**：认证和授权逻辑分散在多个中间件
3. **模块组织混乱**：缺少authorization模块导出
4. **废弃代码残留**：空的auth_middleware.rs文件

## ✅ 重构方案

### Phase 1: 清理和重组
- [x] 删除空的auth_middleware.rs
- [x] 修复mod.rs模块导出
- [ ] 重命名builder.rs为business_builder.rs
- [ ] 明确Core和Server构建器的职责边界

### Phase 2: 分层重构
```rust
// 新的分层架构
Core Layer (基础设施):
├── bearer_auth.rs      # 身份认证
├── token_refresh.rs    # 令牌管理  
├── request_id.rs       # 请求追踪
└── server_time.rs      # 时间中间件

Server Layer (业务逻辑):
├── authorization.rs    # 权限控制
├── workspace.rs        # 多租户上下文
├── chat.rs            # 资源访问控制
└── business_builder.rs # 业务中间件构建器
```

### Phase 3: 接口统一
```rust
// 统一的中间件接口
pub trait MiddlewareLayer {
  type Input;
  type Output;
  type Error;
  
  fn layer_name() -> &'static str;
  fn dependencies() -> Vec<&'static str>;
  fn apply(input: Self::Input) -> Result<Self::Output, Self::Error>;
}
```

### Phase 4: 性能优化
- [ ] 中间件执行顺序优化
- [ ] 缓存机制引入
- [ ] 错误处理统一
- [ ] 监控和日志完善

## 🔧 实施步骤

### Step 1: 重命名和清理
```bash
# 重命名构建器文件
mv src/middlewares/builder.rs src/middlewares/business_builder.rs

# 更新导入引用
grep -r "builder::" src/ --include="*.rs" | # 找到所有引用
sed -i 's/builder::/business_builder::/g'   # 批量替换
```

### Step 2: 职责分离
```rust
// Core层构建器：只负责基础中间件
impl<S, T> CoreBuilder<S, T> {
  pub fn with_auth(self) -> Self { /* 认证 */ }
  pub fn with_refresh(self) -> Self { /* 刷新 */ }
  pub fn with_tracing(self) -> Self { /* 追踪 */ }
}

// Server层构建器：负责业务中间件
impl<S, T> BusinessBuilder<S, T> {
  pub fn from_core(core: CoreBuilder<S, T>) -> Self { /* 继承Core */ }
  pub fn with_authorization(self) -> Self { /* 授权 */ }
  pub fn with_workspace(self) -> Self { /* 工作空间 */ }
  pub fn with_chat_access(self) -> Self { /* 聊天访问 */ }
}
```

### Step 3: 使用示例
```rust
// 重构后的使用方式
let app = Router::new()
  .route("/api/chats", get(list_chats))
  
  // Core层：基础设施中间件
  .core_middlewares(state.clone())
    .with_auth()
    .with_refresh()
    .with_tracing()
    
  // Server层：业务中间件  
  .business_middlewares()
    .with_authorization(&[Permission::ChatView])
    .with_workspace()
    .with_chat_access()
    .build();
```

## 📊 预期收益
1. **清晰的职责分离**：每个中间件职责单一
2. **更好的可测试性**：独立的中间件层便于单元测试
3. **更强的类型安全**：编译时检查中间件依赖
4. **更高的可维护性**：模块化设计便于扩展

## ⚠️ 风险评估
1. **API兼容性**：可能需要更新现有路由代码
2. **性能影响**：需要基准测试验证重构后性能
3. **测试覆盖**：需要更新相关的集成测试

## 📅 时间规划
- Week 1: Phase 1 + Phase 2 (重组和分层)
- Week 2: Phase 3 (接口统一)  
- Week 3: Phase 4 (性能优化)
- Week 4: 测试和文档完善 