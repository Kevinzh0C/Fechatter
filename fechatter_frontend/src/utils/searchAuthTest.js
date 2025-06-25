/**
 * Search Authentication Test
 * Tests the search functionality authentication flow
 */

export async function testSearchAuth() {
  console.log('🧪 Starting Search Authentication Test');
  console.log('=====================================');
  
  try {
    // Get auth store
    const { useAuthStore } = await import('@/stores/auth');
    const authStore = useAuthStore();
    
    // Get current auth state
    console.log('\n📊 Current Auth State:');
    console.log('- isAuthenticated:', authStore.isAuthenticated);
    console.log('- hasToken:', !!authStore.token);
    console.log('- isTokenExpired:', authStore.isTokenExpired);
    
    // Check tokenManager
    console.log('\n🔐 TokenManager State:');
    const tokenManagerStatus = window.tokenManager?.getStatus();
    console.log('- hasToken:', tokenManagerStatus?.hasToken);
    console.log('- isExpired:', tokenManagerStatus?.isExpired);
    console.log('- shouldRefresh:', tokenManagerStatus?.shouldRefresh);
    
    // Check authStateManager
    console.log('\n🗄️ AuthStateManager State:');
    const authState = window.authStateManager?.getAuthState();
    console.log('- hasToken:', authState?.hasToken);
    console.log('- hasUser:', authState?.hasUser);
    console.log('- isAuthenticated:', authState?.isAuthenticated);
    
    // Test consistency
    console.log('\n🔧 Testing Auth State Consistency:');
    await authStore.ensureAuthStateConsistency();
    
    // Re-check after consistency
    console.log('\n📊 Auth State After Consistency Check:');
    console.log('- isAuthenticated:', authStore.isAuthenticated);
    console.log('- hasToken:', !!authStore.token);
    
    // Simulate search button click logic
    console.log('\n🔍 Simulating Search Button Click:');
    if (!authStore.isAuthenticated && !authStore.token) {
      console.log('❌ Would redirect to login');
    } else {
      console.log('✅ Would open search modal');
    }
    
    // Test API call
    console.log('\n📡 Testing API Call with Auth:');
    try {
      const { SearchService } = await import('@/services/api');
      const response = await SearchService.search({
        query: 'test',
        chatId: 1,
        limit: 1
      });
      console.log('✅ API call successful');
    } catch (error) {
      if (error.response?.status === 401) {
        console.log('❌ API call failed with 401');
      } else {
        console.log('❌ API call failed:', error.message);
      }
    }
    
    console.log('\n✅ Test completed');
    
  } catch (error) {
    console.error('❌ Test failed:', error);
  }
}

// Add to window for easy access
if (typeof window !== 'undefined') {
  window.testSearchAuth = testSearchAuth;
}

export default { testSearchAuth };