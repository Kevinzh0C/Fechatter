/**
 * Authentication Diagnostics & Auto-Fix System
 * Production-grade tool to diagnose and resolve authentication issues
 */

class AuthDiagnostics {
  constructor() {
    this.lastDiagnosticsRun = null;
    this.diagnosticsHistory = [];
  }

  /**
   * Run comprehensive authentication diagnostics
   */
  async runDiagnostics() {
    const startTime = Date.now();
    const result = {
      timestamp: startTime,
      checks: {},
      summary: {},
      recommendations: [],
      autoFixable: []
    };

    console.log('🔬 [Auth Diagnostics] Starting comprehensive authentication analysis...');

    try {
      // Check 1: Token Status
      result.checks.tokens = this.checkTokenStatus();

      // Check 2: API Configuration
      result.checks.apiConfig = this.checkAPIConfiguration();

      // Check 3: Auth Store State
      result.checks.authStore = await this.checkAuthStoreState();

      // Check 4: Network Connectivity
      result.checks.network = await this.checkNetworkConnectivity();

      // Check 5: Backend Health
      result.checks.backend = await this.checkBackendHealth();

      // Generate analysis
      result.summary = this.generateAnalysis(result.checks);
      result.recommendations = this.generateRecommendations(result.checks);
      result.autoFixable = this.identifyAutoFixableIssues(result.checks);

      const duration = Date.now() - startTime;
      result.duration = duration;

      console.log(`[Auth Diagnostics] Analysis completed in ${duration}ms`);
      this.displayResults(result);

      this.lastDiagnosticsRun = result;
      this.diagnosticsHistory.push(result);

      return result;

    } catch (error) {
      console.error('ERROR: [Auth Diagnostics] Error during analysis:', error);
      result.error = error.message;
      return result;
    }
  }

  /**
   * Check token status and validity
   */
  checkTokenStatus() {
    const accessToken = localStorage.getItem('access_token');
    const refreshToken = localStorage.getItem('refresh_token');

    const result = {
      hasAccessToken: !!accessToken,
      hasRefreshToken: !!refreshToken,
      accessTokenValid: false,
      refreshTokenValid: false,
      tokenDetails: {}
    };

    if (accessToken) {
      try {
        const payload = JSON.parse(atob(accessToken.split('.')[1]));
        const now = Date.now() / 1000;

        result.accessTokenValid = payload.exp > now;
        result.tokenDetails.access = {
          exp: payload.exp,
          iat: payload.iat,
          expiresIn: Math.max(0, payload.exp - now),
          isExpired: payload.exp <= now,
          user: {
            id: payload.user?.id,
            email: payload.user?.email,
            workspace_id: payload.user?.workspace_id
          }
        };
      } catch (error) {
        result.tokenDetails.access = { error: 'Invalid token format' };
      }
    }

    if (refreshToken) {
      try {
        const payload = JSON.parse(atob(refreshToken.split('.')[1]));
        const now = Date.now() / 1000;

        result.refreshTokenValid = payload.exp > now;
        result.tokenDetails.refresh = {
          exp: payload.exp,
          iat: payload.iat,
          expiresIn: Math.max(0, payload.exp - now),
          isExpired: payload.exp <= now
        };
      } catch (error) {
        result.tokenDetails.refresh = { error: 'Invalid token format' };
      }
    }

    return result;
  }

  /**
   * Check API configuration
   */
  checkAPIConfiguration() {
    const result = {
      baseURL: window.location.origin,
      hasAxiosConfig: typeof window.axios !== 'undefined',
      authHeaderPresent: false
    };

    // Check if authorization header is set
    if (window.axios?.defaults?.headers?.common?.Authorization) {
      result.authHeaderPresent = true;
      result.authHeader = window.axios.defaults.headers.common.Authorization;
    }

    return result;
  }

  /**
   * Check auth store state
   */
  async checkAuthStoreState() {
    const result = {
      storeAvailable: false,
      isAuthenticated: false,
      hasUser: false,
      user: null
    };

    try {
      if (typeof window !== 'undefined' && window.__pinia_stores__) {
        const authStore = window.__pinia_stores__.auth();
        if (authStore) {
          result.storeAvailable = true;
          result.isAuthenticated = authStore.isAuthenticated;
          result.hasUser = !!authStore.user;
          result.user = authStore.user ? {
            id: authStore.user.id,
            email: authStore.user.email,
            workspace_id: authStore.user.workspace_id
          } : null;
        }
      }
    } catch (error) {
      result.error = error.message;
    }

    return result;
  }

