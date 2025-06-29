<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
    <div class="max-w-md w-full space-y-8">
      <div>
        <div class="flex justify-center mb-4">
          <AppIcon :size="64" :preserve-gradient="true" start-color="#6366f1" end-color="#8b5cf6"
            title="Fechatter Logo" />
        </div>
        <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">Sign in to Fechatter</h2>
        <p class="mt-2 text-center text-sm text-gray-600">
          Welcome back! Please sign in to continue.
        </p>
      </div>

      <!-- Error Display -->
      <component :is="ErrorComponent" v-if="showError && ErrorComponent" :error="authStore.error" title="Login Error"
        suggestion="Please check your credentials and try again." @dismiss="dismissError" dismissible />

      <div v-else-if="showError" class="bg-red-50 border border-red-300 text-red-700 px-4 py-3 rounded">
        {{ authStore.error }}
        <button @click="dismissError" class="float-right font-bold text-red-500">&times;</button>
      </div>

      <form class="mt-8 space-y-6" @submit.prevent="handleSubmit" data-testid="login-form">
        <div class="rounded-md shadow-xl border-2 border-blue-200 bg-white -space-y-px">
          <div>
            <label for="email" class="sr-only">Email address</label>
            <input v-model="email" id="email" name="email" type="email" required autocomplete="email"
              :disabled="isLoading"
              class="appearance-none rounded-none relative block w-full px-3 py-2 border-0 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm disabled:opacity-50 disabled:cursor-not-allowed bg-transparent"
              placeholder="Email address" data-testid="email-input" />
          </div>
          <div class="border-t border-blue-200">
            <label for="password" class="sr-only">Password</label>
            <input v-model="password" id="password" name="password" type="password" required
              autocomplete="current-password" :disabled="isLoading"
              class="appearance-none rounded-none relative block w-full px-3 py-2 border-0 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm disabled:opacity-50 disabled:cursor-not-allowed bg-transparent"
              placeholder="Password" data-testid="password-input" />
          </div>
        </div>

        <div class="flex items-center justify-between">
          <div class="text-sm">
            <router-link to="/register" class="font-medium text-indigo-600 hover:text-indigo-500"
              data-testid="register-link">
              Don't have an account? Sign up
            </router-link>
          </div>
        </div>

        <div>
          <button type="submit" :disabled="isLoading || !email || !password"
            class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
            data-testid="login-button">
            <span v-if="isLoading && !preloadProgress.isVisible"
              class="absolute left-0 inset-y-0 flex items-center pl-3">
              <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
            </span>
            {{ getButtonText() }}
          </button>
        </div>
      </form>

      <!-- Developer Mode Toggle (Hidden by Default) -->
      <div class="mt-8 text-center">
        <button v-if="showDevHints" @click="toggleDevAccounts"
          class="group relative inline-flex items-center px-4 py-2 text-sm font-medium text-gray-600 bg-gradient-to-b from-gray-50 to-gray-100 hover:from-gray-100 hover:to-gray-150 border border-gray-300 rounded-lg transition-all duration-200 shadow-sm hover:shadow-md"
          :title="showDevAccounts ? 'Click to hide development accounts' : 'Click to show development accounts'">
          <svg class="w-4 h-4 mr-2 text-gray-500" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd"
              d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-3a1 1 0 00-.867.5 1 1 0 11-1.731-1A3 3 0 0113 8a3.001 3.001 0 01-2 2.83V11a1 1 0 11-2 0v-1a1 1 0 011-1 1 1 0 100-2zm0 8a1 1 0 100-2 1 1 0 000 2z"
              clip-rule="evenodd" />
          </svg>
          <span class="mr-2">Developer Accounts</span>
          <svg class="w-4 h-4 text-gray-400 transform transition-transform duration-200"
            :class="{ 'rotate-180': showDevAccounts }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M19 9l-7 7-7-7" />
          </svg>
        </button>
      </div>

      <!-- Developer Accounts Floating Panel -->
      <div v-if="showDevHints" class="dev-accounts-floating-container">
        <Transition enter-active-class="transition-all ease-out duration-300" enter-from-class="opacity-0 translate-y-2"
          enter-to-class="opacity-100 translate-y-0" leave-active-class="transition-all ease-in duration-200"
          leave-from-class="opacity-100 translate-y-0" leave-to-class="opacity-0 translate-y-2">
          <div v-if="showDevAccounts"
            class="dev-accounts-dropdown mt-4 bg-white border border-gray-200 rounded-lg shadow-xl">
            <!-- Header with Close Button -->
            <div class="px-3 py-2 flex items-center justify-between">
              <span class="text-xs font-medium text-gray-500 select-none">Developer Accounts</span>
              <button @click="toggleDevAccounts"
                class="close-button text-gray-400 hover:text-gray-700 hover:bg-gray-100 transition-all duration-200 p-2 rounded-lg flex items-center justify-center"
                title="Close Developer Accounts" aria-label="Close">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2.5">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <!-- Accounts Container -->
            <div class="accounts-scroll-container">
              <!-- Super User Account -->
              <div class="account-card">
                <div class="account-header">
                  <div class="account-info">
                    <div class="account-avatar super-avatar">S</div>
                    <div>
                      <h4 class="account-title">Super User</h4>
                      <p class="account-subtitle">Full system access</p>
                    </div>
                  </div>
                  <span class="account-badge super">SUPER</span>
                </div>
                <div class="account-content">
                  <div class="credential-row">
                    <span class="label">Email:</span>
                    <span class="value" @click="fillSuperCredentials">super@test.com</span>
                  </div>
                  <div class="credential-row">
                    <span class="label">Password:</span>
                    <span class="value" @click="fillSuperCredentials">password</span>
                  </div>
                </div>
                <button @click="fillSuperCredentials" class="fill-button">
                  <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                  Quick Fill
                </button>
              </div>

              <!-- Admin Account -->
              <div class="account-card">
                <div class="account-header">
                  <div class="account-info">
                    <div class="account-avatar admin-avatar">A</div>
                    <div>
                      <h4 class="account-title">Admin User</h4>
                      <p class="account-subtitle">Administrative access</p>
                    </div>
                  </div>
                  <span class="account-badge admin">ADMIN</span>
                </div>
                <div class="account-content">
                  <div class="credential-row">
                    <span class="label">Email:</span>
                    <span class="value" @click="fillAdminCredentials">admin@test.com</span>
                  </div>
                  <div class="credential-row">
                    <span class="label">Password:</span>
                    <span class="value" @click="fillAdminCredentials">password</span>
                  </div>
                </div>
                <button @click="fillAdminCredentials" class="fill-button">
                  <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                  Quick Fill
                </button>
              </div>
            </div>
          </div>
        </Transition>
      </div>

    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, shallowRef } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { optimizeLoginPerformance, analyzeLoginPerformance } from '@/utils/login-performance';
