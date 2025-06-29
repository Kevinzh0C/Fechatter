/**
 * 🛡️ SSE Startup Guard Service
 * 防止在登录页面等非聊天页面建立SSE连接
 */

class SSEStartupGuard {
  /**
   * 检查是否应该建立SSE连接
   */
  static shouldConnectSSE(authStore = null) {
    // 检查路径
    const currentPath = window.location.pathname;
    const isOnChatPage = currentPath.startsWith('/chat/') && currentPath !== '/chat';
    const isOnAuthPage = currentPath.includes('/login') ||
      currentPath.includes('/register') ||
      currentPath === '/';

    if (import.meta.env.DEV) {
      console.log('🛡️ [SSE GUARD] Connection check:');
      console.log('  - currentPath:', currentPath);
      console.log('  - isOnChatPage:', isOnChatPage);
      console.log('  - isOnAuthPage:', isOnAuthPage);
    }

    // 路径检查
    if (!isOnChatPage || isOnAuthPage) {
      if (import.meta.env.DEV) {
        console.log('WARNING: [SSE GUARD] BLOCKING SSE - not on chat page');
      }
      return false;
    }

    // 认证检查
    if (authStore) {
      const hasValidAuth = authStore.isAuthenticated &&
        authStore.token &&
        authStore.user &&
        authStore.token.length > 50;

      if (!hasValidAuth) {
        if (import.meta.env.DEV) {
          console.log('WARNING: [SSE GUARD] BLOCKING SSE - invalid authentication');
        }
        return false;
      }
    }

    if (import.meta.env.DEV) {
      console.log('[SSE GUARD] ALLOWING SSE connection');
    }
    return true;
  }

  /**
   * 包装 SSE 连接调用，添加保护
   */
  static guardedSSEConnect(sseService, token, authStore = null) {
    if (!this.shouldConnectSSE(authStore)) {
      if (import.meta.env.DEV) {
        console.log('🛡️ [SSE GUARD] SSE connection blocked by guard');
      }
      return Promise.resolve(false);
    }

    if (import.meta.env.DEV) {
      console.log('🛡️ [SSE GUARD] Allowing SSE connection to proceed');
    }
    return sseService.connect(token);
  }
}

export default SSEStartupGuard; 