/**
 * Search Authentication Test
 * Tests the search functionality authentication flow
 */

export async function testSearchAuth() {
  if (import.meta.env.DEV) {
    console.log('🧪 Starting Search Authentication Test');
  if (import.meta.env.DEV) {
    console.log('=====================================');
  }
  
  try {
    // Get auth store
    const { useAuthStore } = await import('@/stores/auth');
    const authStore = useAuthStore();
    
    // Get current auth state
    if (import.meta.env.DEV) {
      console.log('\nCurrent Auth State:');
    if (import.meta.env.DEV) {
      console.log('- isAuthenticated:', authStore.isAuthenticated);
    if (import.meta.env.DEV) {
      console.log('- hasToken:', !!authStore.token);
    if (import.meta.env.DEV) {
      console.log('- isTokenExpired:', authStore.isTokenExpired);
    }
    
    // Check tokenManager
    if (import.meta.env.DEV) {
      console.log('\n🔐 TokenManager State:');
    const tokenManagerStatus = window.tokenManager?.getStatus();
    if (import.meta.env.DEV) {
      console.log('- hasToken:', tokenManagerStatus?.hasToken);
    if (import.meta.env.DEV) {
      console.log('- isExpired:', tokenManagerStatus?.isExpired);
    if (import.meta.env.DEV) {
      console.log('- shouldRefresh:', tokenManagerStatus?.shouldRefresh);
    }
    
    // Check authStateManager
    if (import.meta.env.DEV) {
      console.log('\n🗄️ AuthStateManager State:');
    const authState = window.authStateManager?.getAuthState();
    if (import.meta.env.DEV) {
      console.log('- hasToken:', authState?.hasToken);
    if (import.meta.env.DEV) {
      console.log('- hasUser:', authState?.hasUser);
    if (import.meta.env.DEV) {
      console.log('- isAuthenticated:', authState?.isAuthenticated);
    }
    
    // Test consistency
    if (import.meta.env.DEV) {
      console.log('\nTesting Auth State Consistency:');
    await authStore.ensureAuthStateConsistency();
    
    // Re-check after consistency
    if (import.meta.env.DEV) {
      console.log('\nAuth State After Consistency Check:');
    if (import.meta.env.DEV) {
      console.log('- isAuthenticated:', authStore.isAuthenticated);
    if (import.meta.env.DEV) {
      console.log('- hasToken:', !!authStore.token);
    }
    
    // Simulate search button click logic
    if (import.meta.env.DEV) {
      console.log('\nSimulating Search Button Click:');
    if (!authStore.isAuthenticated && !authStore.token) {
      if (import.meta.env.DEV) {
        console.log('ERROR: Would redirect to login');
      }
    } else {
      if (import.meta.env.DEV) {
        console.log('Would open search modal');
      }
    
    // Test API call
    if (import.meta.env.DEV) {
      console.log('\nSUBSCRIPTION: Testing API Call with Auth:');
    try {
      const { SearchService } = await import('@/services/api');
      const response = await SearchService.search({
        query: 'test',
        chatId: 1,
        limit: 1
      });
      if (import.meta.env.DEV) {
        console.log('API call successful');
      }
    } catch (error) {
      if (error.response?.status === 401) {
        if (import.meta.env.DEV) {
          console.log('ERROR: API call failed with 401');
        }
      } else {
        if (import.meta.env.DEV) {
          console.log('ERROR: API call failed:', error.message);
        }
    
    if (import.meta.env.DEV) {
      console.log('\nTest completed');
    }
    
  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('ERROR: Test failed:', error);
    }

// Add to window for easy access
if (typeof window !== 'undefined') {
  window.testSearchAuth = testSearchAuth;
}

export default { testSearchAuth };