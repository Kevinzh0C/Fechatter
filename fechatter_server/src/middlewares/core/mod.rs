//! # 核心中间件模块
//!
//! **职责**：提供最底层的中间件基础设施
//! - 适配器层: 包装fechatter_core功能，添加业务逻辑
//! - 兼容性层: 确保新旧系统无缝对接
//! - 基础设施: 提供核心的中间件引擎

// 适配器层 - 包装Core功能，添加业务逻辑
pub mod auth_adapter; // 认证适配器：Core认证 + 业务权限
pub mod request_adapter; // 请求适配器：Core请求ID + 性能监控
pub mod security_adapter; // 安全适配器：Core安全 + 业务防护

// 兼容性层 - 保持向后兼容
pub mod compatibility; // 兼容性适配层

// 基础中间件函数
pub mod middlewares; // 基础中间件实现
pub mod primitives; // 中间件原语

// 重新导出适配器的核心功能
pub use auth_adapter::{
  // 适配器类型
  AuthAdapter,
  business_auth_stack,
  chat_auth_middleware,

  // 中间件函数
  core_auth_middleware,
  enhanced_auth_middleware,
  full_auth_stack,

  // 便捷构造函数
  standard_auth_stack,
  workspace_auth_middleware,
};

pub use request_adapter::{
  // 工具类型
  RequestAdapterConfig,
  RequestInfo,
  RequestPerformanceMonitor,
  business_request_middleware,
  business_request_stack,
  // 中间件函数
  core_request_id_middleware,
  debug_request_middleware,

  debug_request_stack,

  enhanced_request_middleware,
  extract_request_info,
  full_request_stack,
  // 便捷构造函数
  standard_request_stack,
};

pub use security_adapter::{
  // 工具类型
  SecurityAdapterConfig,
  SecurityEvent,
  business_security_stack,
  // 中间件函数
  core_security_middleware,
  development_security_middleware,

  development_security_stack,

  enhanced_security_middleware,
  enterprise_security_middleware,
  enterprise_security_stack,
  // 便捷构造函数
  standard_security_stack,
};

// 重新导出兼容性层
pub use compatibility::{
  CompatibilityChecker,
  CompatibilityReport,

  MiddlewareSelector,
  // 兼容性工具
  MiddlewareStrategy,
  ServerTimeLayer,
  SetLayer,
  // 适配器中间件
  core_auth_middleware as compat_auth_middleware,
  core_request_id_middleware as compat_request_id_middleware,
  core_token_refresh_middleware,
  request_id_middleware,
  // Core中间件重新导出
  verify_token_middleware,
};

// =============================================================================
// 统一中间件注册表
// =============================================================================

/// 中间件注册表
///
/// **单一职责**：提供统一的中间件注册和查找机制
pub struct MiddlewareRegistry {
  auth_middlewares: std::collections::HashMap<&'static str, &'static str>,
  request_middlewares: std::collections::HashMap<&'static str, &'static str>,
  security_middlewares: std::collections::HashMap<&'static str, &'static str>,
}

impl Default for MiddlewareRegistry {
  fn default() -> Self {
    let mut registry = Self {
      auth_middlewares: std::collections::HashMap::new(),
      request_middlewares: std::collections::HashMap::new(),
      security_middlewares: std::collections::HashMap::new(),
    };

    // 注册认证中间件
    registry
      .auth_middlewares
      .insert("core", "core_auth_middleware");
    registry
      .auth_middlewares
      .insert("enhanced", "enhanced_auth_middleware");
    registry
      .auth_middlewares
      .insert("workspace", "workspace_auth_middleware");
    registry
      .auth_middlewares
      .insert("chat", "chat_auth_middleware");

    // 注册请求中间件
    registry
      .request_middlewares
      .insert("core", "core_request_id_middleware");
    registry
      .request_middlewares
      .insert("enhanced", "enhanced_request_middleware");
    registry
      .request_middlewares
      .insert("business", "business_request_middleware");
    registry
      .request_middlewares
      .insert("debug", "debug_request_middleware");

    // 注册安全中间件
    registry
      .security_middlewares
      .insert("core", "core_security_middleware");
    registry
      .security_middlewares
      .insert("enhanced", "enhanced_security_middleware");
    registry
      .security_middlewares
      .insert("enterprise", "enterprise_security_middleware");
    registry
      .security_middlewares
      .insert("development", "development_security_middleware");

    registry
  }
}