import { AppIcon } from '@/components/icons';
import { useAnalytics } from '@/composables/useAnalytics';

const router = useRouter();
const authStore = useAuthStore();
const analytics = useAnalytics();

// Performance optimized refs
const email = ref('');
const password = ref('');
const isSubmitting = ref(false);
const mounted = ref(false);
const showDevHints = ref(false);
const showDevAccounts = ref(false); // Initially hide dev accounts
const ErrorComponent = shallowRef(null);


// Preload progress state
const preloadProgress = ref({
  isVisible: false,
  message: ''
});

// Optimized computed properties
const isLoading = computed(() => authStore.loading || isSubmitting.value);
const showError = computed(() => !!authStore.error);

// Dynamic button text
const getButtonText = () => {
  if (!isLoading.value) return 'Sign in';
  if (preloadProgress.value.isVisible) {
    return 'Signing in...';
  }
  return 'Signing in...';
};

// CSRF token handling
const getCsrfToken = () => {
  const metaToken = document.querySelector('meta[name="csrf-token"]');
  return metaToken?.content || null;
};

// CSRF token handling
const dismissError = () => {
  authStore.error = null;
};

// Optimized credential filling
const fillCredentials = (emailVal, passwordVal) => {
  email.value = emailVal;
  password.value = passwordVal;

  // Optimize DOM operations with requestAnimationFrame
  requestAnimationFrame(() => {
    const loginButton = document.querySelector('[data-testid="login-button"]');
    loginButton?.focus();
  });
};

