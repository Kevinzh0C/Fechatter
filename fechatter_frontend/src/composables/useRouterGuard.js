/**
 * Router Navigation Guard Composable
 * Prevents rapid navigation cancellation errors through debouncing
 */
import { ref } from 'vue';
import { useRouter } from 'vue-router';

const navigationState = {
  isNavigating: ref(false),
  pendingNavigation: ref(null),
  lastNavigationTime: ref(0),
  navigationDebounceMs: 300, // Minimum 300ms between navigations
  navigationPromises: new Map() // Track ongoing navigations
};

export function useRouterGuard() {
  const router = useRouter();

  /**
   * Protected navigation with debouncing and duplicate prevention
   */
  const navigateToChat = async (chatId, options = {}) => {
    const now = Date.now();
    const timeSinceLastNav = now - navigationState.lastNavigationTime.value;

    // Convert chatId to string for consistent comparison
    const targetPath = `/chat/${chatId}`;
    const currentPath = router.currentRoute.value.path;

    // Prevent navigation to the same route
    if (currentPath === targetPath) {
      console.log(`🚫 [ROUTER_GUARD] Already on target route: ${targetPath}`);
      return Promise.resolve();
    }

    // Check if navigation is already in progress to this route
    if (navigationState.navigationPromises.has(targetPath)) {
      console.log(`⏳ [ROUTER_GUARD] Navigation to ${targetPath} already in progress`);
      return navigationState.navigationPromises.get(targetPath);
    }

    // Debounce rapid navigations
    if (timeSinceLastNav < navigationState.navigationDebounceMs) {
      const delay = navigationState.navigationDebounceMs - timeSinceLastNav;
      console.log(`🕒 [ROUTER_GUARD] Debouncing navigation for ${delay}ms`);

      // Clear any pending navigation
      if (navigationState.pendingNavigation.value) {
        clearTimeout(navigationState.pendingNavigation.value);
      }

      // Schedule delayed navigation
      return new Promise((resolve, reject) => {
        navigationState.pendingNavigation.value = setTimeout(async () => {
          try {
            const result = await performNavigation(targetPath, options);
            resolve(result);
          } catch (error) {
            reject(error);
          }
        }, delay);
      });
    }

    // Perform immediate navigation
    return performNavigation(targetPath, options);
  };

  /**
   * Internal navigation performer with error handling
   */
  const performNavigation = async (targetPath, options = {}) => {
    const { replace = false, retryOnCancel = true } = options;

    console.log(`🧭 [ROUTER_GUARD] Starting navigation to: ${targetPath}`);

    try {
      navigationState.isNavigating.value = true;
      navigationState.lastNavigationTime.value = Date.now();

      // Create navigation promise
      const navigationPromise = replace
        ? router.replace(targetPath)
        : router.push(targetPath);

      // Track the promise
      navigationState.navigationPromises.set(targetPath, navigationPromise);

      // Wait for navigation to complete
      const result = await navigationPromise;

      console.log(`✅ [ROUTER_GUARD] Navigation completed to: ${targetPath}`);
      return result;

    } catch (error) {
      // Handle specific navigation errors
      if (error.name === 'NavigationCanceled' || error.message?.includes('Navigation cancelled')) {
        console.warn(`⚠️ [ROUTER_GUARD] Navigation cancelled to ${targetPath}:`, error.message);

        // Retry once if specified
        if (retryOnCancel) {
          console.log(`🔄 [ROUTER_GUARD] Retrying cancelled navigation to: ${targetPath}`);
          await new Promise(resolve => setTimeout(resolve, 100)); // Brief delay
          return performNavigation(targetPath, { ...options, retryOnCancel: false });
        }

        // Don't throw for cancelled navigations as they're often user-initiated
        return null;
      }

      // Handle duplicate navigation (not an error)
      if (error.name === 'NavigationDuplicated' || error.message?.includes('Avoided redundant navigation')) {
        console.log(`📍 [ROUTER_GUARD] Already on target route: ${targetPath}`);
        return null;
      }

      // For other navigation errors, log and rethrow
      console.error(`❌ [ROUTER_GUARD] Navigation failed to ${targetPath}:`, error);
      throw error;

    } finally {
      navigationState.isNavigating.value = false;
      navigationState.navigationPromises.delete(targetPath);

      // Clear pending navigation if it matches current target
      if (navigationState.pendingNavigation.value) {
        clearTimeout(navigationState.pendingNavigation.value);
        navigationState.pendingNavigation.value = null;
      }
    }
  };

  /**
   * Cancel any pending navigation
   */
  const cancelPendingNavigation = () => {
    if (navigationState.pendingNavigation.value) {
      clearTimeout(navigationState.pendingNavigation.value);
      navigationState.pendingNavigation.value = null;
      console.log(`🚫 [ROUTER_GUARD] Cancelled pending navigation`);
    }
  };

  /**
   * Get current navigation state
   */
  const getNavigationState = () => ({
    isNavigating: navigationState.isNavigating.value,
    hasPendingNavigation: !!navigationState.pendingNavigation.value,
    lastNavigationTime: navigationState.lastNavigationTime.value,
    activeNavigations: Array.from(navigationState.navigationPromises.keys())
  });

  /**
   * Safe navigation wrapper for any route
   */
  const safeNavigate = async (to, options = {}) => {
    try {
      const targetPath = typeof to === 'string' ? to : router.resolve(to).path;
      return await navigateToChat(targetPath.replace('/chat/', ''), options);
    } catch (error) {
      console.error(`❌ [ROUTER_GUARD] Safe navigation failed:`, error);
      throw error;
    }
  };

  return {
    navigateToChat,
    safeNavigate,
    cancelPendingNavigation,
    getNavigationState,
    isNavigating: navigationState.isNavigating,
    hasPendingNavigation: () => !!navigationState.pendingNavigation.value
  };
}

/**
 * Global router error handler setup
 */
export function setupGlobalRouterErrorHandling(router) {
  // Enhanced afterEach hook to handle navigation failures
  router.afterEach((to, from, failure) => {
    if (failure) {
      // Log failure details for debugging
      console.group(`🚨 [ROUTER_GUARD] Navigation Failure`);
      console.log(`From: ${from.path}`);
      console.log(`To: ${to.path}`);
      console.log(`Type: ${failure.type}`);
      console.log(`Message: ${failure.message}`);
      console.groupEnd();

      // Don't show notifications for cancelled navigations
      if (failure.type !== 8) { // NavigationFailureType.cancelled = 8
        console.error('Navigation failed:', failure);
      }
    }
  });

  // Enhanced error handler
  router.onError((error) => {
    console.error('Router error:', error);

    // Prevent infinite error loops
    const currentPath = router.currentRoute.value.path;
    if (currentPath !== '/error/500' && !currentPath.startsWith('/error/')) {
      router.push('/error/500').catch(console.error);
    }
  });
} 