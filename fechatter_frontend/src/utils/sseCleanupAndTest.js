/**
 * SSE Cleanup and Test Script
 * Comprehensive cleanup of bad states and verification of fixes
 */

export async function cleanupAndTestSSE() {
  console.log('🧹 Starting comprehensive SSE cleanup and test...');

  // Step 1: Complete cleanup
  await performCompleteCleanup();

  // Step 2: Wait for cleanup to complete
  await new Promise(resolve => setTimeout(resolve, 1000));

  // Step 3: Test fixes
  const testResult = await testSSEFixes();

  // Step 4: Report results
  console.log('🎯 Cleanup and test completed!');
  return testResult;
}

/**
 * Perform complete cleanup of SSE state
 */
async function performCompleteCleanup() {
  console.log('🧹 Performing complete SSE cleanup...');

  try {
    // 1. Reset global manager
    if (window.sseGlobalManager) {
      console.log('🔄 Resetting SSE Global Manager...');
      window.sseGlobalManager.reset();
    }

    // 2. Disconnect service
    if (window.realtimeCommunicationService) {
      console.log('🔌 Disconnecting SSE service...');
      window.realtimeCommunicationService.disconnect();

      // Reset retry control
      window.realtimeCommunicationService.resetRetryControl();

      // Clear banned state
      window.realtimeCommunicationService.isGloballyBanned = false;
      window.realtimeCommunicationService.isPaused = false;
    }

    // 3. Clear any orphaned EventSource connections
    console.log('🧹 Cleaning up orphaned connections...');
    // This will be handled by the service disconnect

    console.log('✅ Complete cleanup finished');

  } catch (error) {
    console.error('❌ Error during cleanup:', error);
  }
}

/**
 * Test SSE fixes
 */
async function testSSEFixes() {
  console.log('🧪 Testing SSE fixes...');

  try {
    // Import test utility
    const { default: sseTester } = await import('./sseConnectionTest.js');

    // Run comprehensive tests
    const result = await sseTester.runTests();

    return result;

  } catch (error) {
    console.error('❌ Error during testing:', error);
    return { passed: false, error: error.message };
  }
}

/**
 * Quick diagnostic check
 */
export function quickDiagnostic() {
  console.log('🔍 Quick SSE Diagnostic:');

  // Check global manager
  if (window.sseGlobalManager) {
    const status = window.sseGlobalManager.getStatus();
    console.log(`📊 Global Manager: ${status.connections.length} connections`);

    if (status.sessionStats.permanentlyBanned.length > 0) {
      console.log(`🚫 Banned URLs: ${status.sessionStats.permanentlyBanned.join(', ')}`);
    }

    if (Object.keys(status.sessionStats.bannedUrls).length > 0) {
      console.log(`⏳ Cooling down: ${Object.keys(status.sessionStats.bannedUrls).join(', ')}`);
    }
  }

  // Check service
  if (window.realtimeCommunicationService) {
    const state = window.realtimeCommunicationService.getConnectionState();
    console.log(`🔌 Service: ${state.state} (Connected: ${state.isConnected})`);
    console.log(`🔄 Retries: ${state.retryControl.totalAttempts}/${state.retryControl.maxTotalAttempts}`);

    if (state.retryControl.permanentFailure) {
      console.log('💔 Service permanently failed');
    }
  }
}

/**
 * Emergency reset - use only if everything is broken
 */
export function emergencyReset() {
  console.warn('🚨 EMERGENCY RESET - This will clear all SSE state!');

  // Reset global manager
  if (window.sseGlobalManager) {
    window.sseGlobalManager.reset();
  }

  // Reset service
  if (window.realtimeCommunicationService) {
    const service = window.realtimeCommunicationService;
    service.disconnect();
    service.resetRetryControl();
    service.isGloballyBanned = false;
    service.isPaused = false;
    service.connectionState = 'disconnected';
    service.isConnected = false;
    service.eventSource = null;
  }

  console.log('✅ Emergency reset completed. Try connecting again.');
}

// Expose functions to window for easy access
if (typeof window !== 'undefined') {
  window.cleanupAndTestSSE = cleanupAndTestSSE;
  window.quickDiagnostic = quickDiagnostic;
  window.emergencyReset = emergencyReset;
}

export default {
  cleanupAndTestSSE,
  quickDiagnostic,
  emergencyReset
}; 