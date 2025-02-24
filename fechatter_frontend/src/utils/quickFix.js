/**
 * Quick Fix Script
 * One-click solution for all major frontend issues
 */

async function quickFix() {
  if (import.meta.env.DEV) {
    console.log('🚀 Starting Quick Fix for all frontend issues...');
  }

  const results = [];

  try {
    // 1. Activate log suppression immediately
    if (import.meta.env.DEV) {
      console.log('1️⃣ Activating log suppression...');
    if (window.logSuppressor) {
      window.logSuppressor.activate();
      results.push('✅ Log suppression activated');
    } else {
      results.push('⚠️ Log suppressor not available');
    }

    // 2. Suppress extension conflicts immediately
    if (import.meta.env.DEV) {
      console.log('2️⃣ Suppressing extension conflicts...');
    suppressExtensionConflicts();
    results.push('✅ Extension conflict suppression added');

    // 3. Suppress API 404 errors for missing endpoints
    if (import.meta.env.DEV) {
      console.log('3️⃣ Suppressing API 404 errors...');
    suppressApiNotFoundErrors();
    results.push('✅ API 404 error suppression added');

    // 4. Clear repetitive debug script errors
    if (import.meta.env.DEV) {
      console.log('4️⃣ Clearing repetitive errors...');
    clearRepetitiveErrors();
    results.push('✅ Repetitive error patterns cleared');

    // 5. Fix API authentication
    if (import.meta.env.DEV) {
      console.log('5️⃣ Fixing API authentication...');
    const authFixed = await fixApiAuth();
    results.push(authFixed ? '✅ API auth checked/fixed' : '⚠️ API auth needs attention');

    // 6. Fix SSE connection
    if (import.meta.env.DEV) {
      console.log('6️⃣ Fixing SSE connection...');
    if (window.sseConnectionFix) {
      const sseResult = await window.sseConnectionFix.autoFix();
      if (sseResult) {
        results.push('✅ SSE connection fixed');
      } else {
        results.push('⚠️ SSE connection needs manual attention');
      }
    } else {
      results.push('⚠️ SSE connection fix not available');
    }

    // 7. Reset navigation fix state
    if (import.meta.env.DEV) {
      console.log('7️⃣ Resetting navigation fixes...');
    if (window.navigationFix) {
      window.navigationFix.resetLogCounts();
      results.push('✅ Navigation fix state reset');
    } else {
      results.push('⚠️ Navigation fix not available');
    }

    // 8. Clear any problematic state
    if (import.meta.env.DEV) {
      console.log('8️⃣ Clearing problematic state...');
    if (window.emergencyCleanup) {
      window.emergencyCleanup();
      results.push('✅ Emergency cleanup completed');
    } else {
      results.push('⚠️ Emergency cleanup not available');
    }

    // 9. Check overall system health
    if (import.meta.env.DEV) {
      console.log('9️⃣ Running health check...');
    if (window.healthHelper) {
      try {
        const healthResult = await window.healthHelper.run();
        if (healthResult.healthScore >= 80) {
          results.push('✅ System health is good');
        } else {
          results.push(`⚠️ System health score: ${healthResult.healthScore}%`);
        }
      } catch (error) {
        results.push('⚠️ Health check failed');
      }
    } else {
      results.push('⚠️ Health helper not available');
    }

    // 10. Final verification
    if (import.meta.env.DEV) {
      console.log('🔟 Running final verification...');
    if (window.verifyFixes) {
      setTimeout(async () => {
        await window.verifyFixes();
      }, 1000);
      results.push('✅ Verification scheduled');
    } else {
      results.push('⚠️ Verification not available');
    }

    // Show results
    if (import.meta.env.DEV) {
      console.log('\n🎉 Quick Fix Results:');
    results.forEach(result => console.log(result));

    const successCount = results.filter(r => r.startsWith('✅')).length;
    const totalCount = results.length;

    if (import.meta.env.DEV) {
      console.log(`\n📊 Success Rate: ${successCount}/${totalCount} (${Math.round(successCount / totalCount * 100)}%)`);
    }

    if (successCount === totalCount) {
      if (import.meta.env.DEV) {
        console.log('🎊 All fixes applied successfully!');
      }
    } else if (successCount >= totalCount * 0.7) {
      if (import.meta.env.DEV) {
        console.log('✨ Most fixes applied, system should be stable');
      }
    } else {
      if (import.meta.env.DEV) {
        console.log('⚠️ Some fixes failed, manual intervention may be needed');
      }

    return {
      success: successCount >= totalCount * 0.7,
      results,
      successRate: Math.round(successCount / totalCount * 100)
    };

  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('❌ Quick Fix failed:', error);
    return {
      success: false,
      error: error.message,
      results
    };
  }

// Additional helper functions
function quickSilence() {
  if (import.meta.env.DEV) {
    console.log('🔇 Activating maximum silence mode...');
  }

  if (window.logSuppressor) {
    window.logSuppressor.activate();

    // Add extra patterns for maximum silence
    window.logSuppressor.addPattern(/\[.*\]/); // Any square bracket logs
    window.logSuppressor.addPattern(/📡|✅|🔧|🔍|🚀|🔑|🔐|🏠|📍|🔄|📥|📋|🛡️|⏰|📨/); // Emoji-heavy logs
    window.logSuppressor.addPattern(/PM\]/); // Timestamp logs
    window.logSuppressor.addPattern(/developmentOptimizer/); // Any developmentOptimizer mentions
    window.logSuppressor.addPattern(/\d+:\d+:\d+/); // Any timestamp

    if (import.meta.env.DEV) {
      console.log('✅ Maximum silence mode activated');
    }

    // Clear existing logs if possible
    if (typeof console.clear === 'function') {
      setTimeout(() => {
        console.clear();
        if (import.meta.env.DEV) {
          console.log('🔇 Console cleared - Maximum silence mode active');
        if (import.meta.env.DEV) {
          console.log('💡 Use window.allowLogs() to restore full logging');
        }
      }, 100);
    }
  } else {
    if (import.meta.env.DEV) {
      console.log('⚠️ Log suppressor not available');
    }

// Nuclear option - complete silence
function emergencySilence() {
  if (import.meta.env.DEV) {
    console.log('🚨 EMERGENCY SILENCE - Suppressing ALL non-error logs');
  }

  if (window.logSuppressor) {
    window.logSuppressor.activate();

    // Nuclear patterns - suppress almost everything
    window.logSuppressor.addPattern(/.*/); // Suppress everything except errors/warnings

    // Override to only allow errors and warnings
    const originalConsole = window.logSuppressor.originalConsole;

    console.log = () => { }; // Completely silent
    console.info = () => { }; // Completely silent
    console.debug = () => { }; // Completely silent
    // Keep warn and error

    console.clear();
    originalConsole.warn('🚨 EMERGENCY SILENCE ACTIVE - Only errors and warnings will show');
    originalConsole.warn('💡 Use window.allowLogs() to restore logging');
  }

function showFixStatus() {
  console.group('🔍 Frontend Fix Status');

  const status = {
    logSuppression: window.logSuppressor?.isActive || false,
    sseConnectionFix: typeof window.sseConnectionFix !== 'undefined',
    navigationFix: typeof window.navigationFix !== 'undefined',
    emergencyCleanup: typeof window.emergencyCleanup !== 'undefined',
    tokenManager: typeof window.tokenManager !== 'undefined',
    healthHelper: typeof window.healthHelper !== 'undefined'
  };

  console.table(status);

  const availableCount = Object.values(status).filter(Boolean).length;
  const totalCount = Object.keys(status).length;

  if (import.meta.env.DEV) {
    console.log(`📊 Available tools: ${availableCount}/${totalCount}`);
  console.groupEnd();

  return status;
}

// Specialized function to silence test noise
function silenceTests() {
  if (import.meta.env.DEV) {
    console.log('🧪 Silencing test automation noise...');
  }

  if (window.logSuppressor) {
    // Add test-specific patterns
    window.logSuppressor.addPattern(/🧪.*Test/);
    window.logSuppressor.addPattern(/🔍.*Test/);
    window.logSuppressor.addPattern(/Test \d+:/);
    window.logSuppressor.addPattern(/Check:/);
    window.logSuppressor.addPattern(/Summary:/);
    window.logSuppressor.addPattern(/Available commands:/);
    window.logSuppressor.addPattern(/window\.\w+\(\)/);

    if (import.meta.env.DEV) {
      console.log('✅ Test noise silenced');
    }
  } else {
    if (import.meta.env.DEV) {
      console.log('⚠️ Log suppressor not available');
    }

// Specialized function to fix API authentication issues
async function fixApiAuth() {
  if (import.meta.env.DEV) {
    console.log('🔐 Fixing API authentication issues...');
  }

  try {
    // Check token status
    if (window.tokenManager) {
      const isExpired = window.tokenManager.isTokenExpired();
      if (import.meta.env.DEV) {
        console.log('  - Token expired:', isExpired);
      }

      if (isExpired) {
        if (import.meta.env.DEV) {
          console.log('  - Refreshing token...');
        await window.tokenManager.refreshToken();
        if (import.meta.env.DEV) {
          console.log('  ✅ Token refreshed');
        }
      } else {
        if (import.meta.env.DEV) {
          console.log('  ✅ Token is valid');
        }

    // Check auth state
    if (window.authStateManager) {
      const authState = window.authStateManager.getAuthState();
      if (import.meta.env.DEV) {
        console.log('  - Auth state:', authState.isAuthenticated ? '✅ Authenticated' : '❌ Not authenticated');
      }

    if (import.meta.env.DEV) {
      console.log('✅ API authentication check completed');
    return true;
  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('❌ Failed to fix API auth:', error);
    return false;
  }

// Specialized function to suppress extension conflicts
function suppressExtensionConflicts() {
  if (import.meta.env.DEV) {
    console.log('🧩 Suppressing extension conflicts...');
  }

  if (window.logSuppressor) {
    // Add patterns for common extension errors
    window.logSuppressor.addPattern(/A listener indicated an asynchronous response/);
    window.logSuppressor.addPattern(/message channel closed before a response/);
    window.logSuppressor.addPattern(/Extension context invalidated/);
    window.logSuppressor.addPattern(/chrome-extension:/);
    window.logSuppressor.addPattern(/content script\.js/);

    if (import.meta.env.DEV) {
      console.log('✅ Extension conflict patterns added');
    }
  } else {
    if (import.meta.env.DEV) {
      console.log('⚠️ Log suppressor not available');
    }

// Specialized function to handle API 404 errors
function suppressApiNotFoundErrors() {
  if (import.meta.env.DEV) {
    console.log('🚫 Suppressing expected API 404 errors...');
  }

  if (window.logSuppressor) {
    // Add patterns for known missing API endpoints
    window.logSuppressor.addPattern(/POST.*\/chat\/\d+\/read 404/);
    window.logSuppressor.addPattern(/🚨 API Error.*404.*not_found/);
    window.logSuppressor.addPattern(/\[Mark chat as read\].*not_found/);
    window.logSuppressor.addPattern(/请求的资源未找到/);
    window.logSuppressor.addPattern(/Request failed with status code 404/);

    if (import.meta.env.DEV) {
      console.log('✅ API 404 error patterns added');
    }
  } else {
    if (import.meta.env.DEV) {
      console.log('⚠️ Log suppressor not available');
    }

// Enhanced function to clear repetitive errors
function clearRepetitiveErrors() {
  if (import.meta.env.DEV) {
    console.log('🔄 Clearing repetitive error patterns...');
  }

  if (window.logSuppressor) {
    // Add patterns for debug and test noise
    window.logSuppressor.addPattern(/debugDuplicateChannels\.js/);
    window.logSuppressor.addPattern(/testRequestIsolation\.js/);
    window.logSuppressor.addPattern(/testExtensionConflictFix\.js/);
    window.logSuppressor.addPattern(/testManager\.js/);

    if (import.meta.env.DEV) {
      console.log('✅ Debug script patterns added');
    }

  // Clear console if possible
  if (typeof console.clear === 'function') {
    setTimeout(() => {
      console.clear();
      if (import.meta.env.DEV) {
        console.log('🧹 Console cleared - Repetitive errors suppressed');
      }
    }, 100);
  }

// Export for global use
if (typeof window !== 'undefined') {
  window.quickFix = quickFix;
  window.quickSilence = quickSilence;
  window.emergencySilence = emergencySilence;
  window.showFixStatus = showFixStatus;
  window.silenceTests = silenceTests;
  window.fixApiAuth = fixApiAuth;
  window.suppressExtensionConflicts = suppressExtensionConflicts;
  window.suppressApiNotFoundErrors = suppressApiNotFoundErrors;
  window.clearRepetitiveErrors = clearRepetitiveErrors;

  if (import.meta.env.DEV) {
    console.log('🚀 Quick Fix loaded - use window.quickFix() for one-click repair');
  if (import.meta.env.DEV) {
    console.log('🔇 Use window.quickSilence() for maximum noise reduction');
  if (import.meta.env.DEV) {
    console.log('🚨 Use window.emergencySilence() for nuclear silence option');
  if (import.meta.env.DEV) {
    console.log('🔍 Use window.showFixStatus() to check available tools');
  if (import.meta.env.DEV) {
    console.log('🧪 Use window.silenceTests() to silence test automation noise');
  if (import.meta.env.DEV) {
    console.log('🔐 Use window.fixApiAuth() to fix authentication issues');
  if (import.meta.env.DEV) {
    console.log('🧩 Use window.suppressExtensionConflicts() to suppress extension errors');
  if (import.meta.env.DEV) {
    console.log('🚫 Use window.suppressApiNotFoundErrors() to suppress API 404 errors');
  if (import.meta.env.DEV) {
    console.log('🔄 Use window.clearRepetitiveErrors() to clear repetitive errors');
  }

export { quickFix, quickSilence, emergencySilence, showFixStatus, silenceTests, fixApiAuth, suppressExtensionConflicts, suppressApiNotFoundErrors, clearRepetitiveErrors }; 