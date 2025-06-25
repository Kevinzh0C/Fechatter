import type { Router } from 'vue-router';
import NProgress from 'nprogress';

/**
 * 会话恢复工具
 */
export class SessionRecovery {
  private static readonly REDIRECT_KEY = 'redirectPath';
  private static readonly TIMESTAMP_KEY = 'redirectTimestamp';
  private static readonly MAX_AGE = 30 * 60 * 1000; // 30分钟

  /**
   * 保存重定向路径
   */
  static save(path: string): void {
    sessionStorage.setItem(this.REDIRECT_KEY, path);
    sessionStorage.setItem(this.TIMESTAMP_KEY, Date.now().toString());
  }

  /**
   * 获取并清除重定向路径
   */
  static restore(): string | null {
    const path = sessionStorage.getItem(this.REDIRECT_KEY);
    const timestamp = sessionStorage.getItem(this.TIMESTAMP_KEY);

    if (!path || !timestamp) {
      return null;
    }

    // 检查是否过期
    const age = Date.now() - parseInt(timestamp);
    if (age > this.MAX_AGE) {
      this.clear();
      return null;
    }

    this.clear();
    return path;
  }

  /**
   * 清除会话数据
   */
  static clear(): void {
    sessionStorage.removeItem(this.REDIRECT_KEY);
    sessionStorage.removeItem(this.TIMESTAMP_KEY);
  }

  /**
   * 执行恢复
   */
  static async execute(router: Router): Promise<void> {
    const redirectPath = this.restore();
    if (redirectPath && redirectPath !== '/login') {
      await router.replace(redirectPath);
    }
  }
}

/**
 * 性能监控工具
 */
export class RouterPerformance {
  private static navigationStart: number = 0;

  static startNavigation(): void {
    this.navigationStart = performance.now();
  }

  static endNavigation(routeName?: string): void {
    if (this.navigationStart > 0) {
      const duration = performance.now() - this.navigationStart;

      if (import.meta.env.VITE_DEBUG === 'true') {
        console.log(`Navigation to ${routeName} took ${duration.toFixed(2)}ms`);
      }

      // 上报性能数据到分析服务
      if (window.gtag) {
        window.gtag('event', 'page_view_timing', {
          custom_parameter: duration,
          page_title: routeName
        });
      }

      this.navigationStart = 0;
    }
  }
}

/**
 * 路由守卫完成处理
 */
export const setupRouterAfterHooks = (router: Router): void => {
  router.afterEach((to, _from, failure) => {
    // 完成进度条
    NProgress.done();

    // 记录性能
    RouterPerformance.endNavigation(to.name as string);

    // 调试日志
    if (import.meta.env.VITE_DEBUG === 'true') {
      const context = (to as any).navigationContext;
      if (context) {
        console.groupEnd();
      }
    }

    // 处理导航失败
    if (failure) {
      console.error('Navigation failed:', failure);

      // 上报错误
      if (window.gtag) {
        window.gtag('event', 'exception', {
          description: failure.message,
          fatal: false
        });
      }
    }
  });

  // 导航错误处理
  router.onError((error) => {
    console.error('Router error:', error);
    NProgress.done();

    // 重定向到错误页面
    router.push('/error/500');
  });
};