# Fechatter Podman 部署指南

## 🚀 概述

本项目使用 Podman 和 rust-musl-cross 构建静态链接的二进制文件，生成轻量级容器镜像。

### 为什么选择 Podman？
- 无需 daemon，更安全
- 兼容 Docker 命令
- 原生支持 rootless 容器
- 更好的系统集成

### 为什么使用 musl？
- 生成完全静态链接的二进制文件
- 容器镜像更小（使用 Alpine Linux）
- 跨平台兼容性更好
- 减少运行时依赖

## 📋 前置要求

### 1. 安装 Podman
```bash
# macOS
brew install podman

# Linux (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install -y podman

# Linux (Fedora/RHEL)
sudo dnf install -y podman

# 初始化 Podman 机器（macOS）
podman machine init
podman machine start
```

### 2. 安装 podman-compose
```bash
pip3 install podman-compose
```

### 3. 配置 Rust 工具链
```bash
# 添加 musl target
rustup target add aarch64-unknown-linux-musl
```

## 🏗️ 构建镜像

### 快速构建所有服务
```bash
# 使用 Makefile
make build-podman

# 或使用构建脚本
chmod +x build-musl.sh
./build-musl.sh
```

### 构建单个服务
```bash
# 构建特定服务
make build-podman-gateway
make build-podman-fechatter-server
make build-podman-notify-server
make build-podman-bot-server
make build-podman-analytics-server
```

### 导出镜像
```bash
# 导出所有镜像到 tar 文件
./build-musl.sh --export

# 手动导出单个镜像
podman save -o gateway.tar fechatter/gateway:latest
```

## 🚀 运行服务

### 使用 podman-compose
```bash
# 启动所有服务
podman-compose up -d

# 查看服务状态
podman-compose ps

# 查看日志
podman-compose logs -f

# 停止服务
podman-compose down
```

### 使用 Makefile
```bash
# 启动所有容器
make run-podman

# 查看运行状态
make ps

# 查看日志
make logs-fechatter-server

# 停止所有容器
make kill-podman
```

## 🔧 配置说明

### 环境变量
创建 `.env` 文件：
```bash
cp env.example .env
```

必须配置的变量：
- `OPENAI_API_KEY` - OpenAI API 密钥
- `JWT_SECRET` - JWT 签名密钥
- `REDIS_PASSWORD` - Redis 密码
- `MEILI_MASTER_KEY` - Meilisearch 主密钥

### SELinux 注意事项（Linux）
Podman 在启用 SELinux 的系统上需要特殊的卷挂载标记：
- `:Z` - 私有卷（推荐）
- `:z` - 共享卷

配置文件已包含正确的 SELinux 标记。

### 网络配置
所有服务通过 `fechatter-net` 网络通信：
```bash
# 创建网络
podman network create fechatter-net

# 查看网络
podman network ls

# 检查网络详情
podman network inspect fechatter-net
```

## 📊 监控和管理

### 查看资源使用
```bash
# 实时监控
podman stats

# 快照查看
make stats
```

### 健康检查
```bash
# 检查所有服务健康状态
for service in fechatter notify bot analytics gateway; do
    echo "Checking $service..."
    podman healthcheck run $service
done
```

### 日志管理
```bash
# 查看特定服务日志
podman logs -f fechatter-server

# 导出日志
podman logs fechatter-server > fechatter-server.log

# 清理日志
podman logs --since 1h fechatter-server
```

## 🐛 故障排查

### 1. 容器无法启动
```bash
# 检查镜像是否构建成功
podman images | grep fechatter

# 查看容器状态
podman ps -a

# 查看详细错误
podman logs <container-name>
```

### 2. 网络连接问题
```bash
# 测试容器间连接
podman exec fechatter-server ping postgres

# 检查端口绑定
podman port <container-name>

# 检查防火墙规则（Linux）
sudo firewall-cmd --list-all
```

### 3. 权限问题
```bash
# 以 root 运行（不推荐）
sudo podman run ...

# 使用 rootless 模式（推荐）
podman unshare cat /proc/self/uid_map
```

### 4. 存储问题
```bash
# 查看存储使用
podman system df

# 清理未使用的资源
podman system prune -a

# 重置存储
podman system reset
```

## 🔄 迁移自 Docker

### 命令对照
| Docker | Podman |
|--------|--------|
| docker run | podman run |
| docker-compose | podman-compose |
| docker build | podman build |
| docker ps | podman ps |
| docker logs | podman logs |

### 主要差异
1. **镜像仓库前缀**：Podman 需要完整的仓库地址
   ```bash
   # Docker
   postgres:17
   
   # Podman
   docker.io/postgres:17
   ```

2. **卷挂载**：SELinux 标记
   ```bash
   # 添加 :Z 或 :z
   -v ./config.yml:/app/config.yml:ro,Z
   ```

3. **网络**：默认使用 slirp4netns
   ```bash
   # 创建网络时指定驱动
   podman network create --driver bridge fechatter-net
   ```

## 📚 高级用法

### 多架构构建
```bash
# 构建多架构镜像
podman build --platform linux/amd64,linux/arm64 -t fechatter/server:latest .
```

### Pod 管理
```bash
# 创建 Pod（类似 K8s）
podman pod create --name fechatter-pod -p 8080:8080

# 在 Pod 中运行容器
podman run -d --pod fechatter-pod fechatter/gateway:latest
```

### 使用 Kubernetes YAML
```bash
# 生成 Kubernetes YAML
podman generate kube fechatter-pod > fechatter-pod.yaml

# 从 YAML 运行
podman play kube fechatter-pod.yaml
```

## 🎯 最佳实践

1. **使用非 root 用户运行容器**
2. **定期更新基础镜像**
3. **使用健康检查**
4. **限制资源使用**
   ```bash
   podman run --memory 512m --cpus 1 ...
   ```
5. **使用密钥管理**
   ```bash
   podman secret create jwt_secret ./jwt_secret.txt
   ```

## 📖 参考资源

- [Podman 官方文档](https://docs.podman.io/)
- [rust-musl-cross](https://github.com/messense/rust-musl-cross)
- [Alpine Linux](https://alpinelinux.org/)
- [Podman vs Docker](https://podman.io/whatis.html) 