const fillAdminCredentials = () => fillCredentials('admin@test.com', 'password');
const fillSuperCredentials = () => fillCredentials('super@test.com', 'password');

// Toggle development accounts visibility
const toggleDevAccounts = () => {
  showDevAccounts.value = !showDevAccounts.value;
};

// Handle keyboard events for developer accounts  
const handleKeyDown = (event) => {
  if (event.key === 'Escape' && showDevAccounts.value) {
    showDevAccounts.value = false;
  }
};

// Simplified login handling
const handleSubmit = async () => {
  // Prevent duplicate submissions
  if (!email.value || !password.value || isLoading.value) {
    return;
  }

  try {
    isSubmitting.value = true;
    authStore.error = null;

    // Show login status
    preloadProgress.value = {
      isVisible: true,
      message: 'Signing in...'
    };

    // Track login attempt
    const loginStartTime = Date.now();

    // Use simplified login method
    const success = await authStore.login(email.value.trim(), password.value);

    if (success) {
      // Track successful login
      analytics.trackUserLogin(email.value.trim(), 'password');

      // Track login performance
      const loginDuration = Date.now() - loginStartTime;
      analytics.track('navigation', {
        from: 'login_form',
        to: 'authenticated',
        duration_ms: loginDuration
      });
      // Show success status
      preloadProgress.value = {
        isVisible: true,
        message: 'Login successful, verifying...'
      };

      // Wait for auth state to be fully ready
      await new Promise(resolve => setTimeout(resolve, 100));

      // Pre-verification wait
      // Simplified verification
      if (import.meta.env.DEV) {
        console.log('[LOGIN] Verifying auth state...');
      }

      // Brief stabilization for UI state updates
      await new Promise(resolve => {
        requestAnimationFrame(() => {
          requestAnimationFrame(() => {
            setTimeout(resolve, 150);
          });
        });
      });

      // Simplified verification to avoid blocking
      const isAuthReady = await verifyAuthStateReady();

      if (!isAuthReady) {
        // Single retry if first check fails
        if (import.meta.env.DEV) {
          console.warn('[LOGIN] Initial auth verification failed, retrying...');
        }

        await new Promise(resolve => setTimeout(resolve, 100));
        const retryResult = await verifyAuthStateReady();

        if (!retryResult) {
          // Check if we have basic auth data
          const hasBasicAuth = authStore.token && authStore.user;
          if (!hasBasicAuth) {
            throw new Error('Authentication failed - no valid auth data found');
          } else {
            if (import.meta.env.DEV) {
              console.warn('[LOGIN] Verification failed but basic auth exists - proceeding');
            }
          }
        }
      }

      // Show navigation status
      preloadProgress.value = {
        isVisible: true,
        message: 'Redirecting...'
      };

      // Brief delay to ensure UI state updates
      await new Promise(resolve => setTimeout(resolve, 150));

      // Enhanced navigation logic with duplicate check
      const currentPath = window.location.pathname;
      const redirectPath = sessionStorage.getItem('redirectPath');
      let targetPath = '/home';

      // Determine target path
      if (redirectPath && redirectPath !== '/login' && redirectPath !== '/') {
        sessionStorage.removeItem('redirectPath');
        targetPath = redirectPath;
        console.log('[LOGIN] Redirecting to stored path:', redirectPath);
      } else {
        console.log('[LOGIN] Redirecting to home');
      }

      // Simplified navigation with error handling
      if (currentPath !== targetPath) {
        try {
          console.log('[LOGIN] Navigating from', currentPath, 'to', targetPath);

          await router.push(targetPath);
          console.log('[LOGIN] Navigation successful to:', targetPath);
        } catch (error) {
          const errorName = error?.name || 'Unknown';
          const errorMessage = error?.message || '';

          if (errorName === 'NavigationDuplicated' || errorMessage.includes('Avoided redundant navigation')) {
            console.log('[LOGIN] Navigation duplicate detected - user already at target');
            return;
          } else {
            console.warn('[LOGIN] Router navigation failed:', error);
            // Fallback navigation
            window.location.assign(targetPath);
          }
        }
      } else {
        console.log('[LOGIN] Already at target path, no navigation needed');
      }
    }
  } catch (error) {
    console.error('Login failed:', error);

    // Track login error
    analytics.trackError(
      'login_failed',
      error.code || 'unknown',
      error.message || 'Login failed',
      error.stack || '',
      'login_form'
    );

    preloadProgress.value = {
      isVisible: false,
      message: ''
    };

    if (authStore.error) {
      const errorMessage = typeof authStore.error === 'string'
        ? authStore.error
        : authStore.error.message || 'Login failed';
      console.error('Authentication error:', errorMessage);
    }
  } finally {
    isSubmitting.value = false;
    // Clear progress display
    setTimeout(() => {
      preloadProgress.value = {
        isVisible: false,
        message: ''
      };
    }, 1000);
  }
};

