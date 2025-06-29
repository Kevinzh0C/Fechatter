/**
 * 🚨 EMERGENCY SSE DIAGNOSTIC SCRIPT
 * Run this directly in browser console to diagnose EventSource error
 */

console.log('🚨 EMERGENCY SSE DIAGNOSIS STARTING...');

// Helper function for styled logging
function emergencyLog(message, type = 'info') {
  const styles = {
    error: 'color: #ff0000; font-weight: bold; background: #330000; padding: 2px 5px;',
    success: 'color: #00ff00; font-weight: bold; background: #003300; padding: 2px 5px;',
    warning: 'color: #ffaa00; font-weight: bold; background: #332200; padding: 2px 5px;',
    info: 'color: #0099ff; font-weight: bold; background: #002233; padding: 2px 5px;'
  };
  console.log(`%c${message}`, styles[type]);
}

// Step 1: Check SSE Service
const minimalSSE = window.minimalSSE || window.realtimeCommunicationService;
if (!minimalSSE) {
  emergencyLog('❌ CRITICAL: SSE service not found on window object', 'error');
  emergencyLog('💡 FIX: Check if sse-minimal.js is loaded', 'warning');
} else {
  emergencyLog('✅ SSE service found', 'success');
  emergencyLog(`📊 Service type: ${minimalSSE.constructor.name}`, 'info');
}

// Step 2: Check Auth Store
const authStore = window.__pinia_stores__?.auth?.() ||
  window.pinia?.stores?.auth ||
  window.authStore;

if (!authStore) {
  emergencyLog('❌ CRITICAL: Auth store not accessible', 'error');
  emergencyLog('💡 FIX: Check if pinia stores are initialized', 'warning');
} else {
  emergencyLog('✅ Auth store found', 'success');
}

// Step 3: Check Authentication & Token
if (authStore) {
  emergencyLog(`🔐 Authenticated: ${authStore.isAuthenticated}`, authStore.isAuthenticated ? 'success' : 'error');
  emergencyLog(`🎫 Token exists: ${!!authStore.token}`, authStore.token ? 'success' : 'error');

  if (authStore.token) {
    emergencyLog(`🔍 Token type: ${typeof authStore.token}`, 'info');
    emergencyLog(`🔍 Token length: ${authStore.token.length}`, 'info');
    emergencyLog(`🔍 Token preview: ${authStore.token.substring(0, 30)}...`, 'info');

    // Validate token format
    if (typeof authStore.token !== 'string' || authStore.token.length < 10) {
      emergencyLog('❌ INVALID TOKEN FORMAT!', 'error');
      emergencyLog('💡 FIX: Token should be a string with length > 10', 'warning');
    } else {
      emergencyLog('✅ Token format valid', 'success');
    }
  } else {
    emergencyLog('❌ NO TOKEN - Cannot proceed with SSE', 'error');
    emergencyLog('💡 FIX: Login first to get authentication token', 'warning');
  }
}

// Step 4: Check Current SSE State
if (minimalSSE) {
  emergencyLog(`📡 SSE connected flag: ${minimalSSE.connected}`, 'info');
  emergencyLog(`🔗 EventSource exists: ${!!minimalSSE.eventSource}`, minimalSSE.eventSource ? 'success' : 'error');

  if (minimalSSE.eventSource) {
    const readyState = minimalSSE.eventSource.readyState;
    const states = ['CONNECTING', 'OPEN', 'CLOSED'];
    const stateName = states[readyState] || 'UNKNOWN';

    emergencyLog(`📊 EventSource readyState: ${readyState} (${stateName})`, 'info');
    emergencyLog(`🌐 EventSource URL: ${minimalSSE.eventSource.url}`, 'info');

    if (readyState === 2) {
      emergencyLog('🚨 ROOT CAUSE: EventSource is CLOSED!', 'error');
      emergencyLog('💡 This means connection failed after creation', 'warning');
      emergencyLog('💡 Check browser Network tab for 4xx/5xx errors', 'warning');
    } else if (readyState === 0) {
      emergencyLog('⚠️ EventSource stuck in CONNECTING state', 'warning');
      emergencyLog('💡 This suggests network or server issues', 'warning');
    } else if (readyState === 1) {
      emergencyLog('✅ EventSource is OPEN - connection working!', 'success');
    }
  } else {
    emergencyLog('❌ ROOT CAUSE: EventSource object not created!', 'error');
    emergencyLog('💡 This means connect() failed before EventSource creation', 'warning');
  }
}

