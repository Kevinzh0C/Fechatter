/**
 * Authentication Recovery Strategies - 策略模式实现
 * 
 * 设计原则：
 * - Strategy Pattern: 可插拔的恢复策略
 * - Chain of Responsibility: 策略链执行
 * - Open/Closed Principle: 对扩展开放，对修改封闭
 */

import authService from '@/services/auth.service';

// 恢复策略基类
class RecoveryStrategy {
  constructor(name, priority = 0) {
    this.name = name;
    this.priority = priority;
  }

  // 检查策略是否适用
  async canApply(context) {
    throw new Error('canApply must be implemented');
  }

  // 🔄 执行恢复策略
  async execute(context) {
    throw new Error('execute must be implemented');
  }

  // 获取策略描述
  getDescription() {
    return `${this.name} (Priority: ${this.priority})`;
  }
}

// 🔄 Strategy 1: Token刷新策略
class TokenRefreshStrategy extends RecoveryStrategy {
  constructor() {
    super('Token Refresh', 10);
  }

  async canApply(context) {
    // 检查是否有refresh token且未过期
    const { refreshToken, tokenExpired } = context;
    return !!(refreshToken && tokenExpired && authService.refreshToken);
  }

  async execute(context) {
    try {
      if (import.meta.env.DEV) {
        console.log('🔄 Attempting token refresh...');
      }

      const refreshed = await authService.refreshToken(context.refreshToken);

      return {
        success: true,
        data: refreshed,
        method: 'token_refresh'
      };
    } catch (error) {
      return {
        success: false,
        error: error.message,
        method: 'token_refresh'
      };
    }
  }
}

// 🔄 Strategy 2: 存储恢复策略
class StorageRecoveryStrategy extends RecoveryStrategy {
  constructor() {
    super('Storage Recovery', 8);
  }

  async canApply(context) {
    // 检查localStorage中是否有有效的认证数据
    try {
      const stored = localStorage.getItem('auth');
      if (!stored) return false;

      const authData = JSON.parse(stored);
      const isRecent = authData.timestamp &&
        (Date.now() - authData.timestamp) < 24 * 60 * 60 * 1000;

      return !!(isRecent && authData.token && authData.user);
    } catch {
      return false;
    }
  }

  async execute(context) {
    try {
      const authData = JSON.parse(localStorage.getItem('auth'));

      if (import.meta.env.DEV) {
        console.log('🔄 Recovering from storage...');
      }

      return {
        success: true,
        data: {
          accessToken: authData.token,
          user: authData.user
        },
        method: 'storage_recovery'
      };
    } catch (error) {
      return {
        success: false,
        error: error.message,
        method: 'storage_recovery'
      };
    }
  }
}

// 🔄 Strategy 3: 静默重新认证策略
class SilentReauthStrategy extends RecoveryStrategy {
  constructor() {
    super('Silent Reauth', 6);
  }

  async canApply(context) {
    // 检查是否有记住的凭据
    const { hasRememberedCredentials } = context;
    return hasRememberedCredentials && authService.silentLogin;
  }

  async execute(context) {
    try {
      if (import.meta.env.DEV) {
        console.log('🔄 Attempting silent reauth...');
      }

      const result = await authService.silentLogin();

      return {
        success: true,
        data: result,
        method: 'silent_reauth'
      };
    } catch (error) {
      return {
        success: false,
        error: error.message,
        method: 'silent_reauth'
      };
    }
  }
}

// 🔄 Strategy 4: 优雅登出策略 (最后手段)
class GracefulLogoutStrategy extends RecoveryStrategy {
  constructor() {
    super('Graceful Logout', 1);
  }

  async canApply() {
    return true; // 总是可以执行
  }

  async execute(context) {
    if (import.meta.env.DEV) {
      console.log('🚪 Graceful logout - no recovery possible');
    }

    // 清理所有认证数据
    localStorage.removeItem('auth');
    sessionStorage.clear();

    return {
      success: true,
      data: { shouldRedirectToLogin: true },
      method: 'graceful_logout'
    };
  }
}

// 恢复策略管理器
export class AuthRecoveryManager {
  constructor() {
    this.strategies = [
      new TokenRefreshStrategy(),
      new StorageRecoveryStrategy(),
      new SilentReauthStrategy(),
      new GracefulLogoutStrategy()
    ];

    // 🔢 按优先级排序
    this.strategies.sort((a, b) => b.priority - a.priority);
  }

  // 注册新策略
  registerStrategy(strategy) {
    if (!(strategy instanceof RecoveryStrategy)) {
      throw new Error('Strategy must extend RecoveryStrategy');
    }

    this.strategies.push(strategy);
    this.strategies.sort((a, b) => b.priority - a.priority);
  }

  // 🔄 执行恢复流程
  async attemptRecovery(context = {}) {
    if (import.meta.env.DEV) {
      console.log('Starting recovery process with context:', context);
    }

    const results = [];

    for (const strategy of this.strategies) {
      try {
        const canApply = await strategy.canApply(context);

        if (!canApply) {
          results.push({
            strategy: strategy.name,
            status: 'skipped',
            reason: 'Strategy not applicable'
          });
          continue;
        }

        if (import.meta.env.DEV) {
          console.log(`🔄 Executing strategy: ${strategy.name}`);
        }

        const result = await strategy.execute(context);
        results.push({
          strategy: strategy.name,
          status: result.success ? 'success' : 'failed',
          data: result.data,
          error: result.error,
          method: result.method
        });

        // 如果策略成功，返回结果
        if (result.success) {
          return {
            success: true,
            strategy: strategy.name,
            method: result.method,
            data: result.data,
            results
          };
        }

      } catch (error) {
        results.push({
          strategy: strategy.name,
          status: 'error',
          error: error.message
        });

        console.error(`ERROR: Recovery strategy ${strategy.name} failed:`, error);
      }
    }

    // 🚫 所有策略都失败了
    return {
      success: false,
      error: 'All recovery strategies failed',
      results
    };
  }

  // 获取策略信息
  getStrategies() {
    return this.strategies.map(s => ({
      name: s.name,
      priority: s.priority,
      description: s.getDescription()
    }));
  }

  // 🧪 测试特定策略
  async testStrategy(strategyName, context) {
    const strategy = this.strategies.find(s => s.name === strategyName);

    if (!strategy) {
      throw new Error(`Strategy '${strategyName}' not found`);
    }

    const canApply = await strategy.canApply(context);
    if (!canApply) {
      return { canApply: false, reason: 'Strategy not applicable' };
    }

    const result = await strategy.execute(context);
    return { canApply: true, result };
  }
}

// 创建单例实例
export const authRecoveryManager = new AuthRecoveryManager();

// 🧪 全局调试接口
if (typeof window !== 'undefined') {
  window.authRecovery = {
    strategies: () => authRecoveryManager.getStrategies(),
    test: (strategyName, context) => authRecoveryManager.testStrategy(strategyName, context),
    recover: (context) => authRecoveryManager.attemptRecovery(context)
  };
}

export default authRecoveryManager; 