/**
 * Practical auth state verification
 */
const verifyAuthStateReady = async () => {
  try {
    // Wait for state synchronization
    await new Promise(resolve => setTimeout(resolve, 200));

    // Essential checks
    const hasToken = !!authStore.token && authStore.token.length > 10;
    const hasUser = !!authStore.user && !!authStore.user.id;
    const isAuthReported = authStore.isAuthenticated;

    // Basic functional requirements
    const hasFunctionalAuth = hasToken && hasUser;

    // Accept if we have functional auth OR store reports auth
    const isReady = hasFunctionalAuth || isAuthReported;

    if (import.meta.env.DEV) {
      console.log('[LOGIN] Auth verification:', {
        hasToken,
        hasUser,
        isAuthReported,
        hasFunctionalAuth,
        isReady,
        tokenLength: authStore.token?.length,
        userId: authStore.user?.id
      });
    }

    return isReady;
  } catch (error) {
    console.error('[LOGIN] Auth verification error:', error);
    return false;
  }
};

onMounted(() => {
  mounted.value = true;

  // Enable performance optimizations
  optimizeLoginPerformance();

  // Clear error state
  authStore.error = null;

  // Add keyboard event listener
  document.addEventListener('keydown', handleKeyDown);

  // Optimize initialization with requestAnimationFrame
  requestAnimationFrame(() => {
    // Focus email input
    const emailInput = document.querySelector('[data-testid="email-input"]');
    if (emailInput) emailInput.focus();

    // Delayed dev hints display
    setTimeout(() => {
      // Support dev accounts in production for demo/testing
      showDevHints.value = import.meta.env.DEV ||
        import.meta.env.VITE_SHOW_DEV_ACCOUNTS === 'true' ||
        window.location.hostname.includes('vercel.app') ||
        window.location.hostname.includes('demo') ||
        window.location.hostname.includes('test');

      // Show performance analysis in dev
      if (import.meta.env.DEV) {
        setTimeout(() => {
          analyzeLoginPerformance();
        }, 1000);
      }
    }, 100);
  });

  // Preload error component during idle time
  if ('requestIdleCallback' in window) {
    requestIdleCallback(() => {
      loadErrorComponent();
    });
  } else {
    setTimeout(loadErrorComponent, 200);
  }
});

onUnmounted(() => {
  mounted.value = false;

  // Clean up keyboard event listener
  document.removeEventListener('keydown', handleKeyDown);
});

// Lazy load AuthError component
const loadErrorComponent = async () => {
  if (!ErrorComponent.value) {
    try {
      const module = await import('@/components/common/AuthError.vue');
      ErrorComponent.value = module.default;
    } catch (error) {
      console.warn('Failed to load AuthError component:', error);
    }
  }
};
</script>

