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

    // Handle specific error types
    if (error.name === 'NavigationDuplicated') {
      // Ignore duplicate navigation errors
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

    // For other errors, redirect to error page if not already there
    if (!router.currentRoute.value.path.startsWith('/error')) {
      router.push('/error/500').catch(() => {
        // Fallback if error page navigation also fails
        console.error('[Router] Failed to navigate to error page');
      });
    }
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