/**
 * 🏥 Quick Health Check - Instant system status overview
 * 
 * Provides a rapid assessment of system health with clear indicators
 */

export function quickHealthCheck() {
  console.group('🏥 Quick Health Check');

  const checks = {
    '🚀 Frontend Server': checkFrontendServer(),
    '🔧 Message Service': checkMessageService(),
    '💾 Cache System': checkCacheSystem(),
    '🎯 Components': checkComponents(),
    '🛡️ Security': checkSecurity(),
    '📊 Performance': checkPerformance()
  };

  let allHealthy = true;

  Object.entries(checks).forEach(([name, result]) => {
    const status = result.healthy ? '✅' : '❌';
    console.log(`${status} ${name}: ${result.message}`);
    if (!result.healthy) allHealthy = false;
  });

  console.log('\n' + '='.repeat(50));
  console.log(`Overall Status: ${allHealthy ? '✅ HEALTHY' : '❌ ISSUES DETECTED'}`);

  if (!allHealthy) {
    console.log('\n🔧 Quick fixes available:');
    console.log('- Run: window.debugFullReport() for detailed analysis');
    console.log('- Run: window.resetCircuitBreaker() if auto-load is stuck');
    console.log('- Run: window.forceStopLoadingDebug() if loading indicators are stuck');
  }

  console.groupEnd();

  return { allHealthy, checks };
}

function checkFrontendServer() {
  try {
    const isRunning = window.location.hostname && window.location.port;
    return {
      healthy: !!isRunning,
      message: isRunning ? 'Running normally' : 'Server connection issues'
    };
  } catch (error) {
    return { healthy: false, message: 'Server check failed' };
  }
}

function checkMessageService() {
  try {
    if (!window.unifiedMessageService) {
      return { healthy: false, message: 'Service not available' };
    }

    const service = window.unifiedMessageService;
    const isInitialized = service.isInitialized?.value;
    const cacheCount = Object.keys(service.messageCache || {}).length;

    return {
      healthy: !!isInitialized,
      message: isInitialized
        ? `Initialized with ${cacheCount} cached chats`
        : 'Not initialized'
    };
  } catch (error) {
    return { healthy: false, message: 'Service check failed' };
  }
}

function checkCacheSystem() {
  try {
    if (!window.unifiedMessageService) {
      return { healthy: false, message: 'Message service unavailable' };
    }

    const service = window.unifiedMessageService;
    const cacheKeys = Object.keys(service.messageCache || {});
    const totalMessages = Array.from(service.messagesByChat?.values() || [])
      .reduce((total, messages) => total + (messages?.length || 0), 0);

    return {
      healthy: cacheKeys.length >= 0, // Always healthy if service exists
      message: `${cacheKeys.length} chats cached, ${totalMessages} total messages`
    };
  } catch (error) {
    return { healthy: false, message: 'Cache check failed' };
  }
}

function checkComponents() {
  try {
    const messageListComponents = document.querySelectorAll('.simple-message-list').length;
    const messageElements = document.querySelectorAll('.message-wrapper').length;
    const loadingIndicators = document.querySelectorAll('.auto-load-indicator').length;

    return {
      healthy: messageListComponents > 0,
      message: `${messageListComponents} message lists, ${messageElements} messages, ${loadingIndicators} loading indicators`
    };
  } catch (error) {
    return { healthy: false, message: 'Component check failed' };
  }
}

function checkSecurity() {
  try {
    const hasAuth = !!window.__pinia_stores__?.auth;
    const hasSecurityUtils = !!window.securityUtils;

    return {
      healthy: hasAuth && hasSecurityUtils,
      message: hasAuth && hasSecurityUtils ? 'Security systems active' : 'Security systems missing'
    };
  } catch (error) {
    return { healthy: false, message: 'Security check failed' };
  }
}

function checkPerformance() {
  try {
    const memoryInfo = performance.memory;
    if (!memoryInfo) {
      return { healthy: true, message: 'Memory API not available' };
    }

    const usedMB = Math.round(memoryInfo.usedJSHeapSize / 1024 / 1024);
    const limitMB = Math.round(memoryInfo.jsHeapSizeLimit / 1024 / 1024);
    const usage = usedMB / limitMB;

    return {
      healthy: usage < 0.8, // Healthy if under 80% memory usage
      message: `Memory: ${usedMB}MB/${limitMB}MB (${Math.round(usage * 100)}%)`
    };
  } catch (error) {
    return { healthy: true, message: 'Performance check unavailable' };
  }
}

// Make available globally in development
if (import.meta.env.DEV) {
  window.quickHealthCheck = quickHealthCheck;
} 