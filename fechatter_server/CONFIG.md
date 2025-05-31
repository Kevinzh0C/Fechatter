# Fechatter Server Configuration Management

## 🎯 Overview

Fechatter采用了安全的配置管理策略，**完全移除了硬编码的默认值**。这确保了：

- ✅ 没有硬编码的敏感信息（密钥、密码等）
- ✅ 不同环境使用不同的配置
- ✅ 配置从环境变量或配置文件加载
- ✅ 开发环境有明确的警告和临时配置

## 🚫 我们不再使用的反模式

```rust
// ❌ 硬编码的Default实现 - 已移除
impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            sk: "-----BEGIN PRIVATE KEY-----\nHARDCODED_KEY\n-----END PRIVATE KEY-----".to_string(),
            // ... 其他硬编码值
        }
    }
}
```

## ✅ 新的配置管理策略

### 1. 配置加载优先级

```rust
// 配置加载策略（按优先级）：
AppConfig::load_with_fallback()
    ↓
1. 配置文件: fechatter.toml
    ↓
2. 环境变量: FECHATTER_*
    ↓
3. 最小开发配置: minimal_dev_config() ⚠️
```

### 2. 配置方法说明

#### `AppConfig::from_file(path)`
- 从TOML配置文件加载
- 生产环境推荐方式
- 支持完整的配置选项

#### `AppConfig::from_env()`
- 从环境变量加载
- 容器化部署推荐
- 安全性高，无文件暴露

#### `AppConfig::minimal_dev_config()`
- ⚠️ **仅用于开发环境**
- 生成临时密钥对（每次重启都会变化）
- 有明确的警告提示
- **禁止在生产环境使用**

## 📝 配置方式

### 方式1: TOML配置文件（推荐生产环境）

```bash
# 复制示例配置文件
cp fechatter.example.toml fechatter.toml

# 编辑配置文件
vim fechatter.toml
```

配置文件示例：
```toml
[server]
port = 8080
db_url = "postgresql://user:pass@localhost/fechatter"

[auth]
pk = """-----BEGIN PUBLIC KEY-----
YOUR_ACTUAL_PUBLIC_KEY_HERE
-----END PUBLIC KEY-----"""

sk = """-----BEGIN PRIVATE KEY-----
YOUR_ACTUAL_PRIVATE_KEY_HERE
-----END PRIVATE KEY-----"""
```

### 方式2: 环境变量（推荐容器部署）

```bash
# 加载环境变量模板
source environment.example

# 或设置必要的环境变量
export DATABASE_URL="postgresql://user:pass@localhost/fechatter"
export FECHATTER_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----..."
export FECHATTER_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----..."
```

### 方式3: 开发环境快速启动

```rust
// 仅用于开发 - 会显示警告
let config = AppConfig::minimal_dev_config()?;
```

会输出警告：
```
⚠️  WARNING: Using temporary generated keys for development!
   These keys are NOT persistent and will change on restart.
   DO NOT use this in production!
```

## 🔐 安全密钥生成

### 生成RSA密钥对

```bash
# 生成私钥
openssl genrsa -out private.pem 2048

# 生成公钥
openssl rsa -in private.pem -pubout -out public.pem

# 查看密钥内容
cat private.pem
cat public.pem
```

### 安全存储建议

#### 开发环境
- 使用配置文件，但不要提交到版本控制
- 添加到 `.gitignore`：
  ```
  fechatter.toml
  *.pem
  .env
  ```

#### 生产环境
- 使用环境变量或密钥管理服务
- 考虑使用：
  - Docker Secrets
  - Kubernetes Secrets
  - AWS Secrets Manager
  - HashiCorp Vault

## 🛠️ 配置验证

### 检查配置加载

```rust
use fechatter_server::config::AppConfig;

// 测试配置加载
match AppConfig::load_with_fallback() {
    Ok(config) => println!("✅ 配置加载成功"),
    Err(e) => println!("❌ 配置加载失败: {}", e),
}
```

### 环境变量检查

```bash
# 检查必要的环境变量
echo "DATABASE_URL: $DATABASE_URL"
echo "FECHATTER_PUBLIC_KEY: ${FECHATTER_PUBLIC_KEY:0:50}..."
```

## 🚀 部署配置

### Docker部署

```dockerfile
# 环境变量方式
ENV DATABASE_URL="postgresql://..."
ENV FECHATTER_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----..."
ENV FECHATTER_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----..."
```

### Kubernetes部署

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: fechatter-config
data:
  database-url: <base64-encoded-url>
  public-key: <base64-encoded-public-key>
  private-key: <base64-encoded-private-key>
```

## ⚠️ 错误处理

### 常见错误

1. **缺少必需的环境变量**
   ```
   Error: Missing required environment variable: DATABASE_URL
   ```
   
2. **配置文件格式错误**
   ```
   Error: TOML parsing error: invalid key
   ```

3. **无效的配置值**
   ```
   Error: Invalid configuration value for FECHATTER_PORT: not_a_number
   ```

### 调试配置

```bash
# 启用调试日志
export RUST_LOG=debug

# 检查配置加载过程
cargo run 2>&1 | grep -i config
```

## 🏗️ 架构优势

### 之前的问题
- ❌ 硬编码敏感信息
- ❌ 无法区分环境
- ❌ 安全风险高
- ❌ 配置管理混乱

### 现在的优势
- ✅ 零硬编码敏感信息
- ✅ 环境特定配置
- ✅ 多种配置来源
- ✅ 明确的开发/生产分离
- ✅ 安全的默认行为

## 📚 最佳实践

1. **生产环境**：使用环境变量或专用配置文件
2. **开发环境**：使用配置文件或minimal_dev_config()
3. **测试环境**：使用minimal_dev_config()，注意清理
4. **CI/CD**：使用环境变量，敏感信息存储在secrets中
5. **配置验证**：启动时验证所有必需配置项
6. **密钥轮换**：定期更换JWT密钥对
7. **监控**：记录配置加载成功/失败

---

**记住：安全的配置管理是生产系统的基础。绝不要在代码中硬编码敏感信息！** 