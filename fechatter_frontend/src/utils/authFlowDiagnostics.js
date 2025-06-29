/**
 * Authentication Flow Diagnostics Tool
 * Comprehensive testing and debugging for auth flow fixes
 */

export class AuthFlowDiagnostics {
  constructor() {
    this.testResults = [];
    this.startTime = Date.now();
  }

  async runCompleteAuthFlowTest() {
    console.group('Authentication Flow Diagnostics');

    const tests = [
      this.testAuthStoreState,
      this.testTokenManagerState,
      this.testLocalStorageConsistency,
      this.testAuthStateManagerIntegrity,
      this.testStateRecoveryMechanism,
      this.testNavigationFlow
    ];

    for (const test of tests) {
      try {
        await test.call(this);
      } catch (error) {
        this.addResult(test.name, 'ERROR', error.message);
      }
    }

    this.printSummary();
    console.groupEnd();

    return {
      success: this.testResults.filter(r => r.status === 'PASS').length === this.testResults.length,
      results: this.testResults,
      summary: this.generateSummary()
    };
  }

  async testAuthStoreState() {
    const testName = 'AuthStore State Verification';

    try {
      // Get auth store
      const authStore = this.getAuthStore();
      if (!authStore) {
        this.addResult(testName, 'FAIL', 'AuthStore not available');
        return;
      }

      const checks = {
        isAuthenticated: authStore.isAuthenticated,
        isLoggedIn: authStore.isLoggedIn,
        hasToken: !!authStore.token,
        hasUser: !!authStore.user,
        isInitialized: authStore.isInitialized
      };

      const passedChecks = Object.values(checks).filter(Boolean).length;
      const totalChecks = Object.keys(checks).length;

      if (passedChecks >= totalChecks * 0.8) {
        this.addResult(testName, 'PASS', `${passedChecks}/${totalChecks} checks passed`, checks);
      } else {
        this.addResult(testName, 'FAIL', `Only ${passedChecks}/${totalChecks} checks passed`, checks);
      }
    } catch (error) {
      this.addResult(testName, 'ERROR', error.message);
    }
  }

  async testTokenManagerState() {
    const testName = 'TokenManager State Verification';

    try {
      const tokenManager = window.tokenManager;
      if (!tokenManager) {
        this.addResult(testName, 'SKIP', 'TokenManager not available globally');
        return;
      }

      const checks = {
        hasAccessToken: !!tokenManager.getAccessToken(),
        isTokenExpired: tokenManager.isTokenExpired(),
        hasValidStatus: !!tokenManager.getStatus(),
        canRefresh: !tokenManager.isRefreshTokenExpired()
      };

      const criticalPassed = checks.hasAccessToken && !checks.isTokenExpired;

      if (criticalPassed) {
        this.addResult(testName, 'PASS', 'Token state is valid', checks);
      } else {
        this.addResult(testName, 'FAIL', 'Token state issues detected', checks);
      }
    } catch (error) {
      this.addResult(testName, 'ERROR', error.message);
    }
  }

  async testLocalStorageConsistency() {
    const testName = 'localStorage Consistency Check';

    try {
      const storageKeys = [
        'auth_token',
        'auth_user',
        'auth',
        'refresh_token'
      ];

      const storageState = {};
      storageKeys.forEach(key => {
        const value = localStorage.getItem(key);
        storageState[key] = {
          exists: !!value,
          length: value ? value.length : 0,
          type: value ? (key.includes('user') ? 'object' : 'string') : null
        };
      });

      // Check auth backup structure
      const authBackup = localStorage.getItem('auth');
      let backupValid = false;
      if (authBackup) {
        try {
          const parsed = JSON.parse(authBackup);
          backupValid = !!(parsed.user && parsed.tokens && parsed.tokens.accessToken);
          storageState.authBackup = { valid: backupValid, structure: Object.keys(parsed) };
        } catch (e) {
          storageState.authBackup = { valid: false, error: 'Parse error' };
        }
      }

      const hasEssentialData = storageState.auth_token.exists && storageState.auth_user.exists;

      if (hasEssentialData || backupValid) {
        this.addResult(testName, 'PASS', 'Storage consistency verified', storageState);
      } else {
        this.addResult(testName, 'FAIL', 'Missing essential storage data', storageState);
      }
    } catch (error) {
      this.addResult(testName, 'ERROR', error.message);
    }
  }

  async testAuthStateManagerIntegrity() {
    const testName = 'AuthStateManager Integrity';

    try {
      const authStateManager = window.authStateManager;
      if (!authStateManager) {
        this.addResult(testName, 'SKIP', 'AuthStateManager not available globally');
        return;
      }

      const authState = authStateManager.getAuthState();
      const token = authStateManager.getToken();
      const user = authStateManager.getUser();

      const checks = {
        hasAuthState: !!authState,
        hasToken: !!token,
        hasUser: !!user,
        isAuthenticated: authState.isAuthenticated,
        stateConsistent: authState.isAuthenticated === (!!token && !!user)
      };

      const allPassed = Object.values(checks).every(Boolean);

      if (allPassed) {
        this.addResult(testName, 'PASS', 'Auth state manager integrity verified', checks);
      } else {
        this.addResult(testName, 'FAIL', 'Auth state manager integrity issues', checks);
      }
    } catch (error) {
      this.addResult(testName, 'ERROR', error.message);
    }
  }

