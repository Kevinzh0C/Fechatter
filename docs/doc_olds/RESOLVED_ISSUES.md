# Fechatter 构建问题解决方案总结

## 已解决的问题

### 1. Rust Edition 2024 支持问题
**问题**: Rust 1.82 不支持 edition = "2024"
**解决方案**: 
- 创建了使用 `rust:nightly-slim` 的 Dockerfile
- 在本地构建脚本中自动安装并使用 Rust nightly
- 保持项目使用 Edition 2024，无需降级

### 2. 磁盘空间问题
**问题**: Docker 构建时报错 "no space left on device"
**解决方案**:
- 在构建脚本中自动检查磁盘空间
- 自动清理 Podman 缓存和 Cargo 构建缓存
- 优化 Dockerfile 减少中间层大小

### 3. proc-macro 兼容性
**问题**: 第三方库的 proc-macro 在 musl 目标上不支持
**解决方案**:
- 使用 GNU libc 目标而不是 musl
- 使用 `debian:bookworm-slim` 作为运行时基础镜像

### 4. Docker Compose 服务名称不匹配
**问题**: `podman-compose build fechatter-server` 找不到服务
**解决方案**:
- 创建独立的 `docker-compose.build.yml`
- 在主 `docker-compose.yml` 中使用镜像而不是构建

## 预防的潜在问题

### 1. 依赖缺失
**预防措施**:
- 在 Dockerfile 中包含所有可能需要的构建依赖
- 包括 `git`, `build-essential` 等工具
- 为 analytics_server 的 protobuf 构建添加必要依赖

### 2. 构建缓存失效
**预防措施**:
- 分离依赖构建阶段，最大化缓存利用
- 使用 `CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse` 加速下载
- 清理不必要的构建产物减少镜像大小

### 3. 运行时配置问题
**预防措施**:
- 在运行时镜像中复制配置文件到 `/app/config/`
- 创建必要的目录结构
- 设置正确的用户权限

### 4. 健康检查
**预防措施**:
- 在 Dockerfile 中添加 HEALTHCHECK
- 在启动脚本中验证所有基础设施服务健康状态

### 5. 平台兼容性
**预防措施**:
- 为 macOS 用户提供 osascript 命令启动多个终端
- 兼容 Podman 和 Docker
- 检查 Podman machine 状态

## 使用建议

### 快速开始（推荐）
```bash
# 使用综合脚本
chmod +x ./scripts/build-and-run.sh
./scripts/build-and-run.sh

# 选择选项 6 进行完整设置
```

### Docker 构建（可选）
```bash
# 使用 nightly Dockerfile
podman build -f Dockerfile.nightly -t fechatter/all-services:latest .
```

### 本地开发
```bash
# 安装 Rust nightly
rustup toolchain install nightly
rustup override set nightly

# 构建
cargo build --release

# 运行
./scripts/run-local-dev.sh fechatter_server
```

## 故障排除

1. **磁盘空间不足**: 运行脚本的选项 1 清理空间
2. **服务启动失败**: 检查端口占用，运行 `./scripts/port-manager.sh`
3. **构建失败**: 确保使用 Rust nightly 版本
4. **配置错误**: 检查 `fixtures/` 目录下的配置文件

## 后续优化建议

1. 考虑使用 `cargo-chef` 进一步优化 Docker 构建
2. 设置 CI/CD 自动构建和推送镜像
3. 使用 `sccache` 加速本地构建
4. 考虑使用 distroless 镜像进一步减小运行时镜像大小 