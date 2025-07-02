<div align="center">
  <img src="./assets/logo.svg" alt="Fechatter Logo" width="120" height="120">

<h1>Fechatter</h1>

<p>
    <strong>高效、企业级的 Rust 驱动实时聊天平台</strong>
  </p>

<p>
    <a href="README.md">🇺🇸 English</a> •
    <a href="README.zh-CN.md">🇨🇳 中文</a> •
    <a href="README.ja.md">🇯🇵 日本語</a>
  </p>

<p>
    <a href="https://github.com/Kevinzh0C/Fechatter/blob/master/LICENSE">
      <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License">
    </a>
    <a href="https://www.rust-lang.org/">
      <img src="https://img.shields.io/badge/built%20with-Rust-orange.svg" alt="Built with Rust">
    </a>
    <a href="https://github.com/Kevinzh0C/Fechatter/actions">
      <img src="https://github.com/Kevinzh0C/Fechatter/workflows/build/badge.svg" alt="构建状态">
    </a>
  </p>

  <p>
    <a href="https://fechatter-frontend.vercel.app">🚀 在线演示</a> •
    <a href="#-快速开始">快速开始</a> •
    <a href="#-功能特性">功能特性</a> •
    <a href="#-架构设计">架构设计</a> •
    <a href="./ROADMAP.md">🗺️ 路线图</a> •
    <a href="#-参与贡献">参与贡献</a>
  </p>
</div>

---

## ✨ 什么是 Fechatter？

Fechatter 是一个**现代化、功能完整的聊天平台**，结合了 **Rust 的高效性**与**企业级功能特性**，为用户提供卓越的消息传递体验。无论您是在构建团队协作工具还是社区平台，Fechatter 都能提供强大的基础架构和开箱即用的完整功能。

### 🎮 立即试用

<div align="center">
  <a href="https://fechatter-frontend.vercel.app" target="_blank">
    <img src="https://img.shields.io/badge/在线演示-立即体验%20Fechatter-brightgreen?style=for-the-badge&logo=vercel" alt="在线演示">
  </a>
</div>

## 🎯 功能特性

- **实时消息传递** - 使用服务端推送事件（SSE）实现即时消息收发
- **AI 聊天机器人** - 集成 ChatGPT 驱动的智能对话助手
- **消息搜索** - 基于 Meilisearch 的全文搜索功能
- **工作空间支持** - 在独立工作空间中组织聊天和用户
- **文件分享** - 在对话中上传和分享文件
- **JWT 身份验证** - 安全的基于令牌的身份验证系统
- **分析集成** - 使用 ClickHouse 和 Apache Superset 跟踪使用指标
- **微服务架构** - 模块化设计，不同功能采用独立服务

## 🚀 快速开始

### 快速启动

在 2 分钟内运行 Fechatter：

```bash
# 克隆仓库
git clone https://github.com/Kevinzh0C/fechatter.git
cd fechatter

# 复制环境配置
cp .env.example .env

# 启动所有服务
docker-compose up -d

# 在浏览器中打开
open http://localhost:5173
```

就是这样！🎉

### 系统要求

- Docker 20.10 或更高版本
- Docker Compose 2.0 或更高版本
- 最少 4GB 内存
- 端口 5173 可用

需要帮助？查看我们的[快速开始指南](./docs/QUICK_START.md)。

## 🏗️ 架构设计

Fechatter 采用**微服务架构**，专为可扩展性和可靠性而设计。

### 服务依赖矩阵

| 服务                 | PostgreSQL | Redis | ClickHouse | NATS | Meilisearch | OpenAI | S3 |
| -------------------- | :--------: | :---: | :--------: | :--: | :---------: | :----: | :-: |
| **聊天服务**   |     ✓     |  ✓  |     -     |  ✓  |     ✓     |   -   | ✓ |
| **通知服务**   |     -     |  ✓  |     -     |  ✓  |      -      |   -   | - |
| **机器人服务** |     -     |  ✓  |     -     |  -  |      -      |   ✓   | - |
| **分析服务**   |     -     |   -   |     ✓     |  ✓  |      -      |   -   | - |

### 📋 服务概览

| 服务                       | 端口 | 技术栈      | 用途                 |
| -------------------------- | ---- | ----------- | -------------------- |
| **API 网关**         | 8080 | Pingora     | 负载均衡、路由、认证 |
| **Fechatter 服务器** | 6688 | Axum, SQLx  | 核心聊天功能         |
| **通知服务器**       | 6687 | Tokio, SSE  | 实时通知             |
| **机器人服务器**     | 6686 | OpenAI SDK  | AI 聊天助手          |
| **分析服务器**       | 6690 | ClickHouse  | 事件跟踪和指标       |
| **前端界面**         | 3000 | Vue 3, Vite | 用户界面             |

在我们的[架构指南](./ARCHITECTURE.md)中了解更多。

## 💻 开发

### 本地开发

```bash
# 安装依赖
make setup

# 启动开发环境
make dev

# 运行测试
make test

# 构建生产版本
make build
```

### 技术栈

- **后端**: Rust, Axum, Tokio, SQLx
- **前端**: Vue 3, TypeScript, Vite
- **网关**: Pingora (Cloudflare 的代理框架)
- **数据库**: PostgreSQL, Redis
- **搜索**: Meilisearch
- **消息队列**: NATS JetStream
- **分析**: ClickHouse, Apache Superset
- **部署**: Docker, Kubernetes

## 📚 文档

### 入门指南

- [快速开始指南](./docs/QUICK_START.md) - 2 分钟内运行
- [安装指南](./docs/INSTALLATION.md) - 详细安装说明
- [配置说明](./fechatter_server/docs/CONFIGURATION.md) - 环境配置

### 核心文档

- [架构概览](./ARCHITECTURE.md) - 系统设计
- [API 参考](./fechatter_server/docs/API_REFERENCE.md) - REST API
- [开发指南](./fechatter_server/docs/DEVELOPMENT_GUIDE.md) - 开发环境设置
- [路线图](./ROADMAP.md) - 未来计划和里程碑

### 部署与运维

- [部署指南](./fechatter_server/docs/DEPLOYMENT_GUIDE.md) - 生产环境部署
- [性能指南](./fechatter_server/docs/PERFORMANCE_GUIDE.md) - 优化建议

## 🤝 参与贡献

我们欢迎您的参与！我们希望让 Fechatter 的贡献过程尽可能简单和透明。

查看我们的[贡献指南](./CONTRIBUTING.md)开始参与。

### 适合新手的问题

想要找个地方开始？查看我们的[适合新手的问题](https://github.com/Kevinzh0C/Fechatter/labels/good%20first%20issue)。

## 📄 许可证

Fechatter 采用 [MIT 许可证](./LICENSE)。

---

<div align="center">
  <p>
    <sub>由开发者用 ❤️ 为开发者构建</sub>
  </p>
  <p>
    <a href="https://github.com/Kevinzh0C/Fechatter">⭐ 在 GitHub 上给我们加星</a>
  </p>
</div>