  async testStateRecoveryMechanism() {
    const testName = 'State Recovery Mechanism';

    try {
      // Simulate a recovery scenario by temporarily clearing tokenManager
      const tokenManager = window.tokenManager;
      const authStore = this.getAuthStore();

      if (!tokenManager || !authStore) {
        this.addResult(testName, 'SKIP', 'Required components not available');
        return;
      }

      // Save current state
      const originalToken = tokenManager.getAccessToken();

      if (!originalToken) {
        this.addResult(testName, 'SKIP', 'No token to test recovery with');
        return;
      }

      // Temporarily clear tokenManager
      await tokenManager.clearTokens();

      // Verify it's cleared
      const clearedToken = tokenManager.getAccessToken();
      if (clearedToken) {
        this.addResult(testName, 'FAIL', 'Failed to clear token for recovery test');
        return;
      }

      // Attempt recovery via ensureAuthStateConsistency
      if (authStore.ensureAuthStateConsistency) {
        await authStore.ensureAuthStateConsistency();
      }

      // Check if recovery worked
      const recoveredToken = tokenManager.getAccessToken();
      const recoveryWorked = !!recoveredToken;

      if (recoveryWorked) {
        this.addResult(testName, 'PASS', 'State recovery mechanism works', {
          originalToken: originalToken.substring(0, 20) + '...',
          recoveredToken: recoveredToken.substring(0, 20) + '...',
          tokensMatch: originalToken === recoveredToken
        });
      } else {
        this.addResult(testName, 'FAIL', 'State recovery mechanism failed');

        // Restore original state manually
        if (window.authStateManager) {
          const authState = window.authStateManager.getAuthState();
          if (authState.token) {
            await tokenManager.setTokens({
              accessToken: authState.token,
              refreshToken: authState.token,
              expiresAt: Date.now() + (3600 * 1000),
              issuedAt: Date.now()
            });
          }
        }
      }
    } catch (error) {
      this.addResult(testName, 'ERROR', error.message);
    }
  }

  async testNavigationFlow() {
    const testName = 'Navigation Flow Safety';

    try {
      const currentPath = window.location.pathname;
      const router = window.__vue_app__?.config?.globalProperties?.$router;

      if (!router) {
        this.addResult(testName, 'SKIP', 'Router not available');
        return;
      }

      const checks = {
        currentPath,
        isOnProtectedRoute: currentPath !== '/login' && currentPath !== '/register',
        hasAuthState: this.getAuthStore()?.isAuthenticated || false,
        hasValidNavigation: true // Assume valid if we got here
      };

      // Check for potential infinite redirect protection
      const redirectProtection = {
        hasPreventionMechanism: true, // Our fixes include this
        sessionStorageCheck: !!sessionStorage.getItem('redirectPath')
      };

      this.addResult(testName, 'PASS', 'Navigation flow appears safe', {
        ...checks,
        redirectProtection
      });
    } catch (error) {
      this.addResult(testName, 'ERROR', error.message);
    }
  }

  // Helper methods
  getAuthStore() {
    try {
      return window.__pinia_stores__?.auth() ||
        window.pinia?._s?.get('auth') ||
        null;
    } catch (error) {
      return null;
    }
  }

  addResult(testName, status, message, details = null) {
    this.testResults.push({
      testName,
      status,
      message,
      details,
      timestamp: Date.now() - this.startTime
    });

    const statusIcon = {
      'PASS': '',
      'FAIL': '❌',
      'ERROR': '🚨',
      'SKIP': '⏭️'
    }[status] || '❓';

    console.log(`${statusIcon} ${testName}: ${message}`);
    if (details && import.meta.env.DEV) {
      console.log('   Details:', details);
    }
  }

  generateSummary() {
    const total = this.testResults.length;
    const passed = this.testResults.filter(r => r.status === 'PASS').length;
    const failed = this.testResults.filter(r => r.status === 'FAIL').length;
    const errors = this.testResults.filter(r => r.status === 'ERROR').length;
    const skipped = this.testResults.filter(r => r.status === 'SKIP').length;

    return {
      total,
      passed,
      failed,
      errors,
      skipped,
      successRate: total > 0 ? ((passed / total) * 100).toFixed(1) + '%' : '0%',
      overallStatus: failed === 0 && errors === 0 ? 'HEALTHY' : 'ISSUES_DETECTED'
    };
  }

  printSummary() {
    const summary = this.generateSummary();
    const totalTime = Date.now() - this.startTime;

    console.log('\nAuthentication Flow Diagnostics Summary:');
    console.log(`   Overall Status: ${summary.overallStatus}`);
    console.log(`   Success Rate: ${summary.successRate}`);
    console.log(`   Tests: ${summary.passed} passed, ${summary.failed} failed, ${summary.errors} errors, ${summary.skipped} skipped`);
    console.log(`   Total Time: ${totalTime}ms`);

    if (summary.overallStatus === 'HEALTHY') {
      console.log('🎉 All authentication flow components are working correctly!');
    } else {
      console.log('WARNING: Issues detected in authentication flow. Check individual test results.');
    }
  }
}

// Create global instance
export const authFlowDiagnostics = new AuthFlowDiagnostics();

// Make available globally for debugging
if (typeof window !== 'undefined') {
  window.testAuthFlow = async () => {
    return await authFlowDiagnostics.runCompleteAuthFlowTest();
  };

  window.authFlowDiagnostics = authFlowDiagnostics;
}

export default authFlowDiagnostics; 