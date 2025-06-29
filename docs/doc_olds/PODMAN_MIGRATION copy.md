# Podman 和 rust-musl-cross 迁移总结

## 🎯 迁移目标

将 Fechatter 项目从 Docker 迁移到 Podman，并使用 rust-musl-cross 构建静态链接的 aarch64 二进制文件。

## 📋 完成的更改

### 1. **Dockerfile 重构**
- 使用 `messense/rust-musl-cross:aarch64-musl` 作为构建镜像
- 生成静态链接的 aarch64 二进制文件
- 使用 Alpine Linux 作为运行时基础镜像
- 最小化镜像体积（预计每个服务 < 50MB）

### 2. **Makefile 更新**
- 所有 `docker` 命令替换为 `podman`
- 添加了 `build-podman` 和 `run-podman` 目标
- 支持构建网关服务
- 添加了 SELinux 卷挂载标记（`:Z`）
- 完整的基础设施服务支持

### 3. **新增 podman-compose.yml**
- 专为 Podman 优化的配置
- 使用完整的镜像仓库地址（如 `docker.io/`）
- 正确的 SELinux 标记
- 本地镜像使用 `localhost/` 前缀

### 4. **构建脚本**
- `build-musl.sh` - 自动化构建所有服务
- 支持导出镜像到 tar 文件
- 显示镜像大小信息

### 5. **文档**
- `PODMAN_GUIDE.md` - 完整的 Podman 使用指南
- 包含安装、配置、运行和故障排查

## 🚀 快速开始

```bash
# 1. 构建所有镜像
make build-podman
# 或
./build-musl.sh

# 2. 启动服务
make run-podman
# 或
podman-compose up -d

# 3. 查看状态
make ps
podman-compose ps
```

## 💡 主要优势

### 1. **更小的镜像体积**
- 静态链接二进制，无需运行时库
- Alpine Linux 基础镜像（~5MB）
- 每个服务镜像预计 < 50MB

### 2. **更好的安全性**
- Rootless 容器
- 无需 daemon 进程
- SELinux 支持

### 3. **跨平台兼容**
- aarch64 静态二进制
- 可在任何支持该架构的 Linux 上运行

## ⚠️ 注意事项

### 1. **macOS 用户**
需要初始化 Podman 机器：
```bash
podman machine init
podman machine start
```

### 2. **Linux 用户**
确保 SELinux 配置正确，卷挂载使用了 `:Z` 标记。

### 3. **环境变量**
确保设置了必要的环境变量：
- `OPENAI_API_KEY`
- `JWT_SECRET`
- `REDIS_PASSWORD`
- `MEILI_MASTER_KEY`

## 📝 与 Docker 的差异

| 特性 | Docker | Podman |
|------|--------|--------|
| Daemon | 需要 | 不需要 |
| Root 权限 | 通常需要 | 支持 rootless |
| 命令兼容性 | - | 99% 兼容 |
| 镜像格式 | OCI | OCI |
| Compose | docker-compose | podman-compose |

## 🔧 后续优化建议

1. **CI/CD 集成**
   - 更新 GitHub Actions 使用 Podman
   - 添加多架构构建支持

2. **生产部署**
   - 考虑使用 Podman pods
   - 集成 Kubernetes YAML 生成

3. **性能优化**
   - 使用 `--security-opt` 进一步优化
   - 考虑使用 crun 而非 runc

## 📚 相关文档

- [Podman 官方文档](https://docs.podman.io/)
- [rust-musl-cross 项目](https://github.com/messense/rust-musl-cross)
- [Alpine Linux](https://alpinelinux.org/)

## ✅ 迁移检查清单

- [x] Dockerfile 更新为使用 rust-musl-cross
- [x] Makefile 支持 Podman 命令
- [x] 创建 podman-compose.yml
- [x] 添加构建脚本
- [x] 编写使用文档
- [x] 测试构建流程
- [ ] 更新 CI/CD 配置
- [ ] 性能基准测试
- [ ] 生产环境验证 