  /**
   * Check network connectivity
   */
  async checkNetworkConnectivity() {
    const result = {
      online: navigator.onLine,
      canReachAPI: false,
      latency: null
    };

    try {
      const startTime = Date.now();
      const response = await fetch('/api/health', {
        method: 'GET',
        headers: { 'Cache-Control': 'no-cache' }
      });
      result.latency = Date.now() - startTime;
      result.canReachAPI = response.ok;
      result.statusCode = response.status;
    } catch (error) {
      result.networkError = error.message;
    }

    return result;
  }

  /**
   * Check backend health and auth endpoints
   */
  async checkBackendHealth() {
    const result = {
      healthCheck: false,
      authEndpointsAvailable: {},
      userEndpointsAvailable: {}
    };

    const endpointsToTest = [
      { name: 'health', url: '/api/health', requiresAuth: false },
      { name: 'refresh', url: '/api/refresh', requiresAuth: false },
      { name: 'users', url: '/api/users', requiresAuth: true },
      { name: 'userProfile', url: '/api/users/profile', requiresAuth: true }
    ];

    for (const endpoint of endpointsToTest) {
      try {
        const headers = endpoint.requiresAuth ? {
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`
        } : {};

        const response = await fetch(endpoint.url, { headers });
        const testResult = {
          available: true,
          status: response.status,
          ok: response.ok
        };

        if (endpoint.name === 'health') {
          result.healthCheck = response.ok;
        } else if (endpoint.requiresAuth) {
          result.userEndpointsAvailable[endpoint.name] = testResult;
        } else {
          result.authEndpointsAvailable[endpoint.name] = testResult;
        }
      } catch (error) {
        const testResult = {
          available: false,
          error: error.message
        };

        if (endpoint.requiresAuth) {
          result.userEndpointsAvailable[endpoint.name] = testResult;
        } else {
          result.authEndpointsAvailable[endpoint.name] = testResult;
        }
      }
    }

    return result;
  }

  /**
   * Generate analysis summary
   */
  generateAnalysis(checks) {
    const analysis = {
      authTokenStatus: 'unknown',
      apiConnectivity: 'unknown',
      storeIntegrity: 'unknown',
      overallHealth: 'unknown'
    };

    // Analyze token status
    if (checks.tokens.hasAccessToken && checks.tokens.accessTokenValid) {
      analysis.authTokenStatus = 'healthy';
    } else if (checks.tokens.hasRefreshToken && checks.tokens.refreshTokenValid) {
      analysis.authTokenStatus = 'needs-refresh';
    } else {
      analysis.authTokenStatus = 'critical';
    }

    // Analyze API connectivity
    if (checks.network.canReachAPI && checks.backend.healthCheck) {
      analysis.apiConnectivity = 'healthy';
    } else {
      analysis.apiConnectivity = 'critical';
    }

    // Analyze store integrity
    if (checks.authStore.storeAvailable && checks.authStore.isAuthenticated) {
      analysis.storeIntegrity = 'healthy';
    } else {
      analysis.storeIntegrity = 'degraded';
    }

    // Overall health
    const healthyCount = Object.values(analysis).filter(status => status === 'healthy').length;
    if (healthyCount >= 3) {
      analysis.overallHealth = 'healthy';
    } else if (healthyCount >= 1) {
      analysis.overallHealth = 'degraded';
    } else {
      analysis.overallHealth = 'critical';
    }

    return analysis;
  }

  /**
   * Generate specific recommendations
   */
  generateRecommendations(checks) {
    const recommendations = [];

    if (!checks.tokens.hasAccessToken) {
      recommendations.push({
        priority: 'critical',
        issue: 'No access token found',
        solution: 'User needs to log in again',
        autoFixable: false
      });
    } else if (!checks.tokens.accessTokenValid) {
      if (checks.tokens.hasRefreshToken && checks.tokens.refreshTokenValid) {
        recommendations.push({
          priority: 'high',
          issue: 'Access token expired but refresh token valid',
          solution: 'Refresh the access token',
          autoFixable: true
        });
      } else {
        recommendations.push({
          priority: 'critical',
          issue: 'Both access and refresh tokens expired',
          solution: 'User needs to log in again',
          autoFixable: false
        });
      }
    }

    if (!checks.network.canReachAPI) {
      recommendations.push({
        priority: 'critical',
        issue: 'Cannot reach API endpoints',
        solution: 'Check network connection and backend server status',
        autoFixable: false
      });
    }

    if (!checks.authStore.storeAvailable) {
      recommendations.push({
        priority: 'high',
        issue: 'Auth store not properly initialized',
        solution: 'Reinitialize the auth store',
        autoFixable: true
      });
    }

    return recommendations;
  }

  /**
   * Identify auto-fixable issues
   */
  identifyAutoFixableIssues(checks) {
    const autoFixable = [];

    if (checks.tokens.hasRefreshToken && checks.tokens.refreshTokenValid && !checks.tokens.accessTokenValid) {
      autoFixable.push('refresh-access-token');
    }

    if (!checks.authStore.storeAvailable) {
      autoFixable.push('reinitialize-auth-store');
    }

    return autoFixable;
  }

  /**
   * Auto-fix authentication issues
   */
  async autoFix() {
    console.log('[Auth Diagnostics] Starting auto-fix process...');

    const diagnostics = this.lastDiagnosticsRun || await this.runDiagnostics();
    const fixes = [];

    for (const fixType of diagnostics.autoFixable) {
      try {
        switch (fixType) {
          case 'refresh-access-token':
            await this.fixRefreshToken();
            fixes.push({ type: fixType, status: 'success' });
            break;
          case 'reinitialize-auth-store':
            await this.fixAuthStore();
            fixes.push({ type: fixType, status: 'success' });
            break;
        }
      } catch (error) {
        fixes.push({ type: fixType, status: 'failed', error: error.message });
      }
    }

    console.log('[Auth Diagnostics] Auto-fix completed:', fixes);
    return fixes;
  }

  /**
   * Fix refresh token issue
   */
  async fixRefreshToken() {
    const refreshToken = localStorage.getItem('refresh_token');
    if (!refreshToken) throw new Error('No refresh token available');

    const response = await fetch('/api/refresh', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refresh_token: refreshToken })
    });

    if (!response.ok) throw new Error(`Refresh failed: ${response.status}`);

    const data = await response.json();
    localStorage.setItem('access_token', data.access_token);
    if (data.refresh_token) {
      localStorage.setItem('refresh_token', data.refresh_token);
    }

    console.log('[Auth Diagnostics] Access token refreshed successfully');
  }

  /**
   * Fix auth store issues
   */
  async fixAuthStore() {
    if (typeof window !== 'undefined' && window.__pinia_stores__) {
      const authStore = window.__pinia_stores__.auth();
      if (authStore && typeof authStore.initialize === 'function') {
        await authStore.initialize();
        console.log('[Auth Diagnostics] Auth store reinitialized');
      }
    }
  }

  /**
   * Display results in console
   */
  displayResults(result) {
    console.group('🔬 Authentication Diagnostics Results');

    console.log('Summary:');
    console.table(result.summary);

    if (result.recommendations.length > 0) {
      console.log('Recommendations:');
      result.recommendations.forEach((rec, idx) => {
        const icon = rec.priority === 'critical' ? '🚨' : rec.priority === 'high' ? '⚠️' : 'ℹ️';
        console.log(`  ${idx + 1}. ${icon} ${rec.issue} → ${rec.solution}`);
      });
    }

    if (result.autoFixable.length > 0) {
      console.log('Auto-fixable issues found:', result.autoFixable);
      console.log('Run fixDAuth() to attempt automatic repairs');
    }

    console.groupEnd();
  }
}

// Create global instance
const authDiagnostics = new AuthDiagnostics();

// Expose to window for console access
if (typeof window !== 'undefined') {
  window.authDiagnostics = authDiagnostics;

  // Convenience commands
  window.diagnoseDAuth = () => authDiagnostics.runDiagnostics();
  window.fixDAuth = () => authDiagnostics.autoFix();
}

export default authDiagnostics; 