impl MiddlewareRegistry {
  /// 获取认证中间件名称
  pub fn get_auth_middleware(&self, level: &str) -> Option<&'static str> {
    self.auth_middlewares.get(level).copied()
  }

  /// 获取请求中间件名称
  pub fn get_request_middleware(&self, level: &str) -> Option<&'static str> {
    self.request_middlewares.get(level).copied()
  }

  /// 获取安全中间件名称
  pub fn get_security_middleware(&self, level: &str) -> Option<&'static str> {
    self.security_middlewares.get(level).copied()
  }

  /// 列出所有可用的认证中间件
  pub fn list_auth_middlewares(&self) -> Vec<&'static str> {
    self.auth_middlewares.keys().copied().collect()
  }

  /// 列出所有可用的请求中间件
  pub fn list_request_middlewares(&self) -> Vec<&'static str> {
    self.request_middlewares.keys().copied().collect()
  }

  /// 列出所有可用的安全中间件
  pub fn list_security_middlewares(&self) -> Vec<&'static str> {
    self.security_middlewares.keys().copied().collect()
  }
}

// =============================================================================
// 便捷配置函数
// =============================================================================

/// 创建标准中间件配置
///
/// **单一职责**：提供标准的中间件组合配置
pub fn standard_middleware_config() -> MiddlewareConfig {
  MiddlewareConfig {
    auth_level: "core",
    request_level: "core",
    security_level: "core",
    environment: "production",
  }
}

/// 创建业务中间件配置
///
/// **单一职责**：提供业务场景的中间件组合配置
pub fn business_middleware_config() -> MiddlewareConfig {
  MiddlewareConfig {
    auth_level: "enhanced",
    request_level: "enhanced",
    security_level: "enhanced",
    environment: "production",
  }
}

/// 创建企业级中间件配置
///
/// **单一职责**：提供企业级的完整中间件配置
pub fn enterprise_middleware_config() -> MiddlewareConfig {
  MiddlewareConfig {
    auth_level: "chat",
    request_level: "business",
    security_level: "enterprise",
    environment: "production",
  }
}

/// 创建开发环境中间件配置
///
/// **单一职责**：提供开发环境的中间件配置
pub fn development_middleware_config() -> MiddlewareConfig {
  MiddlewareConfig {
    auth_level: "enhanced",
    request_level: "debug",
    security_level: "development",
    environment: "development",
  }
}

/// 中间件配置结构体
///
/// **单一职责**：存储中间件配置参数
#[derive(Debug, Clone)]
pub struct MiddlewareConfig {
  pub auth_level: &'static str,
  pub request_level: &'static str,
  pub security_level: &'static str,
  pub environment: &'static str,
}

impl MiddlewareConfig {
  /// 根据环境自动选择配置
  pub fn for_environment(env: &str) -> Self {
    match env {
      "development" | "dev" => development_middleware_config(),
      "staging" | "test" => business_middleware_config(),
      "production" | "prod" => enterprise_middleware_config(),
      _ => standard_middleware_config(),
    }
  }

  /// 验证配置的有效性
  pub fn validate(&self) -> Result<(), String> {
    let registry = MiddlewareRegistry::default();

    if registry.get_auth_middleware(self.auth_level).is_none() {
      return Err(format!("无效的认证级别: {}", self.auth_level));
    }

    if registry
      .get_request_middleware(self.request_level)
      .is_none()
    {
      return Err(format!("无效的请求级别: {}", self.request_level));
    }

    if registry
      .get_security_middleware(self.security_level)
      .is_none()
    {
      return Err(format!("无效的安全级别: {}", self.security_level));
    }

    Ok(())
  }
}

