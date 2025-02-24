/**
 * 简化版登录性能优化工具
 * Simplified Login Performance Utilities
 */

// 基本资源预加载
export const preloadCriticalResources = () => {
  const criticalResources = [
    '/src/main.js',
    '/src/stores/auth.js'
  ];

  criticalResources.forEach(resource => {
    const link = document.createElement('link');
    link.rel = 'modulepreload';
    link.href = resource;
    document.head.appendChild(link);
  });
};

// 简化版DNS预取
export const prefetchDNS = () => {
  const apiHost = import.meta.env.VITE_API_BASE_URL?.replace(/^https?:\/\//, '') || 'localhost';

  const link = document.createElement('link');
  link.rel = 'dns-prefetch';
  link.href = `//${apiHost}`;
  document.head.appendChild(link);
};

// 简化版性能监控
export const setupPerformanceMonitoring = () => {
  // 仅记录基本指标
  window.loginMetrics = {
    startTime: performance.now(),
    getLoginTime: () => performance.now() - window.loginMetrics.startTime
  };
};

// 主要优化函数 - 简化版
export const optimizeLoginPerformance = () => {
  try {
    preloadCriticalResources();
    prefetchDNS();
    setupPerformanceMonitoring();
  } catch (error) {
    if (import.meta.env.DEV) {
      console.warn('⚠️ 登录优化失败，继续使用标准流程:', error);
    }
  }
};

// 简化版性能分析
export const analyzeLoginPerformance = () => {
  if (import.meta.env.DEV) {
    const loadTime = performance.timing.loadEventEnd - performance.timing.navigationStart;
    console.log('Login performance:', loadTime + 'ms');
  }
};

export default {
  optimizeLoginPerformance,
  analyzeLoginPerformance,
  setupPerformanceMonitoring
}; 