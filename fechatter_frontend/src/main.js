import { createApp } from "vue";
import { createPinia } from "pinia";

// Import unified error handler FIRST - before any other error handling
import unifiedErrorHandler from "./utils/unifiedErrorHandler";

// Import content script error suppressor after unified handler
import contentScriptSuppressor from "./utils/contentScriptErrorSuppressor";

// Import quick navigation fix
import { applyAllFixes } from "./utils/quickNavigationFix";

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

// Import extension conflict handler for production-level error isolation
import extensionConflictHandler from "./utils/extensionConflictHandler";

// Import aggressive extension blocker
import aggressiveExtensionBlocker from "./utils/aggressiveExtensionBlocker";

// Import chat optimizations
import { injectOptimizationStyles } from "./utils/chatOptimizations";

// Import token manager and make it globally available
import tokenManager from '@/services/tokenManager';
window.tokenManager = tokenManager;

// Import real-time communication service - use minimal version
import minimalSSE from '@/services/sse-minimal';

// Import SSE connection manager
import sseConnectionManager from '@/utils/sseConnectionManager';

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
    // 1. Initialize error handling and extension conflict protection
    errorHandler.initialize();
    extensionConflictHandler.initialize();

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

    // 5. Apply navigation fixes
    setTimeout(() => {
      applyAllFixes();
      console.log('✨ Navigation fixes applied');
    }, 1000);

    // 6. Inject chat optimization styles
    injectOptimizationStyles();
    console.log('✨ Chat optimizations applied');

    // 7. Initialize SSE connection manager
    const { useAuthStore } = await import('@/stores/auth');
    const authStore = useAuthStore();
    sseConnectionManager.initialize(router, authStore);

    // 8. 跟踪应用启动事件
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

// ===== 全局对象暴露 =====
window.app = app;
window.pinia = pinia;
window.$router = router; // 为健康检查提供router访问

// Expose errorHandler globally for application-wide access
window.errorHandler = errorHandler;

