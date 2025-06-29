/**
 * Router Guard Composable
 * Provides enhanced router error handling and navigation guards
 */

/**
 * Setup global router error handling
 * @param {Router} router - Vue Router instance
 */
export function setupGlobalRouterErrorHandling(router) {
  // Enhanced error handling for navigation failures
  router.onError((error) => {
    console.error('[Router Error]:', error);

    // CRITICAL FIX: 忽略更多常见的无害错误
    if (error.name === 'NavigationDuplicated' ||
      error.message?.includes('redundant navigation') ||
      error.message?.includes('Avoided redundant')) {
      console.log('[Router] Ignoring redundant navigation error');
      return;
    }

    if (error.name === 'NavigationAborted') {
      console.warn('[Router] Navigation was aborted:', error.message);
      return;
    }

    if (error.name === 'NavigationCancelled') {
      console.warn('[Router] Navigation was cancelled:', error.message);
      return;
    }

    // CRITICAL FIX: 只在真正严重的错误时重定向
    if (error.name === 'ChunkLoadError' ||
      error.message?.includes('Loading chunk') ||
      error.message?.includes('Loading CSS chunk')) {
      console.error('[Router] Chunk load error, redirecting to error page');
      if (!router.currentRoute.value.path.startsWith('/error')) {
        router.push('/error/500').catch(() => {
          console.error('[Router] Failed to navigate to error page');
        });
      }
      return;
    }

    // 对于其他错误，仅记录，不强制重定向
    console.warn('[Router] Non-critical error, continuing:', error.message);
  });

  // Setup navigation timeout handling
  let navigationTimeout;

  router.beforeEach((to, from, next) => {
    // Clear any existing timeout
    if (navigationTimeout) {
      clearTimeout(navigationTimeout);
    }

    // Set navigation timeout (30 seconds)
    navigationTimeout = setTimeout(() => {
      console.error('[Router] Navigation timeout');
      if (!router.currentRoute.value.path.startsWith('/error')) {
        router.push('/error/500').catch(() => {
          console.error('[Router] Failed to navigate to error page after timeout');
        });
      }
    }, 30000);

    next();
  });

  router.afterEach(() => {
    // Clear navigation timeout on successful navigation
    if (navigationTimeout) {
      clearTimeout(navigationTimeout);
      navigationTimeout = null;
    }
  });

  console.log('[Router] Global error handling setup complete');
}

/**
 * Additional router guard utilities
 */
export function createAuthGuard(authStore) {
  return async (to, from, next) => {
    if (to.meta.requiresAuth && !authStore.isAuthenticated) {
      next('/login');
    } else {
      next();
    }
  };
}

export function createAdminGuard(authStore) {
  return async (to, from, next) => {
    if (to.meta.requiresAdmin && (!authStore.isAuthenticated || !authStore.user?.isAdmin)) {
      next('/error/403');
    } else {
      next();
    }
  };
}

export default {
  setupGlobalRouterErrorHandling,
  createAuthGuard,
  createAdminGuard
}; 