# Fechatter gRPC 协议定义

## 概述

本目录包含 Fechatter 项目的所有 gRPC 协议定义文件，按服务功能组织。协议代码通过 `fechatter_protos` 库统一构建和分发。

## 目录结构

```
protos/
├── README.md                    # 本文档
├── Cargo.toml                   # 协议库配置
├── build.rs                     # 协议构建脚本
├── src/lib.rs                   # 协议库入口
├── fechatter/v1/               # Fechatter v1 API 协议
│   ├── core.proto              # 核心数据模型和类型
│   ├── auth.proto              # 认证和用户管理服务
│   ├── chat.proto              # 聊天和消息服务
│   ├── files.proto             # 文件上传下载服务
│   ├── notifications.proto     # 通知服务
│   ├── bot.proto               # 机器人和代码索引服务
│   └── analytics.proto         # 分析和指标服务
└── storage/                    # 数据库相关
    └── analytics.sql           # ClickHouse 分析数据库定义
```

## 协议说明

### 1. 核心协议 (core.proto)
- 定义基础数据类型：User, Chat, Message, FileInfo 等
- 通用枚举和错误类型
- 分页和响应结构

### 2. 认证服务 (auth.proto)
- **AuthService**: 用户注册、登录、登出、令牌管理
- **UserService**: 用户信息管理、搜索

主要功能：
- 用户注册和登录
- JWT 令牌管理
- 用户信息 CRUD 操作

### 3. 聊天服务 (chat.proto)
- **ChatService**: 聊天管理（创建、加入、离开）
- **MessageService**: 消息 CRUD 操作
- **MessageStreamService**: 实时消息流

主要功能：
- 聊天创建和管理
- 消息发送和接收
- 实时消息推送
- 消息已读状态管理

### 4. 文件服务 (files.proto)
- **FileService**: 文件上传、下载、管理

主要功能：
- 流式文件上传
- 分块文件下载
- 文件元数据管理

### 5. 通知服务 (notifications.proto)
- **NotificationService**: 通知推送和管理

主要功能：
- 实时通知推送
- 通知历史查询
- 通知已读状态管理

### 6. 机器人服务 (bot.proto)
- **BotService**: 机器人创建和管理
- **CodeIndexService**: 代码索引和搜索

主要功能：
- RAG 问答机器人
- 代码助手功能
- 向量搜索和检索

### 7. 分析服务 (analytics.proto)
- **AnalyticsService**: 事件收集和分析

主要功能：
- 用户行为事件追踪
- 系统性能监控
- 数据分析和报表

## 使用指南

### 在 Rust 项目中使用

1. **添加依赖**：
```toml
[dependencies]
fechatter_protos = { workspace = true }
```

2. **使用协议类型**：
```rust
use fechatter_protos::fechatter::v1::{
    User, Chat, Message,
    AuthServiceClient, ChatServiceClient,
};

// 创建客户端
let mut auth_client = AuthServiceClient::connect("http://localhost:50051").await?;

// 使用协议类型
let user = User {
    id: 1,
    fullname: "John Doe".to_string(),
    email: "john@example.com".to_string(),
    // ...
};
```

3. **实现服务**：
```rust
use fechatter_protos::fechatter::v1::{
    AuthService, AuthServiceServer,
    SignInRequest, SignInResponse,
};
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct MyAuthService;

#[tonic::async_trait]
impl AuthService for MyAuthService {
    async fn sign_in(
        &self,
        request: Request<SignInRequest>,
    ) -> Result<Response<SignInResponse>, Status> {
        // 实现登录逻辑
        todo!()
    }
}

// 启动服务器
let service = AuthServiceServer::new(MyAuthService::default());
Server::builder()
    .add_service(service)
    .serve(addr)
    .await?;
```

### 构建协议

协议代码会自动构建到 `target/proto/` 目录：

```bash
# 构建协议库
cargo build -p fechatter_protos

# 检查生成的代码
ls target/proto/
```

### 版本管理

- 协议版本通过 package 命名管理：`fechatter.v1`
- 向后兼容的更改可在同一版本内进行
- 破坏性更改需要创建新版本：`fechatter.v2`

### 版本兼容性规则

✅ **兼容的更改**：
- 添加新的服务方法
- 添加新的消息字段（使用新的字段号）
- 添加新的枚举值
- 重命名服务、方法、字段（仅在客户端）

❌ **不兼容的更改**：
- 删除或重新编号现有字段
- 更改字段类型
- 删除服务方法
- 更改方法签名

## 最佳实践

### 1. 字段设计
- 使用 protobuf 推荐的字段命名（snake_case）
- 为 optional 字段使用 `google.protobuf.*` 包装类型
- 预留字段号以供未来扩展

### 2. 错误处理
- 统一使用 `Error` 消息类型
- 包含错误代码、消息和详细信息
- 遵循 Google API 错误模型

### 3. 分页
- 使用标准的 `Pagination` 和 `PaginatedResponse`
- 支持基于游标的分页（适用于大数据集）

### 4. 流式处理
- 对实时数据使用服务器流
- 对大文件使用客户端流或双向流
- 实现适当的背压机制

## 开发环境设置

### 必需工具

```bash
# 安装 Protocol Buffers 编译器
# macOS
brew install protobuf

# Ubuntu/Debian
sudo apt install protobuf-compiler

# CentOS/RHEL
sudo yum install protobuf-compiler
```

### IDE 支持

推荐 IDE 插件：
- **VS Code**: Protocol Buffers extension
- **IntelliJ**: Protocol Buffers and gRPC plugin
- **Vim**: vim-protobuf

## 测试

### 使用 grpcurl 测试

```bash
# 列出服务
grpcurl -plaintext localhost:50051 list

# 调用方法
grpcurl -plaintext -d '{"email":"test@example.com","password":"password"}' \
  localhost:50051 fechatter.v1.AuthService/SignIn
```

### 使用 grpcui 图形化测试

```bash
# 安装 grpcui
go install github.com/fullstorydev/grpcui/cmd/grpcui@latest

# 启动 UI
grpcui -plaintext localhost:50051
```

## 部署注意事项

### 生产环境
- 启用 TLS 加密
- 配置适当的超时和重试策略
- 实现服务发现和负载均衡
- 监控 gRPC 指标

### 性能优化
- 使用连接池
- 启用 gRPC 压缩
- 实现客户端缓存
- 优化消息大小

## 更新历史

- **v1.0.0**: 初始版本，包含核心服务定义
- **v1.1.0**: 统一协议库，简化构建流程
- 更多版本历史请查看 git 提交记录 