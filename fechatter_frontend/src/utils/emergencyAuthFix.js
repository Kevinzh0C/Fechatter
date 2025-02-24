/**
 * Emergency Auth Fix
 * Quick fix for authentication issues and 401 errors
 */

async function emergencyAuthFix() {
  console.group('🚨 Emergency Auth Fix');

  try {
    if (import.meta.env.DEV) {
      console.log('1️⃣ Clearing all authentication state...');
    }

    // Clear localStorage
    const authKeys = ['auth', 'auth_token', 'token', 'refreshToken', 'user', 'fechatter_access_token', 'fechatter_token_expiry'];
    authKeys.forEach(key => {
      localStorage.removeItem(key);
      sessionStorage.removeItem(key);
    });

    // Clear all localStorage with auth-related keys
    for (let i = localStorage.length - 1; i >= 0; i--) {
      const key = localStorage.key(i);
      if (key && (key.includes('auth') || key.includes('token') || key.includes('user'))) {
        localStorage.removeItem(key);
      }

    if (import.meta.env.DEV) {
      console.log('✅ Storage cleared');
    }

    if (import.meta.env.DEV) {
      console.log('2️⃣ Clearing token manager...');
    }

    // Clear token manager if available
    try {
      const { default: tokenManager } = await import('@/services/tokenManager');
      await tokenManager.clearTokens();
      if (import.meta.env.DEV) {
        console.log('✅ Token manager cleared');
      }
    } catch (error) {
      if (import.meta.env.DEV) {
        console.warn('⚠️ Token manager not available or failed to clear:', error.message);
      }

    if (import.meta.env.DEV) {
      console.log('3️⃣ Clearing auth store...');
    }

    // Clear auth store if available
    try {
      const { useAuthStore } = await import('@/stores/auth');
      const authStore = useAuthStore();
      authStore.clearAuth();
      if (import.meta.env.DEV) {
        console.log('✅ Auth store cleared');
      }
    } catch (error) {
      if (import.meta.env.DEV) {
        console.warn('⚠️ Auth store not available or failed to clear:', error.message);
      }

    if (import.meta.env.DEV) {
      console.log('4️⃣ Disconnecting SSE...');
    }

    // Disconnect SSE if available
    try {
      if (window.sseConnectionManager) {
        window.sseConnectionManager.disconnect();
        if (import.meta.env.DEV) {
          console.log('✅ SSE disconnected');
        }
      } else {
        if (import.meta.env.DEV) {
          console.log('✅ No SSE connection to disconnect');
        }
    } catch (error) {
      if (import.meta.env.DEV) {
        console.warn('⚠️ SSE disconnect failed:', error.message);
      }

    if (import.meta.env.DEV) {
      console.log('5️⃣ Redirecting to login...');
    }

    // Force redirect to login
    const currentPath = window.location.pathname;
    if (currentPath !== '/login' && currentPath !== '/register') {
      if (import.meta.env.DEV) {
        console.log('🔄 Redirecting to login page...');
      window.location.href = '/login';
    } else {
      if (import.meta.env.DEV) {
        console.log('✅ Already on login page');
      }

    if (import.meta.env.DEV) {
      console.log('✅ Emergency auth fix completed');
    }

  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('❌ Emergency auth fix failed:', error);
    }

    // Ultimate fallback: hard reload to login
    if (import.meta.env.DEV) {
      console.log('🔄 Performing hard redirect as fallback...');
    window.location.href = '/login';
  }

  console.groupEnd();
}

/**
 * Quick 401 error suppression
 */
function quick401Suppression() {
  if (import.meta.env.DEV) {
    console.log('🔇 Applying quick 401 error suppression...');
  }

  // Add patterns to log suppressor if available
  if (window.logSuppressor) {
    const patterns401 = [
      /401 \(Unauthorized\)/,
      /GET.*401/,
      /POST.*401/,
      /🚨 API Error.*401/,
      /Authentication failure/,
      /session has expired/i,
      /unauthorized/i
    ];

    patterns401.forEach(pattern => {
      window.logSuppressor.addPattern(pattern);
    });

    if (import.meta.env.DEV) {
      console.log('✅ 401 error suppression patterns added');
    }
  } else {
    if (import.meta.env.DEV) {
      console.warn('⚠️ Log suppressor not available');
    }

/**
 * Check if we're in an auth error state
 */
function isAuthErrorState() {
  // Check for common indicators of auth errors
  const hasLocalStorageAuth = !!localStorage.getItem('auth');
  const hasSessionStorageAuth = !!sessionStorage.getItem('auth_token');
  const currentPath = window.location.pathname;
  const isOnAuthPage = currentPath === '/login' || currentPath === '/register';

  // If we have auth data but not on auth page, check if it's valid
  if ((hasLocalStorageAuth || hasSessionStorageAuth) && !isOnAuthPage) {
    try {
      const authData = JSON.parse(localStorage.getItem('auth') || '{}');
      const tokenExpiry = authData.tokens?.expiresAt;

      if (tokenExpiry && Date.now() > tokenExpiry) {
        if (import.meta.env.DEV) {
          console.log('🔍 Detected expired token');
        return true;
      }
    } catch (error) {
      if (import.meta.env.DEV) {
        console.log('🔍 Detected corrupted auth data');
      return true;
    }

  return false;
}

// Make available globally
if (typeof window !== 'undefined') {
  window.emergencyAuthFix = emergencyAuthFix;
  window.quick401Suppression = quick401Suppression;
  window.isAuthErrorState = isAuthErrorState;

  // Auto-apply 401 suppression
  quick401Suppression();

  // Auto-run if in auth error state
  if (isAuthErrorState()) {
    if (import.meta.env.DEV) {
      console.log('🔍 Auth error state detected, auto-running emergency fix in 2 seconds...');
    setTimeout(() => {
      emergencyAuthFix();
    }, 2000);
  }

  if (import.meta.env.DEV) {
    console.log('🚨 Emergency Auth Fix loaded');
  if (import.meta.env.DEV) {
    console.log('💡 Use window.emergencyAuthFix() for immediate auth cleanup');
  if (import.meta.env.DEV) {
    console.log('💡 Use window.quick401Suppression() to suppress 401 noise');
  }

export default emergencyAuthFix; 