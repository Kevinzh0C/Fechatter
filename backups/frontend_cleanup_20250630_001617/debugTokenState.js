/**
 * Token State Debugging Utility
 * 
 * Diagnoses token state inconsistencies between authStore and tokenManager
 */

import { useAuthStore } from '@/stores/auth';
import tokenManager from '@/services/tokenManager';

export const debugTokenState = () => {
  console.group('🔍 [DEBUG] Token State Analysis');

  const authStore = useAuthStore();
  const tokenStatus = tokenManager.getStatus();
  const storedAuth = localStorage.getItem('auth');

  // Check localStorage
  if (import.meta.env.DEV) {
    console.log('📦 localStorage auth data:', storedAuth ? 'EXISTS' : 'MISSING');
  }

  if (storedAuth) {
    try {
      const parsedAuth = JSON.parse(storedAuth);
      if (import.meta.env.DEV) {
        console.log('📦 localStorage structure:', {
        hasUser: !!parsedAuth.user,
        hasTokens: !!parsedAuth.tokens,
        userEmail: parsedAuth.user?.email,
        accessTokenExists: !!parsedAuth.tokens?.accessToken,
        accessTokenLength: parsedAuth.tokens?.accessToken?.length || 0,
        refreshTokenExists: !!parsedAuth.tokens?.refreshToken,
        expiresAt: parsedAuth.tokens?.expiresAt ? new Date(parsedAuth.tokens.expiresAt).toISOString() : 'NULL',
        issuedAt: parsedAuth.tokens?.issuedAt ? new Date(parsedAuth.tokens.issuedAt).toISOString() : 'NULL',
      });
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('📦 localStorage parsing error:', error);
      }

  // Check authStore state
  if (import.meta.env.DEV) {
    console.log('🏪 authStore state:', {
        isAuthenticated: authStore.isAuthenticated,
    hasUser: !!authStore.user,
    userEmail: authStore.user?.email,
    isInitialized: authStore.isInitialized,
    token: authStore.token ? authStore.token.substring(0, 20) + '...' : 'NULL',
    isTokenExpired: authStore.isTokenExpired,
  });

  // Check tokenManager state
  if (import.meta.env.DEV) {
    console.log('🔐 tokenManager state:', {
        hasAccessToken: !!tokenManager.getAccessToken(),
    accessToken: tokenManager.getAccessToken() ? tokenManager.getAccessToken().substring(0, 20) + '...' : 'NULL',
    isTokenExpired: tokenManager.isTokenExpired(),
    shouldRefreshToken: tokenManager.shouldRefreshToken(),
    fullStatus: tokenStatus,
  });

  // Check tokens object directly
  const tokens = tokenManager.getTokens();
  if (import.meta.env.DEV) {
    console.log('🔑 tokenManager.tokens:', {
        accessToken: tokens.accessToken ? tokens.accessToken.substring(0, 20) + '...' : 'NULL',
    refreshToken: tokens.refreshToken ? tokens.refreshToken.substring(0, 20) + '...' : 'NULL',
    expiresAt: tokens.expiresAt ? new Date(tokens.expiresAt).toISOString() : 'NULL',
    issuedAt: tokens.issuedAt ? new Date(tokens.issuedAt).toISOString() : 'NULL',
  });

  // Inconsistency detection
  const inconsistencies = [];

  if (authStore.isAuthenticated && !tokenManager.getAccessToken()) {
    inconsistencies.push('AuthStore shows authenticated but no access token in tokenManager');
  }

  if (authStore.user && !tokenManager.getAccessToken()) {
    inconsistencies.push('User data exists but no access token');
  }

  if (!tokenManager.isTokenExpired() && !tokenManager.getAccessToken()) {
    inconsistencies.push('Token shows as not expired but no actual token exists');
  }

  if (inconsistencies.length > 0) {
    if (import.meta.env.DEV) {
      console.warn('⚠️ Detected inconsistencies:');
    inconsistencies.forEach((issue, index) => {
      if (import.meta.env.DEV) {
        console.warn(`${index + 1}. ${issue}`);
      }
    });
  } else {
    if (import.meta.env.DEV) {
      console.log('✅ No inconsistencies detected');
    }

  console.groupEnd();

  return {
    authStore: {
      isAuthenticated: authStore.isAuthenticated,
      hasUser: !!authStore.user,
      hasToken: !!authStore.token,
      isTokenExpired: authStore.isTokenExpired,
    },
    tokenManager: {
      hasAccessToken: !!tokenManager.getAccessToken(),
      isTokenExpired: tokenManager.isTokenExpired(),
      status: tokenStatus,
    },
    inconsistencies,
  };
};

export const fixTokenState = async () => {
  console.group('🔧 [FIX] Attempting to fix token state');

  const authStore = useAuthStore();

  try {
    // Check if we have stored auth data
    const storedAuth = localStorage.getItem('auth');

    if (!storedAuth) {
      if (import.meta.env.DEV) {
        console.log('📦 No stored auth data, clearing all state');
      authStore.clearAuth();
      console.groupEnd();
      return { success: true, action: 'cleared_all_state' };
    }

    const parsedAuth = JSON.parse(storedAuth);

    // Check if stored tokens exist
    if (!parsedAuth.tokens || !parsedAuth.tokens.accessToken) {
      if (import.meta.env.DEV) {
        console.log('🔑 No valid tokens in storage, clearing auth state');
      authStore.clearAuth();
      console.groupEnd();
      return { success: true, action: 'cleared_invalid_auth' };
    }

    // Check if stored access token is already expired
    const now = Date.now();
    const tokenExpiresAt = parsedAuth.tokens.expiresAt;
    const isStoredTokenExpired = !tokenExpiresAt || (now + 30000) > tokenExpiresAt; // 30s buffer

    if (isStoredTokenExpired) {
      if (import.meta.env.DEV) {
        console.log('🔑 Stored access token is expired, checking refresh token...');
      }

      // Check if refresh token is also expired
      const refreshTokenExpired = parsedAuth.tokens.absoluteExpiry && now > parsedAuth.tokens.absoluteExpiry;

      if (refreshTokenExpired || !parsedAuth.tokens.refreshToken) {
        if (import.meta.env.DEV) {
          console.log('🔑 Refresh token also expired or missing, clearing all auth state');
        authStore.clearAuth();
        console.groupEnd();
        return { success: true, action: 'cleared_expired_tokens' };
      }

      // Try to refresh the expired token
      if (import.meta.env.DEV) {
        console.log('🔄 Attempting to refresh expired token...');
      tokenManager.setTokens({
        accessToken: parsedAuth.tokens.accessToken,
        refreshToken: parsedAuth.tokens.refreshToken,
        expiresAt: parsedAuth.tokens.expiresAt,
        issuedAt: parsedAuth.tokens.issuedAt,
        absoluteExpiry: parsedAuth.tokens.absoluteExpiry,
      });

      try {
        await tokenManager.refreshToken();
        if (import.meta.env.DEV) {
          console.log('✅ Token refreshed successfully');
        console.groupEnd();
        return { success: true, action: 'token_refreshed' };
      } catch (refreshError) {
        if (import.meta.env.DEV) {
          console.error('❌ Token refresh failed:', refreshError);
        authStore.clearAuth();
        console.groupEnd();
        return { success: false, action: 'refresh_failed', error: refreshError.message };
      }
    } else {
      // Token is not expired, just reinitialize tokenManager
      if (import.meta.env.DEV) {
        console.log('🔄 Reinitializing tokenManager with valid stored tokens');
      tokenManager.setTokens({
        accessToken: parsedAuth.tokens.accessToken,
        refreshToken: parsedAuth.tokens.refreshToken,
        expiresAt: parsedAuth.tokens.expiresAt,
        issuedAt: parsedAuth.tokens.issuedAt,
        absoluteExpiry: parsedAuth.tokens.absoluteExpiry,
      });

      if (import.meta.env.DEV) {
        console.log('✅ Token state synchronized successfully');
      console.groupEnd();
      return { success: true, action: 'synchronized' };
    }

  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('❌ Failed to fix token state:', error);
    authStore.clearAuth();
    console.groupEnd();
    return { success: false, action: 'fix_failed', error: error.message };
  }
};

/**
 * Diagnose token persistence issue
 */
export const diagnoseTokenPersistence = async () => {
  console.group('🔍 [DEBUG] Token Persistence Diagnosis');

  const authStore = useAuthStore();

  // Step 1: Check current state
  if (import.meta.env.DEV) {
    console.log('📊 Step 1: Current State Analysis');
  debugTokenState();

  // Step 2: Try to manually set and retrieve token
  if (import.meta.env.DEV) {
    console.log('\n📊 Step 2: Manual Token Test');
  const testToken = 'test_token_' + Date.now();
  const testTokenData = {
    accessToken: testToken,
    refreshToken: 'test_refresh_' + Date.now(),
    expiresAt: Date.now() + (60 * 60 * 1000), // 1 hour
    issuedAt: Date.now()
  };

  if (import.meta.env.DEV) {
    console.log('Setting test token:', testTokenData.accessToken);
  tokenManager.setTokens(testTokenData);

  const retrievedToken = tokenManager.getAccessToken();
  if (import.meta.env.DEV) {
    console.log('Retrieved token:', retrievedToken);
  if (import.meta.env.DEV) {
    console.log('Token set successfully:', retrievedToken === testToken);
  }

  // Step 3: Check localStorage persistence
  if (import.meta.env.DEV) {
    console.log('\n📊 Step 3: LocalStorage Persistence Test');
  const testAuthData = {
    user: { id: 1, email: 'test@example.com' },
    tokens: tokenManager.getTokens(),
    lastLoginTime: Date.now(),
    timestamp: Date.now()
  };

  localStorage.setItem('auth_test', JSON.stringify(testAuthData));
  const storedTest = localStorage.getItem('auth_test');

  if (storedTest) {
    const parsed = JSON.parse(storedTest);
    if (import.meta.env.DEV) {
      console.log('LocalStorage write/read test:', {
        success: true,
      hasTokens: !!parsed.tokens,
      tokenMatch: parsed.tokens?.accessToken === testToken
    });
    localStorage.removeItem('auth_test');
  } else {
    if (import.meta.env.DEV) {
      console.error('LocalStorage write/read test failed');
    }

  // Step 4: Check auth store persist method
  if (import.meta.env.DEV) {
    console.log('\n📊 Step 4: Auth Store Persist Test');
  if (authStore.isAuthenticated) {
    if (import.meta.env.DEV) {
      console.log('Calling persistAuth()...');
    authStore.persistAuth();

    const storedAuth = localStorage.getItem('auth');
    if (storedAuth) {
      const parsed = JSON.parse(storedAuth);
      if (import.meta.env.DEV) {
        console.log('Persisted auth data:', {
        hasUser: !!parsed.user,
        hasTokens: !!parsed.tokens,
        hasAccessToken: !!parsed.tokens?.accessToken,
        tokenLength: parsed.tokens?.accessToken?.length || 0
      });
  } else {
    if (import.meta.env.DEV) {
      console.log('Cannot test persistAuth - user not authenticated');
    }

  // Step 5: Check for any errors or exceptions
  if (import.meta.env.DEV) {
    console.log('\n📊 Step 5: Error Detection');
  try {
    // Test token manager internal state
    const tokenStatus = tokenManager.getStatus();
    if (import.meta.env.DEV) {
      console.log('Token Manager Status:', tokenStatus);
    }

    // Test if tokens object is properly structured
    const tokens = tokenManager.getTokens();
    if (import.meta.env.DEV) {
      console.log('Token structure valid:', {
        hasAccessToken: 'accessToken' in tokens,
      hasRefreshToken: 'refreshToken' in tokens,
      hasExpiresAt: 'expiresAt' in tokens,
      hasIssuedAt: 'issuedAt' in tokens
    });
  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('Error during diagnosis:', error);
    }

  // Clean up test token
  tokenManager.clearTokens();

  console.groupEnd();

  return {
    timestamp: new Date().toISOString(),
    diagnosis: 'Check console for detailed analysis'
  };
};

// Expose for debugging
if (typeof window !== 'undefined') {
  window.debugTokenState = debugTokenState;
  window.fixTokenState = fixTokenState;
  window.diagnoseTokenPersistence = diagnoseTokenPersistence;
} 