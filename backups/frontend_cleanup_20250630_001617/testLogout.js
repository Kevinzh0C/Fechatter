/**
 * Test Logout Functionality
 * 测试退出登录功能的工具
 */

class LogoutTester {
  constructor() {
    this.testResults = [];
  }

  /**
   * Test the complete logout flow
   */
  async testLogoutFlow() {
    if (import.meta.env.DEV) {
      console.log('🧪 [LOGOUT_TEST] Starting logout functionality test...');
    }

    const results = {
      authStoreLogout: false,
      storageCleared: false,
      navigationWorked: false,
      overallSuccess: false,
      errors: []
    };

    try {
      // Test 1: Check if auth store is available
      const authStore = this.getAuthStore();
      if (!authStore) {
        throw new Error('Auth store not available');
      }

      if (import.meta.env.DEV) {
        console.log('✅ [LOGOUT_TEST] Auth store is available');
      }

      // Test 2: Check initial auth state
      const initialState = {
        isAuthenticated: authStore.isAuthenticated,
        hasUser: !!authStore.user,
        hasToken: !!authStore.token
      };

      if (import.meta.env.DEV) {
        console.log('📊 [LOGOUT_TEST] Initial auth state:', initialState);
      }

      // Test 3: Execute logout
      if (import.meta.env.DEV) {
        console.log('🚪 [LOGOUT_TEST] Executing logout...');
      await authStore.logout('Test logout');
      results.authStoreLogout = true;

      // Test 4: Check if storage was cleared
      const storageCleared = this.checkStorageCleared();
      results.storageCleared = storageCleared;

      // Test 5: Check final auth state
      const finalState = {
        isAuthenticated: authStore.isAuthenticated,
        hasUser: !!authStore.user,
        hasToken: !!authStore.token
      };

      if (import.meta.env.DEV) {
        console.log('📊 [LOGOUT_TEST] Final auth state:', finalState);
      }

      // Test 6: Check if navigation worked (indirectly)
      const currentPath = window.location.pathname;
      results.navigationWorked = currentPath === '/login';

      results.overallSuccess = results.authStoreLogout &&
        results.storageCleared &&
        !finalState.isAuthenticated;

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ [LOGOUT_TEST] Test failed:', error);
      results.errors.push(error.message);
    }

    // Display results
    this.displayTestResults(results);
    return results;
  }

  /**
   * Test emergency logout (direct method)
   */
  async testEmergencyLogout() {
    if (import.meta.env.DEV) {
      console.log('🚨 [LOGOUT_TEST] Testing emergency logout...');
    }

    try {
      // Clear all storage
      localStorage.clear();
      sessionStorage.clear();

      // Force navigation
      window.location.replace('/login');

      if (import.meta.env.DEV) {
        console.log('✅ [LOGOUT_TEST] Emergency logout executed');
      return true;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ [LOGOUT_TEST] Emergency logout failed:', error);
      return false;
    }

  /**
   * Check if auth-related storage was cleared
   */
  checkStorageCleared() {
    const authKeys = [
      'auth', 'auth_token', 'fechatter_access_token', 'token_expires_at',
      'fechatter_token_expiry', 'remember_me', 'user', 'token'
    ];

    let clearedCount = 0;
    authKeys.forEach(key => {
      if (!localStorage.getItem(key) && !sessionStorage.getItem(key)) {
        clearedCount++;
      }
    });

    const clearanceRatio = clearedCount / authKeys.length;
    if (import.meta.env.DEV) {
      console.log(`🧹 [LOGOUT_TEST] Storage clearance: ${clearedCount}/${authKeys.length} (${(clearanceRatio * 100).toFixed(1)}%)`);
    }

    return clearanceRatio > 0.8; // 80% or more cleared is considered success
  }

  /**
   * Get auth store instance
   */
  getAuthStore() {
    try {
      // Try multiple methods to get the auth store
      if (window.app?._instance?.proxy?.$pinia?._s?.get('auth')) {
        return window.app._instance.proxy.$pinia._s.get('auth');
      }

      // Alternative method
      if (window.$pinia) {
        return window.$pinia._s.get('auth');
      }

      return null;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to get auth store:', error);
      return null;
    }

  /**
   * Display test results in a formatted way
   */
  displayTestResults(results) {
    if (import.meta.env.DEV) {
      console.log('\n🧪 LOGOUT TEST RESULTS');
    if (import.meta.env.DEV) {
      console.log('======================');
    if (import.meta.env.DEV) {
      console.log(`Auth Store Logout: ${results.authStoreLogout ? '✅' : '❌'}`);
    if (import.meta.env.DEV) {
      console.log(`Storage Cleared: ${results.storageCleared ? '✅' : '❌'}`);
    if (import.meta.env.DEV) {
      console.log(`Navigation Worked: ${results.navigationWorked ? '✅' : '❌'}`);
    if (import.meta.env.DEV) {
      console.log(`Overall Success: ${results.overallSuccess ? '✅' : '❌'}`);
    }

    if (results.errors.length > 0) {
      if (import.meta.env.DEV) {
        console.log('\nErrors:');
      results.errors.forEach(error => console.log(`  ❌ ${error}`));
    }

    if (import.meta.env.DEV) {
      console.log('\n');
    }

  /**
   * Quick logout test (simplified)
   */
  async quickTest() {
    if (import.meta.env.DEV) {
      console.log('⚡ [LOGOUT_TEST] Quick logout test...');
    }

    try {
      const authStore = this.getAuthStore();
      if (authStore && authStore.logout) {
        await authStore.logout();
        if (import.meta.env.DEV) {
          console.log('✅ [LOGOUT_TEST] Quick test passed');
        return true;
      } else {
        if (import.meta.env.DEV) {
          console.error('❌ [LOGOUT_TEST] Auth store or logout method not available');
        return false;
      }
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ [LOGOUT_TEST] Quick test failed:', error);
      return false;
    }

// Create global instance
const logoutTester = new LogoutTester();

// Expose to window for testing
if (typeof window !== 'undefined') {
  window.testLogout = {
    full: () => logoutTester.testLogoutFlow(),
    quick: () => logoutTester.quickTest(),
    emergency: () => logoutTester.testEmergencyLogout(),
    checkStorage: () => logoutTester.checkStorageCleared()
  };

  if (import.meta.env.DEV) {
    console.log('🧪 Logout testing tools loaded:');
  if (import.meta.env.DEV) {
    console.log('  window.testLogout.full() - Complete logout test');
  if (import.meta.env.DEV) {
    console.log('  window.testLogout.quick() - Quick logout test');
  if (import.meta.env.DEV) {
    console.log('  window.testLogout.emergency() - Emergency logout');
  if (import.meta.env.DEV) {
    console.log('  window.testLogout.checkStorage() - Check storage clearance');
  }

export default logoutTester; 