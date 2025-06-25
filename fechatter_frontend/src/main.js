import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import authPlugin from "./plugins/auth";
import errorMonitor from "./utils/errorMonitor";
import healthCheck, { startHealthMonitoring } from "./utils/healthCheck";
import { initializeConfig } from "@/utils/configLoader";
import { updateApiInstance } from "@/services/api";
import { analytics } from "./lib/analytics-protobuf";
import { errorHandler } from "./utils/errorHandler";
import logCleaner from "./utils/logCleaner";

// Import development optimizer
import developmentOptimizer from "./utils/developmentOptimizer";

// Import token manager and make it globally available
import tokenManager from '@/services/tokenManager';
window.tokenManager = tokenManager;

const app = createApp(App);
const pinia = createPinia();

// 配置Vue错误处理器
app.config.errorHandler = (err, instance, info) => {
  errorMonitor.logError(err, {
    component: instance?.$options.name || 'Unknown',
    componentInfo: info,
    type: 'VUE_ERROR'
  });

  // 发送错误到analytics
  analytics.trackError(err, `Vue Error in ${instance?.$options.name || 'Unknown'} - ${info}`, 'VUE_ERROR');
};

// 配置Vue警告处理器（开发环境）
if (import.meta.env.DEV) {
  app.config.warnHandler = (msg, instance, trace) => {
    errorMonitor.logWarning(msg, {
      component: instance?.$options.name || 'Unknown',
      trace
    });
  };
}

// 注册插件（顺序很重要）
app.use(pinia);  // Pinia必须先注册
app.use(router);
app.use(authPlugin);  // Auth插件在router之后

// 异步初始化应用
async function initializeApp() {
  try {
    // 1. 初始化错误处理器
    errorHandler.initialize();

    // 注册认证错误处理器
    errorHandler.registerHandler('auth', (error, details) => {
      console.log('Auth error handled, redirecting to login...');
      // 认证错误已由 errorHandler.handleAuthError() 处理
    });

    // 2. 初始化配置系统
    await initializeConfig();

    // 3. 更新API实例使用新配置
    updateApiInstance();

    // 4. 挂载应用
    app.mount('#app');

    // 5. 跟踪应用启动事件
    analytics.trackAppStart();

  } catch (error) {
    console.error('❌ Failed to initialize Fechatter Frontend:', error);

    // 降级处理：使用默认配置挂载应用
    console.warn('⚠️ Falling back to default configuration');
    app.mount('#app');
  }
}

// 启动应用
initializeApp();

// 全局暴露router供store使用
window.$router = router;

// ===== 健康检查所需的全局变量（所有环境） =====
window.app = app;
window.pinia = pinia;
window.$router = router; // 为健康检查提供router访问

// 启动健康监控
try {
  // 开发环境：使用优化的间隔
  const healthCheckInterval = import.meta.env.DEV
    ? developmentOptimizer.getConfig('healthCheck').intervalMs
    : 60000; // 生产环境1分钟

  startHealthMonitoring(healthCheckInterval);
  console.log(`✅ Health monitoring started with ${healthCheckInterval / 60000}min intervals`);
} catch (error) {
  console.warn('⚠️ Failed to start health monitoring:', error.message);
}

// 开发环境：显示优化状态
if (import.meta.env.DEV) {
  console.log('🔧 Development optimizations active');
  console.log('💡 Use window.dev for development helpers');

  // Import health check helper in development
  import('./utils/devHealthCheckHelper.js').then(() => {
    console.log('🏥 Health check helper loaded - use window.healthHelper');
  }).catch(err => {
    console.warn('Failed to load health check helper:', err);
  });

  // Import SSE cleanup utility in development
  import('./utils/sseCleanup.js').then(() => {
    console.log('🧹 SSE cleanup utility loaded - use window.cleanupSSE()');
  }).catch(err => {
    console.warn('Failed to load SSE cleanup utility:', err);
  });
}

// 生产环境错误上报（可以集成到你的错误追踪服务）
if (import.meta.env.PROD) {
  errorMonitor.subscribe((errorEntry) => {
    // 这里可以集成Sentry、LogRocket等错误追踪服务
    if (errorEntry.type === 'ERROR' || errorEntry.critical) {
      console.error('Production error:', errorEntry);
      // 例如: Sentry.captureException(errorEntry.error);
    }
  });
}

// 页面卸载时跟踪应用退出事件
window.addEventListener('beforeunload', () => {
  analytics.trackAppExit(1); // exit code 1 for normal exit
  analytics.destroy(); // 清理资源并发送剩余事件
});
