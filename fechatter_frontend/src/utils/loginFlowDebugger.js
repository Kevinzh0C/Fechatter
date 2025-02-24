/**
 * Login Flow Debugger
 * Comprehensive debugging tool for authentication flow issues
 */

import { useAuthStore } from '@/stores/auth';
import tokenManager from '@/services/tokenManager';
import authService from '@/services/auth.service';

export class LoginFlowDebugger {
  constructor() {
    this.logs = [];
    this.startTime = Date.now();
  }

  log(message, data = null) {
    const timestamp = Date.now() - this.startTime;
    const logEntry = {
      timestamp,
      message,
      data: data ? JSON.parse(JSON.stringify(data)) : null,
      time: new Date().toISOString()
    };

    this.logs.push(logEntry);
    if (import.meta.env.DEV) {
      console.log(`🔍 [${timestamp}ms] ${message}`, data || '');
    return logEntry;
  }

  getCurrentState() {
    const authStore = useAuthStore();
    const tokenStatus = tokenManager.getStatus();
    const storedAuth = localStorage.getItem('auth');

    return {
      authStore: {
        isAuthenticated: authStore.isAuthenticated,
        hasUser: !!authStore.user,
        userEmail: authStore.user?.email,
        isInitialized: authStore.isInitialized,
        token: authStore.token ? authStore.token.substring(0, 20) + '...' : null,
        isTokenExpired: authStore.isTokenExpired,
      },
      tokenManager: {
        hasToken: tokenStatus.hasToken,
        isExpired: tokenStatus.isExpired,
        shouldRefresh: tokenStatus.shouldRefresh,
        isRefreshing: tokenStatus.isRefreshing,
        expiresIn: tokenStatus.expiresIn,
        actualToken: tokenManager.getAccessToken() ? tokenManager.getAccessToken().substring(0, 20) + '...' : null,
      },
      localStorage: {
        exists: !!storedAuth,
        structure: storedAuth ? (() => {
          try {
            const parsed = JSON.parse(storedAuth);
            return {
              hasUser: !!parsed.user,
              hasTokens: !!parsed.tokens,
              hasAccessToken: !!parsed.tokens?.accessToken,
              accessTokenLength: parsed.tokens?.accessToken?.length || 0,
            };
          } catch (e) {
            return { error: e.message };
          }
        })() : null,
      }
    };
  }

  async debugLogin(email, password) {
    this.log('🚀 Starting complete login flow debug');

    // Step 1: Initial state
    this.log('📊 Initial state', this.getCurrentState());

    // Step 2: Direct API call
    this.log('🔗 Testing direct API call');
    try {
      const apiResponse = await authService.login(email, password);
      this.log('✅ Direct API call successful', {
        hasUser: !!apiResponse.user,
        hasAccessToken: !!apiResponse.accessToken,
        hasRefreshToken: !!apiResponse.refreshToken,
        tokenLength: apiResponse.accessToken?.length || 0,
        expiresIn: apiResponse.expiresIn,
      });
    } catch (error) {
      this.log('❌ Direct API call failed', { error: error.message });
      return this.generateReport();
    }

    // Step 3: Auth store login
    this.log('🏪 Testing auth store login');
    const authStore = useAuthStore();

    try {
      // Monitor state before login
      this.log('📊 Pre-login state', this.getCurrentState());

      const loginResult = await authStore.login(email, password);

      // Monitor state immediately after login
      this.log('📊 Immediate post-login state', this.getCurrentState());

      if (loginResult) {
        this.log('✅ Auth store login successful');

        // Wait a bit for any async operations
        await new Promise(resolve => setTimeout(resolve, 100));

        // Monitor state after delay
        this.log('📊 Delayed post-login state', this.getCurrentState());

      } else {
        this.log('❌ Auth store login returned false');
      }

    } catch (error) {
      this.log('❌ Auth store login failed', { error: error.message });

    // Step 4: Test token retrieval
    this.log('🔑 Testing token retrieval methods');

    const directToken = tokenManager.getAccessToken();
    const authStoreToken = authStore.token;
    const tokensMatch = directToken === authStoreToken;

    this.log('🔑 Token comparison', {
      directTokenExists: !!directToken,
      authStoreTokenExists: !!authStoreToken,
      tokensMatch,
      directTokenLength: directToken?.length || 0,
      authStoreTokenLength: authStoreToken?.length || 0,
    });

    // Step 5: Final state
    this.log('📊 Final state', this.getCurrentState());

    return this.generateReport();
  }

  generateReport() {
    const finalState = this.getCurrentState();

    const report = {
      summary: {
        totalTime: Date.now() - this.startTime,
        logCount: this.logs.length,
        finalAuthState: finalState.authStore.isAuthenticated,
        finalTokenState: finalState.tokenManager.hasToken,
        hasInconsistency: finalState.authStore.isAuthenticated !== finalState.tokenManager.hasToken,
      },
      finalState,
      logs: this.logs,
      recommendations: this.generateRecommendations(finalState),
    };

    console.group('🔍 Login Flow Debug Report');
    if (import.meta.env.DEV) {
      console.log('📋 Summary:', report.summary);
    if (import.meta.env.DEV) {
      console.log('📊 Final State:', report.finalState);
    if (import.meta.env.DEV) {
      console.log('💡 Recommendations:', report.recommendations);
    console.groupEnd();

    return report;
  }

  generateRecommendations(state) {
    const recommendations = [];

    if (state.authStore.isAuthenticated && !state.tokenManager.hasToken) {
      recommendations.push({
        issue: 'Auth state/token mismatch',
        description: 'User appears authenticated but no token available',
        solution: 'Check tokenManager.setTokens() call in auth store login method'
      });

    if (state.authStore.hasUser && !state.authStore.isAuthenticated) {
      recommendations.push({
        issue: 'User data without authentication',
        description: 'User object exists but not marked as authenticated',
        solution: 'Verify auth state setting in login method'
      });

    if (!state.localStorage.exists && state.authStore.isAuthenticated) {
      recommendations.push({
        issue: 'No persistent storage',
        description: 'User authenticated but no localStorage data',
        solution: 'Check persistAuth() call in auth store'
      });

    if (state.tokenManager.isExpired) {
      recommendations.push({
        issue: 'Token expired immediately',
        description: 'Token is expired right after login',
        solution: 'Check server clock synchronization and token expiration time'
      });

    return recommendations;
  }

// Global debugger instance
export const loginDebugger = new LoginFlowDebugger();

// Expose to window for easy console access
if (typeof window !== 'undefined') {
  window.debugLogin = async (email = 'admin@test.com', password = 'password') => {
    const flowDebugger = new LoginFlowDebugger();
    return await flowDebugger.debugLogin(email, password);
  };
}

export default LoginFlowDebugger; 