// =============================================================================
// 中间件工厂
// =============================================================================

/// 中间件工厂
///
/// **单一职责**：根据配置创建中间件实例
pub struct MiddlewareFactory;

impl MiddlewareFactory {
  /// 根据配置创建中间件栈描述
  pub fn create_middleware_stack(config: &MiddlewareConfig) -> MiddlewareStack {
    let registry = MiddlewareRegistry::default();

    MiddlewareStack {
      auth_middleware: registry
        .get_auth_middleware(config.auth_level)
        .unwrap_or("core_auth_middleware"),
      request_middleware: registry
        .get_request_middleware(config.request_level)
        .unwrap_or("core_request_id_middleware"),
      security_middleware: registry
        .get_security_middleware(config.security_level)
        .unwrap_or("core_security_middleware"),
      environment: config.environment,
    }
  }

  /// 验证中间件栈的兼容性
  pub fn validate_stack(stack: &MiddlewareStack) -> Result<(), String> {
    // 检查中间件之间的兼容性
    if stack.auth_middleware == "chat_auth_middleware"
      && stack.security_middleware != "enterprise_security_middleware"
      && stack.security_middleware != "enhanced_security_middleware"
    {
      return Err("聊天认证需要增强或企业级安全中间件".to_string());
    }

    Ok(())
  }
}

/// 中间件栈描述
///
/// **单一职责**：描述完整的中间件栈配置
#[derive(Debug, Clone)]
pub struct MiddlewareStack {
  pub auth_middleware: &'static str,
  pub request_middleware: &'static str,
  pub security_middleware: &'static str,
  pub environment: &'static str,
}

// =============================================================================
// 测试模块
// =============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_middleware_registry() {
    let registry = MiddlewareRegistry::default();

    assert_eq!(
      registry.get_auth_middleware("core"),
      Some("core_auth_middleware")
    );
    assert_eq!(
      registry.get_request_middleware("enhanced"),
      Some("enhanced_request_middleware")
    );
    assert_eq!(
      registry.get_security_middleware("enterprise"),
      Some("enterprise_security_middleware")
    );

    assert!(registry.list_auth_middlewares().contains(&"core"));
    assert!(registry.list_request_middlewares().contains(&"business"));
    assert!(
      registry
        .list_security_middlewares()
        .contains(&"development")
    );
  }

  #[test]
  fn test_middleware_config() {
    let config = standard_middleware_config();
    assert_eq!(config.auth_level, "core");
    assert_eq!(config.request_level, "core");
    assert_eq!(config.security_level, "core");

    let business_config = business_middleware_config();
    assert_eq!(business_config.auth_level, "enhanced");

    let enterprise_config = enterprise_middleware_config();
    assert_eq!(enterprise_config.auth_level, "chat");
    assert_eq!(enterprise_config.security_level, "enterprise");
  }

  #[test]
  fn test_config_validation() {
    let valid_config = standard_middleware_config();
    assert!(valid_config.validate().is_ok());

    let invalid_config = MiddlewareConfig {
      auth_level: "invalid",
      request_level: "core",
      security_level: "core",
      environment: "production",
    };
    assert!(invalid_config.validate().is_err());
  }

  #[test]
  fn test_middleware_factory() {
    let config = business_middleware_config();
    let stack = MiddlewareFactory::create_middleware_stack(&config);

    assert_eq!(stack.auth_middleware, "enhanced_auth_middleware");
    assert_eq!(stack.request_middleware, "enhanced_request_middleware");
    assert_eq!(stack.security_middleware, "enhanced_security_middleware");

    assert!(MiddlewareFactory::validate_stack(&stack).is_ok());
  }

  #[test]
  fn test_environment_config() {
    let dev_config = MiddlewareConfig::for_environment("development");
    assert_eq!(dev_config.environment, "development");
    assert_eq!(dev_config.request_level, "debug");

    let prod_config = MiddlewareConfig::for_environment("production");
    assert_eq!(prod_config.environment, "production");
    assert_eq!(prod_config.security_level, "enterprise");
  }
}
