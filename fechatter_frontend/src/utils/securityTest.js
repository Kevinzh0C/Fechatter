/**
 * Security Test Utility
 * Tests JWT token security and prevents console exposure
 */

import { secureLogger } from './secureLogger';

export class SecurityTest {
  constructor() {
    this.testResults = [];
  }

  /**
   * Test JWT token sanitization
   */
  testJWTSanitization() {
    console.group('🔒 JWT Security Test');

    const testToken = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c';
    const testBearerToken = `Bearer ${testToken}`;
    const testURLWithToken = `http://example.com/events?access_token=${testToken}`;

    console.log('🧪 Testing JWT token sanitization...');

    // Test 1: Direct token
    console.log('Test 1 - Direct JWT token:');
    secureLogger.log('Token:', testToken);

    // Test 2: Bearer token
    console.log('Test 2 - Bearer token:');
    secureLogger.log('Authorization:', testBearerToken);

    // Test 3: URL with token
    console.log('Test 3 - URL with token:');
    secureLogger.log('SSE URL:', testURLWithToken);

    // Test 4: Object with nested token
    console.log('Test 4 - Object with nested token:');
    const testObject = {
      user: { id: 123, name: 'John' },
      auth: { token: testToken, type: 'Bearer' },
      url: testURLWithToken
    };
    secureLogger.log('Auth object:', testObject);

    // Test 5: Array with tokens
    console.log('Test 5 - Array with tokens:');
    const testArray = [testToken, testBearerToken, testURLWithToken];
    secureLogger.log('Token array:', testArray);

    console.log('✅ JWT sanitization test completed');
    console.groupEnd();
  }

  /**
   * Test SSE URL sanitization
   */
  testSSEURLSanitization() {
    console.group('🔒 SSE URL Security Test');

    const testToken = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c';

    // Test SSE URL patterns for token exposure
    const suspiciousUrls = [
      `/events?access_token=${testToken}`,
      `/events?token=${testToken}`,
    ];

    console.log('🧪 Testing SSE URL sanitization...');

    suspiciousUrls.forEach((url, index) => {
      console.log(`Test ${index + 1} - SSE URL:`);
      secureLogger.log(`Connecting to: ${url}`);
    });

    console.log('✅ SSE URL sanitization test completed');
    console.groupEnd();
  }

  /**
   * Run all security tests
   */
  runAllTests() {
    console.group('🛡️ Security Test Suite');
    console.log('Running comprehensive security tests...');

    this.testJWTSanitization();
    this.testSSEURLSanitization();

    console.log('🎯 All security tests completed');
    console.log('💡 Check console output above - no actual tokens should be visible');
    console.groupEnd();

    return {
      success: true,
      testsRun: 2,
      message: 'Security tests completed - review console output'
    };
  }
}

// Global instance
export const securityTest = new SecurityTest();

// Expose to window for manual testing
if (typeof window !== 'undefined') {
  window.testSecurity = () => securityTest.runAllTests();
}

export default securityTest; 