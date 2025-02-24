/**
 * SSE Connection Manager
 * Manages SSE connection lifecycle based on authentication and routing state
 * Production-grade implementation following Occam's Razor principle
 */

import { watch } from 'vue';

class SSEConnectionManager {
  constructor() {
    this.isInitialized = false;
    this.authRoutes = ['/login', '/register', '/forgot-password', '/reset-password'];
  }

  /**
   * Initialize the manager with router and auth store
   */
  initialize(router, authStore) {
    if (this.isInitialized) return;

    this.router = router;
    this.authStore = authStore;
    this.isInitialized = true;

    // Watch route changes
    this.setupRouteWatcher();

    // Watch auth state changes
    this.setupAuthWatcher();

    // console.log('✅ SSE Connection Manager initialized');
  }

  /**
   * Setup route watcher
   */
  setupRouteWatcher() {
    this.router.beforeEach((to, from, next) => {
      const isAuthRoute = this.authRoutes.includes(to.path);
      const wasAuthRoute = this.authRoutes.includes(from.path);

      // Navigating to auth page - disconnect SSE
      if (isAuthRoute && !wasAuthRoute) {
        // console.log('🔌 [SSE Manager] Navigating to auth page, disconnecting SSE');
        this.disconnectSSE();
      }
      // Navigating from auth page to app - connect SSE if authenticated
      else if (!isAuthRoute && wasAuthRoute && this.authStore.isAuthenticated) {
        // console.log('🔌 [SSE Manager] Leaving auth page, connecting SSE');
        this.connectSSE();
      }

      next();
    });
  }

  /**
   * Setup auth state watcher
   */
  setupAuthWatcher() {
    // Watch for auth state changes
    watch(
      () => this.authStore.isAuthenticated,
      (isAuthenticated, wasAuthenticated) => {
        const currentPath = this.router.currentRoute.value.path;
        const isAuthRoute = this.authRoutes.includes(currentPath);

        if (isAuthenticated && !wasAuthenticated && !isAuthRoute) {
          // User logged in and not on auth page
          // console.log('🔌 [SSE Manager] User authenticated, connecting SSE');
          this.connectSSE().catch(console.error);
        } else if (!isAuthenticated && wasAuthenticated) {
          // User logged out
          // console.log('🔌 [SSE Manager] User logged out, disconnecting SSE');
          this.disconnectSSE();
        }
      }
    );
  }

  /**
   * Connect SSE service
   */
  async connectSSE() {
    try {
      if (window.realtimeCommunicationService) {
        // CRITICAL: Ensure we have a valid token before connecting
        await this.authStore.ensureAuthStateConsistency();

        // Get token from multiple sources
        const token = this.authStore.token ||
          window.tokenManager?.getAccessToken() ||
          this.authStore.getAccessToken?.();

        if (token) {
          // console.log('🔌 [SSE Manager] Connecting with token:', token.substring(0, 20) + '...');
          window.realtimeCommunicationService.connect(token);
        } else {
          if (import.meta.env.DEV) {
            console.warn('🔌 [SSE Manager] No token available for SSE connection');
          }
        }
      }
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to connect SSE:', error);
      }
    }
  }

  /**
   * Disconnect SSE service
   */
  disconnectSSE() {
    try {
      if (window.realtimeCommunicationService) {
        window.realtimeCommunicationService.disconnect();
      }
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to disconnect SSE:', error);
      }
    }
  }

  /**
   * Check if current route is auth route
   */
  isOnAuthRoute() {
    try {
      if (!this.router || !this.router.currentRoute) {
        // console.warn('[SSE Manager] Router not initialized, defaulting to false');
        return false;
      }

      const currentPath = this.router.currentRoute.value.path;
      return this.authRoutes.includes(currentPath);
    } catch (error) {
      // console.warn('[SSE Manager] Error checking auth route:', error.message);
      return false;
    }
  }
}

// Export singleton instance
const sseConnectionManager = new SSEConnectionManager();

export default sseConnectionManager; 