// Step 5: Test Direct EventSource Creation
if (authStore && authStore.token) {
  emergencyLog('🧪 TESTING DIRECT EVENTSOURCE CREATION...', 'warning');

  const token = authStore.token;
  const testUrl = `/events?access_token=${encodeURIComponent(token)}`;

  emergencyLog(`🔗 Test URL: ${testUrl.substring(0, 80)}...`, 'info');

  try {
    const testEventSource = new EventSource(testUrl);
    emergencyLog('✅ Direct EventSource created successfully!', 'success');
    emergencyLog('🔧 Problem is in SSE service logic, not EventSource itself', 'warning');

    testEventSource.onopen = function () {
      emergencyLog('🎉 DIRECT TEST SUCCESS: Connection opened!', 'success');
      emergencyLog('✅ URL and token are working correctly', 'success');
      emergencyLog('🔧 The issue is in minimalSSE.connect() logic', 'warning');
    };

    testEventSource.onerror = function (error) {
      emergencyLog('❌ Direct test also failed - server/network issue', 'error');
      emergencyLog(`📊 Direct test readyState: ${testEventSource.readyState}`, 'error');
    };

    // Store for manual cleanup
    window.emergencyTestES = testEventSource;

    // Auto-close after 5 seconds
    setTimeout(() => {
      testEventSource.close();
      emergencyLog('🔚 Direct test completed', 'info');
    }, 5000);

  } catch (error) {
    emergencyLog(`❌ Direct EventSource creation failed: ${error.message}`, 'error');
    emergencyLog(`🔍 Error type: ${error.name}`, 'error');

    if (error.name === 'SecurityError') {
      emergencyLog('🚨 SECURITY ERROR: CORS or origin issue', 'error');
      emergencyLog('💡 FIX: Check vite proxy configuration', 'warning');
    } else if (error.name === 'SyntaxError') {
      emergencyLog('🚨 SYNTAX ERROR: Invalid URL format', 'error');
      emergencyLog('💡 FIX: Check URL construction logic', 'warning');
    }
  }
}

// Step 6: Provide Fix Recommendations
emergencyLog('🔧 EMERGENCY FIX RECOMMENDATIONS:', 'warning');

if (!minimalSSE) {
  emergencyLog('1. Reload page to ensure SSE service loads', 'warning');
} else if (!authStore || !authStore.token) {
  emergencyLog('1. Login to get authentication token', 'warning');
  emergencyLog('2. Refresh page after login', 'warning');
} else if (minimalSSE.eventSource && minimalSSE.eventSource.readyState === 2) {
  emergencyLog('1. Run: minimalSSE.disconnect()', 'warning');
  emergencyLog('2. Wait 2 seconds', 'warning');
  emergencyLog('3. Run: minimalSSE.connect(authStore.token)', 'warning');
} else if (!minimalSSE.eventSource) {
  emergencyLog('1. Check console for SSE connection logs', 'warning');
  emergencyLog('2. Force connection: minimalSSE.connect(authStore.token)', 'warning');
}

emergencyLog('🚨 DIAGNOSIS COMPLETE - Check recommendations above', 'info');

// Return diagnostic data for further analysis
window.sseEmergencyDiagnostic = {
  sseService: !!minimalSSE,
  authStore: !!authStore,
  authenticated: authStore?.isAuthenticated,
  tokenExists: !!authStore?.token,
  eventSourceExists: !!minimalSSE?.eventSource,
  eventSourceState: minimalSSE?.eventSource?.readyState,
  connected: minimalSSE?.connected,
  timestamp: new Date().toISOString()
};

emergencyLog('📋 Diagnostic data saved to window.sseEmergencyDiagnostic', 'info'); 