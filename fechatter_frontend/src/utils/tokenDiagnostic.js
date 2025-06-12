/**
 * Token Diagnostic Tool
 * Analyze token validity and 401 authentication errors
 */

export class TokenDiagnostic {
  constructor() {
    this.results = [];
  }

  /**
   * Run comprehensive token diagnostics
   */
  async runDiagnostics() {
    console.log('🔐 Starting Token Diagnostics...');
    this.results = [];

    try {
      // Test 1: Check token manager state
      await this.checkTokenManagerState();

      // Test 2: Analyze current tokens
      await this.analyzeCurrentTokens();

      // Test 3: Test token with auth API
      await this.testTokenWithAuthAPI();

      // Test 4: Test SSE endpoint specifically
      await this.testSSEEndpoint();

      // Test 5: Token refresh test
      await this.testTokenRefresh();

    } catch (error) {
      this.addResult('ERROR', 'Diagnostic execution failed', error.message);
    }

    this.printReport();
    return this.results;
  }

  /**
   * Check token manager state
   */
  async checkTokenManagerState() {
    try {
      const { default: tokenManager } = await import('@/services/tokenManager');

      const tokens = tokenManager.getTokens();
      const isExpired = tokenManager.isTokenExpired();
      const shouldRefresh = tokenManager.shouldRefreshToken();

      this.addResult('INFO', 'Token Manager', `Has Access Token: ${!!tokens.accessToken}`);
      this.addResult('INFO', 'Token Manager', `Has Refresh Token: ${!!tokens.refreshToken}`);
      this.addResult('INFO', 'Token Manager', `Is Expired: ${isExpired}`);
      this.addResult('INFO', 'Token Manager', `Should Refresh: ${shouldRefresh}`);

      if (tokens.expiresAt) {
        const timeLeft = Math.max(0, tokens.expiresAt - Date.now());
        this.addResult('INFO', 'Token Expiry', `Time left: ${Math.floor(timeLeft / 1000)}s`);
      }

    } catch (error) {
      this.addResult('ERROR', 'Token Manager Check', error.message);
    }
  }

  /**
   * Analyze current tokens
   */
  async analyzeCurrentTokens() {
    try {
      const { default: tokenManager } = await import('@/services/tokenManager');
      const tokens = tokenManager.getTokens();

      if (tokens.accessToken) {
        // Decode JWT payload (without verification)
        const payload = this.decodeJWTPayload(tokens.accessToken);

        if (payload) {
          this.addResult('INFO', 'JWT Payload', `Subject: ${payload.sub || 'N/A'}`);
          this.addResult('INFO', 'JWT Payload', `Issued At: ${new Date(payload.iat * 1000).toISOString()}`);
          this.addResult('INFO', 'JWT Payload', `Expires At: ${new Date(payload.exp * 1000).toISOString()}`);

          const now = Math.floor(Date.now() / 1000);
          if (payload.exp < now) {
            this.addResult('ERROR', 'JWT Analysis', 'Token is expired according to JWT payload');
          } else {
            this.addResult('SUCCESS', 'JWT Analysis', 'Token is valid according to JWT payload');
          }
        } else {
          this.addResult('ERROR', 'JWT Analysis', 'Failed to decode JWT payload');
        }

        // Check token format
        const tokenParts = tokens.accessToken.split('.');
        if (tokenParts.length !== 3) {
          this.addResult('ERROR', 'Token Format', `Invalid JWT format: ${tokenParts.length} parts instead of 3`);
        } else {
          this.addResult('SUCCESS', 'Token Format', 'Valid JWT format (3 parts)');
        }

        // Check token length (reasonable bounds)
        if (tokens.accessToken.length < 100) {
          this.addResult('WARNING', 'Token Length', `Token seems too short: ${tokens.accessToken.length} chars`);
        } else if (tokens.accessToken.length > 2000) {
          this.addResult('WARNING', 'Token Length', `Token seems too long: ${tokens.accessToken.length} chars`);
        } else {
          this.addResult('INFO', 'Token Length', `${tokens.accessToken.length} characters`);
        }

      } else {
        this.addResult('ERROR', 'Token Analysis', 'No access token found');
      }

    } catch (error) {
      this.addResult('ERROR', 'Token Analysis', error.message);
    }
  }

  /**
   * Test token with auth API
   */
  async testTokenWithAuthAPI() {
    try {
      const { default: api } = await import('@/services/api');

      // Test with user profile endpoint
      const response = await api.get('/user/profile');

      if (response.status === 200) {
        this.addResult('SUCCESS', 'API Auth Test', 'Token works with /user/profile');
      } else {
        this.addResult('WARNING', 'API Auth Test', `Unexpected status: ${response.status}`);
      }

    } catch (error) {
      if (error.response?.status === 401) {
        this.addResult('ERROR', 'API Auth Test', 'Token rejected by /user/profile (401)');
      } else {
        this.addResult('ERROR', 'API Auth Test', error.message);
      }
    }
  }

