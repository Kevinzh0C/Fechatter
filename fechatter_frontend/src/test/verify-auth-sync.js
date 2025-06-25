/**
 * Auth State Sync Verification Test
 * Tests that authStateManager remains the single source of truth
 */

export async function verifyAuthStateSync() {
  console.log('🧪 Starting Auth State Sync Verification...\n');

  try {
    // Import required modules
    const { useAuthStore } = await import('@/stores/auth');
    const { default: authStateManager } = await import('@/utils/authStateManager');
    const { default: tokenManager } = await import('@/services/tokenManager');

    const authStore = useAuthStore();

    // Test 1: Check initial state
    console.log('📋 Test 1: Initial State Check');
    const authState = authStateManager.getAuthState();
    const tokenManagerState = tokenManager.getStatus();

    console.log('Auth State:', {
      hasToken: authState.hasToken,
      hasUser: authState.hasUser,
      isAuthenticated: authState.isAuthenticated
    });

    console.log('Token Manager State:', {
      hasToken: tokenManagerState.hasToken,
      isExpired: tokenManagerState.isExpired
    });

    console.log('Auth Store State:', {
      isAuthenticated: authStore.isAuthenticated,
      hasUser: !!authStore.user,
      hasToken: !!authStore.token
    });

    // Test 2: Verify consistency
    console.log('\n📋 Test 2: State Consistency Check');
    const isConsistent =
      authState.isAuthenticated === authStore.isAuthenticated &&
      authState.isAuthenticated === tokenManagerState.hasToken &&
      authState.token === authStore.token;

    if (isConsistent) {
      console.log('✅ State is consistent across all systems');
    } else {
      console.error('❌ State inconsistency detected!');
      console.log('Details:', {
        'authStateManager.isAuthenticated': authState.isAuthenticated,
        'authStore.isAuthenticated': authStore.isAuthenticated,
        'tokenManager.hasToken': tokenManagerState.hasToken,
        'authStateManager.token': !!authState.token,
        'authStore.token': !!authStore.token
      });
    }

    // Test 3: Debug the specific issue
    console.log('\n📋 Test 3: Debug Specific Issue');
    if (authStore.isAuthenticated && !authState.token) {
      console.error('❌ Found the issue: isAuthenticated=true but no token!');
      console.log('This should never happen with our fixes.');
    } else if (!authStore.isAuthenticated && authState.token) {
      console.error('❌ Found opposite issue: has token but isAuthenticated=false!');
    } else {
      console.log('✅ No authentication state inconsistency found');
    }

    // Test 4: Verify authStateManager is the source of truth
    console.log('\n📋 Test 4: Source of Truth Verification');
    console.log('authStateManager should be the single source of truth');
    console.log('Auth decisions should be based on token existence');
    console.log(`Current decision: isAuthenticated=${authState.isAuthenticated} (hasToken=${authState.hasToken})`);

    return {
      success: isConsistent,
      authState,
      tokenManagerState,
      authStoreState: {
        isAuthenticated: authStore.isAuthenticated,
        hasUser: !!authStore.user,
        hasToken: !!authStore.token
      }
    };

  } catch (error) {
    console.error('❌ Test failed with error:', error);
    return {
      success: false,
      error: error.message
    };
  }
}

// Auto-run if executed directly
if (import.meta.env.DEV) {
  console.log('🚀 Running auth state sync verification...');
  verifyAuthStateSync().then(result => {
    console.log('\n📊 Test Results:', result);
  });
}

// Export for console usage
window.verifyAuthStateSync = verifyAuthStateSync; 