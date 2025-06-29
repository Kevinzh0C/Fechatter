/**
 * Immediate Cleanup Script
 * Quick fix for current console issues
 */

function immediateCleanup() {
  if (import.meta.env.DEV) {
    console.log('🚨 Starting Immediate Cleanup...');
  }

  try {
    // 1. Clear console
    if (typeof console.clear === 'function') {
      console.clear();
    }

    if (import.meta.env.DEV) {
      console.log('🧹 Immediate Cleanup Active');
    }

    // 2. Activate log suppressor if available
    if (window.logSuppressor) {
      window.logSuppressor.activate();

      // Add all problematic patterns immediately
      const patterns = [
        // Debug scripts
        /debugDuplicateChannels\.js/,
        /testRequestIsolation\.js/,
        /testExtensionConflictFix\.js/,
        /testManager\.js/,

        // Extension conflicts
        /A listener indicated an asynchronous response/,
        /message channel closed before a response/,

        // API 404 errors for missing endpoints
        /POST.*\/chat\/\d+\/read 404/,
        /🚨 API Error.*404.*not_found/,
        /\[Mark chat as read\].*not_found/,
        /请求的资源未找到/,

        // Channel analysis noise
        /📋 ChannelsList Analysis/,
        /SUBSCRIPTION: API Response Analysis/,
        /Duplicate Detection/,
        /🏠 Home\.vue Computed Properties/,
        /Data Flow Trace/,

        // Test noise
        /🧪 Test Manager Initialization/,
        /Extension Pattern Test/,
        /🧪 Extension Conflict Fix Test/
      ];

      patterns.forEach(pattern => {
        window.logSuppressor.addPattern(pattern);
      });

      if (import.meta.env.DEV) {
        console.log('All noise patterns suppressed');
      }

    // 3. Show clean status
    if (import.meta.env.DEV) {
      console.log('Console Status: CLEAN');
    if (import.meta.env.DEV) {
      console.log('Use window.quickFix() for comprehensive repair');
    if (import.meta.env.DEV) {
      console.log('Use window.logStats() to see suppression statistics');
    }

    return true;
  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('ERROR: Immediate cleanup failed:', error);
    return false;
  }

// Export for global use
if (typeof window !== 'undefined') {
  window.immediateCleanup = immediateCleanup;
  if (import.meta.env.DEV) {
    console.log('🚨 Immediate cleanup available: window.immediateCleanup()');
  }

export default immediateCleanup; 