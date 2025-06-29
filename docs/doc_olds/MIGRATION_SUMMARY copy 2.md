# Bot Server 迁移到 Fechatter 总结

## 迁移概述

成功将 bot_server 从原项目迁移到 Fechatter 项目，并针对 Fechatter 的类型系统进行了适配。

## 主要变更

### 1. 依赖更新
- 将 `chat-core` 替换为 `fechatter_core`
- 更新包名从 `bot-server` 到 `bot_server`
- 使用工作空间依赖管理
- 修复 `swiftide-pgvector` 版本为 `0.1.1`

### 2. 类型系统适配
- 更新所有 ID 类型使用 Fechatter 的 newtype wrappers：
  - `UserId` - 用户 ID 包装器
  - `ChatId` - 聊天 ID 包装器  
  - `MessageId` - 消息 ID 包装器
- 修复类型不匹配问题，将 `HashSet<i64>` 更新为 `HashSet<UserId>`
- 保持数据库兼容性，ID 类型在数据库层面仍为 `i64`

### 3. 配置更新
- 更新数据库连接从 `chat` 到 `fechatter`
- 保持端口 6687 不变

### 4. 导入路径修正
- 所有 `use chat_core::*` 改为 `use fechatter_core::*`
- 添加必要的类型导入

## 文件结构

```
bot_server/
├── Cargo.toml              # 项目配置
├── bot.yml                 # 服务配置
├── MIGRATION_SUMMARY.md   # 本文档
└── src/
    ├── lib.rs              # 库模块
    ├── config.rs           # 配置管理
    ├── server.rs           # 服务器入口
    ├── indexer.rs          # 代码索引器
    └── notif.rs            # 通知处理
```

## 功能组件

### 1. 代码索引器 (indexer.rs)
- 使用 Swiftide 框架对 Rust 代码进行向量化索引
- 支持 OpenAI 嵌入模型 `text-embedding-3-small`
- 将代码块存储到 PostgreSQL + pgvector

### 2. 智能聊天机器人 (server.rs + notif.rs)
- 监听 PostgreSQL 通知 `chat_message_created`
- 自动响应直接消息给机器人用户
- 使用 RAG (检索增强生成) 进行智能回复：
  - 生成子问题
  - 向量检索相关代码
  - 总结和生成回答
- 支持 OpenAI GPT-4o-mini 模型

### 3. 配置管理 (config.rs)
- 支持多种配置文件位置：
  - `./bot.yml`
  - `/etc/config/bot.yml`
  - 环境变量 `BOT_CONFIG`

## 部署要求

### 1. 环境依赖
- PostgreSQL 数据库（需要启用 pgvector 扩展）
- OpenAI API 密钥
- Rust 工具链

### 2. 数据库设置
```sql
-- 启用 pgvector 扩展
CREATE EXTENSION IF NOT EXISTS vector;

-- 创建机器人用户
INSERT INTO users (fullname, email, password_hash, status, workspace_id, is_bot) 
VALUES ('Assistant Bot', 'bot@fechatter.com', NULL, 'Active', 1, TRUE);
```

### 3. 环境变量
```bash
export OPENAI_API_KEY="your-openai-api-key"
export BOT_CONFIG="./bot.yml"  # 可选
```

## 使用方法

### 1. 代码索引
```bash
# 运行一次以索引代码库
cargo run --bin indexer
```

### 2. 启动机器人服务
```bash
# 启动聊天机器人
cargo run --bin bot
```

### 3. 配置文件示例
```yaml
# bot.yml
server:
  port: 6687
  db_url: postgres://postgres:postgres@localhost:5432/fechatter
```

## 技术架构

### 1. 向量搜索
- 使用 Swiftide 框架进行文档处理
- 代码块大小：10-2048 字符
- 向量维度：1536 (OpenAI text-embedding-3-small)
- 存储：PostgreSQL + pgvector

### 2. 智能问答
- 查询管道：
  ```
  用户消息 → 子问题生成 → 向量嵌入 → 检索匹配 → 总结 → 生成回答
  ```
- 支持批量处理（批次大小：10）

### 3. 实时通知
- PostgreSQL LISTEN/NOTIFY 机制
- 只处理与机器人的直接消息
- 异步消息处理

## 后续优化建议

1. **功能增强**
   - 支持多机器人实例
   - 添加会话上下文记忆
   - 实现代码搜索缓存
   - 支持多种编程语言索引

2. **性能优化**
   - 添加向量搜索缓存
   - 实现增量代码索引
   - 优化查询管道

3. **监控告警**
   - 添加 Prometheus 指标
   - 实现健康检查端点
   - 添加错误告警

4. **部署优化**
   - 容器化部署
   - 支持多实例负载均衡
   - 配置热重载

## 测试

```bash
# 检查编译
cargo check

# 运行测试
cargo test

# 检查代码索引
cargo run --bin indexer

# 启动机器人服务
cargo run --bin bot
```

## 注意事项

1. 确保 PostgreSQL 已启用 pgvector 扩展
2. 需要有效的 OpenAI API 密钥
3. 机器人只响应直接消息（1对1聊天）
4. 索引器需要对代码目录有读取权限
5. 建议在生产环境中使用专用的向量数据库 