<style scoped>
/* Developer Accounts floating container - absolute positioning */
.dev-accounts-floating-container {
  /* Absolute positioning */
  position: absolute;
  top: calc(100% - 5rem);
  left: 0;
  right: 0;
  z-index: 50;

  /* No layout impact */
  pointer-events: none;
}

/* Floating panel content responds to clicks */
.dev-accounts-floating-container .dev-accounts-dropdown {
  pointer-events: auto;
}

/* Close button */
.close-button {
  /* Adequate click area for mobile */
  min-width: 36px;
  min-height: 36px;

  border: 1px solid transparent;

  transform: scale(1);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.close-button:hover {
  transform: scale(1.05);
  border-color: #e5e7eb;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.close-button:active {
  transform: scale(0.95);
  background-color: #f3f4f6;
}

.close-button:focus {
  outline: 2px solid #3b82f6;
  outline-offset: 2px;
}

/* Developer accounts 下拉内容 */
.dev-accounts-dropdown {
  /* Hardware acceleration */
  transform: translateZ(0);
  will-change: transform, opacity;

  height: 320px;
  overflow: hidden;

  padding-top: 0;
  margin-top: 0;

  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);

  background-color: #ffffff;

  border: 1px solid #e5e7eb;

  border-radius: 10px;
}

/* Account scroll container */
.accounts-scroll-container {
  height: 260px;
  overflow-y: auto;
  overflow-x: hidden;

  scrollbar-width: thin;
  scrollbar-color: rgba(107, 114, 128, 0.4) rgba(107, 114, 128, 0.1);

  padding: 6px 12px;

  scroll-behavior: smooth;
  scroll-snap-type: y mandatory;
}

/* Webkit滚动条样式 */
.accounts-scroll-container::-webkit-scrollbar {
  width: 8px;
}

.accounts-scroll-container::-webkit-scrollbar-track {
  background: rgba(59, 130, 246, 0.15);
  border-radius: 4px;
}

.accounts-scroll-container::-webkit-scrollbar-thumb {
  background: rgba(59, 130, 246, 0.6);
  border-radius: 4px;
}

.accounts-scroll-container::-webkit-scrollbar-thumb:hover {
  background: rgba(59, 130, 246, 0.8);
}

/* Account cards */
.account-card {
  min-height: 200px;
  max-height: 200px;
  padding: 18px;
  background: linear-gradient(135deg, #ffffff 0%, #fafbfc 100%);
  border-radius: 10px;
  border: 1px solid #e5e7eb;
  margin-bottom: 12px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;

  scroll-snap-align: start;

  overflow: hidden;

  transition: all 200ms ease;
  box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06);
}

.account-card:hover {
  transform: translateY(-1px);
  border-color: #d1d5db;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
}

/* Account header */
.account-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
  padding-bottom: 8px;
  border-bottom: 1px solid #f3f4f6;
}

.account-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.account-avatar {
  width: 52px;
  height: 52px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 1.375rem;
  letter-spacing: 0.025em;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  transition: all 0.2s ease;
}