  /**
   * Test SSE endpoint specifically
   */
  async testSSEEndpoint() {
    try {
      const { default: tokenManager } = await import('@/services/tokenManager');
      const tokens = tokenManager.getTokens();

      if (!tokens.accessToken) {
        this.addResult('ERROR', 'SSE Test', 'No token available for SSE test');
        return;
      }

      // Test SSE endpoint with current token
      const { getApiConfig } = await import('@/utils/configLoader');
      const apiConfig = getApiConfig();
      const sseUrl = apiConfig.sse_url || '/events';

      const testUrl = `${sseUrl}?access_token=${encodeURIComponent(tokens.accessToken)}`;

      try {
        const response = await fetch(testUrl, {
          method: 'GET',
          headers: {
            'Accept': 'text/event-stream',
            'Cache-Control': 'no-cache'
          }
        });

        if (response.status === 200) {
          this.addResult('SUCCESS', 'SSE Endpoint Test', 'Token accepted by SSE endpoint');
        } else if (response.status === 401) {
          this.addResult('ERROR', 'SSE Endpoint Test', 'Token rejected by SSE endpoint (401)');

          // Get response body for more details
          const errorText = await response.text();
          this.addResult('ERROR', 'SSE Error Details', errorText.slice(0, 200));
        } else {
          this.addResult('WARNING', 'SSE Endpoint Test', `Unexpected status: ${response.status}`);
        }

      } catch (fetchError) {
        this.addResult('ERROR', 'SSE Endpoint Test', `Fetch failed: ${fetchError.message}`);
      }

    } catch (error) {
      this.addResult('ERROR', 'SSE Test Setup', error.message);
    }
  }

  /**
   * Test token refresh
   */
  async testTokenRefresh() {
    try {
      const { default: tokenManager } = await import('@/services/tokenManager');

      const tokensBefore = tokenManager.getTokens();

      if (!tokensBefore.refreshToken) {
        this.addResult('WARNING', 'Refresh Test', 'No refresh token available');
        return;
      }

      this.addResult('INFO', 'Refresh Test', 'Attempting token refresh...');

      try {
        await tokenManager.refreshToken();

        const tokensAfter = tokenManager.getTokens();

        if (tokensAfter.accessToken !== tokensBefore.accessToken) {
          this.addResult('SUCCESS', 'Refresh Test', 'Token refreshed successfully');
        } else {
          this.addResult('WARNING', 'Refresh Test', 'Token refresh returned same token');
        }

      } catch (refreshError) {
        this.addResult('ERROR', 'Refresh Test', `Refresh failed: ${refreshError.message}`);
      }

    } catch (error) {
      this.addResult('ERROR', 'Refresh Test Setup', error.message);
    }
  }

  /**
   * Decode JWT payload (without verification)
   */
  decodeJWTPayload(token) {
    try {
      const parts = token.split('.');
      if (parts.length !== 3) return null;

      const payload = parts[1];
      // Add padding if needed
      const paddedPayload = payload + '='.repeat((4 - payload.length % 4) % 4);
      const decoded = atob(paddedPayload);
      return JSON.parse(decoded);

    } catch (error) {
      console.warn('Failed to decode JWT payload:', error);
      return null;
    }
  }

  /**
   * Add diagnostic result
   */
  addResult(level, category, message) {
    const result = {
      timestamp: new Date().toISOString(),
      level,
      category,
      message
    };
    this.results.push(result);

    const emoji = {
      'SUCCESS': '✅',
      'INFO': 'ℹ️',
      'WARNING': '⚠️',
      'ERROR': '❌'
    }[level] || '🔸';

    console.log(`${emoji} [${category}] ${message}`);
  }

  /**
   * Print diagnostic report
   */
  printReport() {
    console.log('\n🔐 =============== Token Diagnostic Report ===============');

    const summary = {
      SUCCESS: 0,
      INFO: 0,
      WARNING: 0,
      ERROR: 0
    };

    this.results.forEach(result => {
      summary[result.level]++;
    });

    console.log(`✅ Success: ${summary.SUCCESS}`);
    console.log(`ℹ️ Info: ${summary.INFO}`);
    console.log(`⚠️ Warnings: ${summary.WARNING}`);
    console.log(`❌ Errors: ${summary.ERROR}`);

    if (summary.ERROR === 0) {
      console.log('🎉 No token errors detected!');
    } else {
      console.log('🔧 Token issues detected. Check errors above.');
    }

    console.log('==================================================\n');
  }

  /**
   * Quick token check
   */
  async quickCheck() {
    try {
      const { default: tokenManager } = await import('@/services/tokenManager');
      const tokens = tokenManager.getTokens();

      console.log('🔍 Quick Token Check:');
      console.log(`🔑 Has Access Token: ${!!tokens.accessToken}`);
      console.log(`🔄 Has Refresh Token: ${!!tokens.refreshToken}`);
      console.log(`⏰ Is Expired: ${tokenManager.isTokenExpired()}`);
      console.log(`🔄 Should Refresh: ${tokenManager.shouldRefreshToken()}`);

      if (tokens.expiresAt) {
        const timeLeft = Math.max(0, tokens.expiresAt - Date.now());
        console.log(`⏳ Time Left: ${Math.floor(timeLeft / 1000)}s`);
      }

    } catch (error) {
      console.error('❌ Quick check failed:', error);
    }
  }
}

// Create global instance
const tokenDiagnostic = new TokenDiagnostic();

// Expose to window for debugging
if (typeof window !== 'undefined') {
  window.tokenDiagnostic = tokenDiagnostic;
}

export default tokenDiagnostic; 