// Expose SSE connection manager for debugging
window.sseConnectionManager = sseConnectionManager;

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

  // Import connection test to verify fix
  import('./utils/connectionTest.js').then(() => {
    console.log('🔧 Connection test loaded - verifying no localhost errors');
  }).catch(err => {
    console.warn('Failed to load connection test:', err);
  });

  // Import extension conflict test to verify conflict handler
  import('./utils/extensionConflictTest.js').then(() => {
    console.log('🔧 Extension conflict test loaded - verifying conflict handling');
  }).catch(err => {
    console.warn('Failed to load extension conflict test:', err);
  });

  // Import login flow debugger for authentication issue diagnosis
  import('./utils/loginFlowDebugger.js').then(() => {
    console.log('🔍 Login flow debugger loaded - use window.debugLogin()');
  }).catch(err => {
    console.warn('Failed to load login flow debugger:', err);
  });

  // Import security test for JWT protection verification
  import('./utils/securityTest.js').then(() => {
    console.log('🛡️ Security test loaded - use window.testSecurity()');
  }).catch(err => {
    console.warn('Failed to load security test:', err);
  });

  // Import error handler test to verify showNotification fix
  import('./utils/errorHandlerTest.js').then(() => {
    console.log('🛠️ Error handler test loaded - use window.testCurrentError()');
  }).catch(err => {
    console.warn('Failed to load error handler test:', err);
  });

  // Import network conflict test for extension request conflicts
  import('./utils/networkConflictTest.js').then(() => {
    console.log('🌐 Network conflict test loaded - verifying extension conflict handling');
  }).catch(err => {
    console.warn('Failed to load network conflict test:', err);
  });

  // Import test manager for centralized test control
  import('./utils/testManager.js').then(() => {
    console.log('🧪 Test Manager loaded - use window.tests.show() for available tests');
  }).catch(err => {
    console.warn('Failed to load test manager:', err);
  });

  // Import content script error suppression test
  import('./utils/testContentScriptErrorSuppression.js').then(() => {
    console.log('🔇 Content script error suppression test loaded - use window.testContentScriptErrorSuppression()');
  }).catch(err => {
    console.warn('Failed to load content script error suppression test:', err);
  });

  // Import auth state diagnostic
  import('./utils/authStateDiagnostic.js').then(() => {
    console.log('🔍 Auth state diagnostic loaded - use window.diagnoseAuthState()');
  }).catch(err => {
    console.warn('Failed to load auth state diagnostic:', err);
  });

  // Import presence optimization config
  import('./utils/presenceOptimization.js').then(() => {
    console.log('👁️ Presence optimization loaded - use window.presenceConfig');
  }).catch(err => {
    console.warn('Failed to load presence optimization:', err);
  });

  // Import presence behavior test
  import('./utils/presenceTest.js').then(() => {
    console.log('🧪 Presence test loaded - use window.testPresenceBehavior()');
  }).catch(err => {
    console.warn('Failed to load presence test:', err);
  });

  // Import SSE robustness guide
  import('./utils/sseRobustnessGuide.js').then(() => {
    console.log('🛡️ SSE robustness guide loaded - use window.sseRobustness');
  }).catch(err => {
    console.warn('Failed to load SSE robustness guide:', err);
  });

  // Import SSE robustness test
  import('./utils/testSSERobustness.js').then(() => {
    console.log('🧪 SSE robustness test loaded - use window.testSSERobustness()');
  }).catch(err => {
    console.warn('Failed to load SSE robustness test:', err);
  });

  // Import minimal SSE test
  import('./utils/testMinimalSSE.js').then(() => {
    console.log('🧪 Minimal SSE test loaded - use window.testMinimalSSE() or window.compareSSE()');
  }).catch(err => {
    console.warn('Failed to load minimal SSE test:', err);
  });

  // Import SSE simplification guide
  import('./utils/sseSimplificationGuide.js').then(() => {
    console.log('📚 SSE simplification guide loaded - use window.sseSimplification');
  }).catch(err => {
    console.warn('Failed to load SSE simplification guide:', err);
  });

  // Import auth state sync verification test
  import('./test/verify-auth-sync.js').then(() => {
    console.log('🔐 Auth state sync test loaded - use window.verifyAuthStateSync()');
  }).catch(err => {
    console.warn('Failed to load auth state sync test:', err);
  });

  // Import group chat debug tool
  import('./utils/debugGroupChatIssue.js').then(() => {
    console.log('🐛 Group chat debug tool loaded - use window.debugGroupChat()');
  }).catch(err => {
    console.warn('Failed to load group chat debug tool:', err);
  });

  // Import message loading debug tool
  import('./utils/debugMessageLoading.js').then(() => {
    console.log('📨 Message loading debug tool loaded - use window.debugMessageLoading()');
  }).catch(err => {
    console.warn('Failed to load message loading debug tool:', err);
  });

  // Import network message diagnostic tool
  import('./utils/networkMessageDiagnostic.js').then(() => {
    console.log('🔬 Network message diagnostic loaded - use window.diagnoseMessages()');
  }).catch(err => {
    console.warn('Failed to load network message diagnostic:', err);
  });

  // Import emergency message fix
  import('./utils/emergencyMessageFix.js').then(() => {
    console.log('🚨 Emergency message fix loaded - use window.fixMessages()');
  }).catch(err => {
    console.warn('Failed to load emergency message fix:', err);
  });

  // Import message chain diagnostic tool - REMOVED: file does not exist
  // import('./utils/messageChainDiagnostic.js').then(module => {
  //   window.messageChainDiagnostic = module.default;
  //   console.log('🔍 Message Chain Diagnostic loaded. Run window.testMessageChain() to start');
  // });

  // Import request isolation test tool
  import('./utils/testRequestIsolation.js').then(() => {
    console.log('🛡️ Request isolation test loaded - use window.testRequestIsolation()');
  }).catch(err => {
    console.warn('Failed to load request isolation test:', err);
  });

  // Import message diagnostic tool
  import('./utils/messageDiagnostic.js').then(() => {
    console.log('📊 Message diagnostic loaded - use window.diagnoseMessages()');
  }).catch(err => {
    console.warn('Failed to load message diagnostic:', err);
  });

  // Import auth loop fix test
  import('./utils/testAuthLoopFix.js').then(() => {
    console.log('🔐 Auth loop fix test loaded - use window.testAuthLoopFix()');
  }).catch(err => {
    console.warn('Failed to load auth loop fix test:', err);
  });

  // Import extension conflict fix test
  import('./utils/testExtensionConflictFix.js').then(() => {
    console.log('🧩 Extension conflict fix test loaded - use window.testExtensionConflictFix()');
  }).catch(err => {
    console.warn('Failed to load extension conflict fix test:', err);
  });

  // Import extension coordination test
  import('./utils/testExtensionCoordination.js').then(() => {
    console.log('🤝 Extension coordination test loaded - use window.testExtensionCoordination()');
  }).catch(err => {
    console.warn('Failed to load extension coordination test:', err);
  });

  // Import console monitor for debugging
  import('./utils/consoleMonitor.js').then(() => {
    console.log('📊 Console monitor loaded - use window.consoleMonitor.generateReport()');
  }).catch(err => {
    console.warn('Failed to load console monitor:', err);
  });

  // Import verification script
  import('./utils/verifyAllFixes.js').then(() => {
    console.log('✅ Verification script loaded - use window.verifyAllFixes()');
  }).catch(err => {
    console.warn('Failed to load verification script:', err);
  });

  // Import message display test
  import('./utils/testMessageDisplay.js').then(() => {
    console.log('💬 Message display test loaded - use window.testMessageDisplay()');
  }).catch(err => {
    console.warn('Failed to load message display test:', err);
  });

  // Import message loading diagnostic
  import('./utils/messageLoadingDiagnostic.js').then(() => {
    console.log('🔍 Message loading diagnostic loaded - use window.diagnoseMessageLoading()');
  }).catch(err => {
    console.warn('Failed to load message loading diagnostic:', err);
  });

  // Import message user profile diagnostic
  import('./utils/messageUserProfileDiagnostic.js').then(() => {
    console.log('👤 Message user profile diagnostic loaded - use window.diagnoseMessageUserProfiles()');
  }).catch(err => {
    console.warn('Failed to load message user profile diagnostic:', err);
  });

  // Import channel switch diagnostic
  import('./utils/channelSwitchDiagnostic.js').then(() => {
    console.log('🔄 Channel switch diagnostic loaded - use window.diagnoseChannelSwitch()');
  }).catch(err => {
    console.warn('Failed to load channel switch diagnostic:', err);
  });

  // Import strict channel validation test
  import('./utils/strictChannelValidationTest.js').then(() => {
    console.log('🔍 Strict channel validation test loaded - use window.testStrictChannelValidation()');
  }).catch(err => {
    console.warn('Failed to load strict channel validation test:', err);
  });

  // Import error source preservation test
  import('./utils/testErrorSourcePreservation.js').then(() => {
    console.log('🔍 Error source preservation test loaded - use window.testErrorSourcePreservation()');
  }).catch(err => {
    console.warn('Failed to load error source preservation test:', err);
  });

  // Import transparent error handling verification
  import('./utils/verifyTransparentErrorHandling.js').then(() => {
    console.log('🔬 Transparent error handling verification loaded - use window.verifyTransparentErrorHandling()');
  }).catch(err => {
    console.warn('Failed to load transparent error handling verification:', err);
  });

  // Import pragmatic suppressor test
  import('./utils/testPragmaticSuppressor.js').then(() => {
    console.log('🧪 Pragmatic suppressor test loaded - use window.testPragmaticSuppressor()');
  }).catch(err => {
    console.warn('Failed to load pragmatic suppressor test:', err);
  });

  // Import unified error handler test
  import('./utils/testUnifiedErrorHandler.js').then(() => {
    console.log('🧪 Unified error handler test loaded - use window.testUnifiedErrorHandler()');
  }).catch(err => {
    console.warn('Failed to load unified error handler test:', err);
  });

  // Import other diagnostic tools - REMOVED: redundant/non-existent
  // import('./utils/networkMessageDiagnostic.js').then(module => {
  //   window.networkDiagnostic = module.default;
  // });
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