.account-avatar:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.super-avatar {
  background: linear-gradient(135deg, #dc2626, #991b1b);
}

.admin-avatar {
  background: linear-gradient(135deg, #2563eb, #1d4ed8);
}

.account-title {
  font-size: 1.1rem;
  font-weight: 600;
  color: #1f2937;
  margin: 0;
  letter-spacing: 0.025em;
  line-height: 1.3;
}

.account-subtitle {
  font-size: 0.875rem;
  color: #6b7280;
  margin: 0;
  font-weight: 400;
  line-height: 1.4;
}

/* Account badges */
.account-badge {
  font-size: 0.5rem;
  font-weight: 500;
  padding: 2px 6px;
  border-radius: 3px;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  opacity: 0.8;
}

.account-badge.super {
  background: #fef2f2;
  color: #b91c1c;
  border: 1px solid #fecaca;
}

.account-badge.admin {
  background: #eff6ff;
  color: #2563eb;
  border: 1px solid #dbeafe;
}

/* Account content */
.account-content {
  flex: 1;
  font-size: 0.875rem;
  margin: 8px 0;
}

.credential-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  padding: 3px 0;
}

.label {
  color: #9ca3af;
  font-weight: 500;
  font-size: 0.8rem;
  line-height: 1.3;
}

.value {
  color: #1f2937;
  font-weight: 600;
  cursor: pointer;
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  font-size: 0.9rem;
  line-height: 1.4;
  padding: 8px 12px;
  border-radius: 6px;
  background: #f9fafb;
  border: 1px solid #e5e7eb;
  transition: all 150ms ease;
}

.value:hover {
  background-color: #f3f4f6;
  border-color: #d1d5db;
  color: #111827;
}

/* Fill button */
.fill-button {
  width: 100%;
  padding: 14px 20px;
  background: linear-gradient(135deg, #4f46e5, #6366f1);
  border: none;
  border-radius: 10px;
  color: white;
  font-size: 0.95rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 44px;
  box-shadow: 0 2px 4px rgba(79, 70, 229, 0.2);
  -webkit-font-smoothing: antialiased;
  text-rendering: optimizeLegibility;
}

.fill-button:hover {
  background: linear-gradient(135deg, #4338ca, #5b21b6);
  transform: translateY(-2px);
  box-shadow: 0 6px 12px rgba(79, 70, 229, 0.3);
}

.fill-button:active {
  transform: translateY(-1px);
  background: linear-gradient(135deg, #3730a3, #4c1d95);
  box-shadow: 0 3px 8px rgba(79, 70, 229, 0.25);
}

.fill-button:focus {
  outline: 2px solid #a5b4fc;
  outline-offset: 2px;
}

/* Last account card */
.account-card:last-child {
  margin-bottom: 0;
}

/* Page layout stability */
.min-h-screen {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem 1rem;

  contain: layout;
}

.max-w-md {
  contain: layout style;

  position: relative;
  width: 100%;
}

/* Responsive optimization */
@media (max-width: 640px) {
  .dev-accounts-dropdown {
    height: 392px;
  }

  .accounts-scroll-container {
    height: 200px;
    padding: 8px 12px;
  }

  .account-card {
    min-height: 160px;
    max-height: 160px;
    padding: 14px;
  }

  .account-avatar {
    width: 40px;
    height: 40px;
    font-size: 1.1rem;
  }

  .min-h-screen {
    padding: 1rem;
  }

  /* Height-constrained optimization */
  @media (max-height: 700px) {
    .dev-accounts-dropdown {
      height: 342px;
    }

    .min-h-screen {
      padding: 1rem;
    }

    /* Very small screen support */
    @media (max-height: 600px) {
      .dev-accounts-dropdown {
        height: 292px;
      }

      .min-h-screen {
        padding: 0.5rem;
      }

      /* High contrast support */
      @media (prefers-contrast: high) {
        .dev-accounts-dropdown {
          border: 2px solid #1e40af;
          box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.3);
        }

        /* Reduced motion */
        @media (prefers-reduced-motion: reduce) {
          .dev-accounts-dropdown {
            transition: none !important;
          }

          .dev-accounts-dropdown-container {
            transition: none !important;
          }

          /* Scrollbar optimization */
          .dev-accounts-dropdown::-webkit-scrollbar {
            width: 4px;
          }

          .dev-accounts-dropdown::-webkit-scrollbar-track {
            background: rgba(59, 130, 246, 0.1);
            border-radius: 2px;
          }

          .dev-accounts-dropdown::-webkit-scrollbar-thumb {
            background: rgba(59, 130, 246, 0.3);
            border-radius: 2px;
          }

          .dev-accounts-dropdown::-webkit-scrollbar-thumb:hover {
            background: rgba(59, 130, 246, 0.5);
          }
        }
      }
    }
  }
}
</style>