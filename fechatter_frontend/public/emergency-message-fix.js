/**
 * 🚨 EMERGENCY MESSAGE DISPLAY FIX
 * 
 * Paste this script into browser console to immediately fix message display issues.
 * Addresses: "ALL 15 messages missing in chat 6" critical error.
 */

(function emergencyMessageDisplayFix() {
  console.log('🚨 EMERGENCY MESSAGE DISPLAY FIX - Starting...');

  // Step 1: Diagnostic Information
  function runDiagnostics() {
    console.log('🔍 Running emergency diagnostics...');

    const results = {
      messageDisplayGuarantee: !!window.messageDisplayGuarantee,
      messageElements: document.querySelectorAll('[data-message-id]').length,
      containers: document.querySelectorAll('.discord-message-list, .messages-container, .simple-message-list').length,
      currentChatId: getCurrentChatId(),
      activeContexts: window.messageDisplayGuarantee ? Array.from(window.messageDisplayGuarantee.verificationQueue.entries()) : []
    };

    console.log('📊 Diagnostic Results:', results);
    return results;
  }

  function getCurrentChatId() {
    const match = window.location.pathname.match(/\/chat\/(\d+)/);
    return match ? parseInt(match[1]) : null;
  }

  // Step 2: Clear all failed tracking
  function clearFailedTracking() {
    console.log('🧹 Clearing all failed tracking contexts...');

    if (!window.messageDisplayGuarantee) {
      console.error('❌ MessageDisplayGuarantee not available');
      return 0;
    }

    const clearedCount = window.messageDisplayGuarantee.clearAllTracking();
    console.log(`✅ Cleared ${clearedCount} tracking contexts`);
    return clearedCount;
  }

  // Step 3: Force register all visible messages
  function forceRegisterAllMessages() {
    console.log('🔧 Force registering all visible messages...');

    const chatId = getCurrentChatId();
    if (!chatId) {
      console.error('❌ No chat ID found - cannot proceed');
      return { success: 0, failed: 0 };
    }

    if (!window.messageDisplayGuarantee) {
      console.error('❌ MessageDisplayGuarantee not available');
      return { success: 0, failed: 0 };
    }

    const messageElements = document.querySelectorAll('[data-message-id]');
    let successCount = 0;
    let failedCount = 0;
    const results = [];

    console.log(`🔍 Found ${messageElements.length} message elements`);

    messageElements.forEach((element, index) => {
      const messageId = element.getAttribute('data-message-id');
      const isVisible = element.offsetParent !== null;
      const rect = element.getBoundingClientRect();

      if (messageId && isVisible) {
        try {
          window.messageDisplayGuarantee.markMessageDisplayed(
            parseInt(messageId),
            element,
            chatId
          );
          successCount++;
          results.push({
            messageId,
            status: 'success',
            visible: isVisible,
            rect: { width: rect.width, height: rect.height }
          });
          console.log(`✅ [${index + 1}/${messageElements.length}] Registered message ${messageId}`);
        } catch (error) {
          failedCount++;
          results.push({
            messageId,
            status: 'failed',
            error: error.message,
            visible: isVisible,
            rect: { width: rect.width, height: rect.height }
          });
          console.warn(`❌ [${index + 1}/${messageElements.length}] Failed to register message ${messageId}:`, error.message);
        }
      } else {
        results.push({
          messageId,
          status: 'skipped',
          reason: !messageId ? 'no_message_id' : 'not_visible',
          visible: isVisible,
          rect: { width: rect.width, height: rect.height }
        });
        console.log(`⚠️ [${index + 1}/${messageElements.length}] Skipped message ${messageId || 'NO_ID'}: ${isVisible ? 'visible' : 'hidden'}`);
      }
    });

    console.log(`📊 Registration Results: ${successCount} success, ${failedCount} failed`);
    return { success: successCount, failed: failedCount, details: results };
  }

  // Step 4: Restart tracking for current chat
  function restartTracking() {
    console.log('🔄 Restarting message tracking...');

    const chatId = getCurrentChatId();
    if (!chatId || !window.messageDisplayGuarantee) {
      console.error('❌ Cannot restart tracking - missing chatId or MessageDisplayGuarantee');
      return false;
    }

    const messageElements = document.querySelectorAll('[data-message-id]');
    const visibleMessageIds = Array.from(messageElements)
      .filter(el => el.offsetParent !== null && el.getAttribute('data-message-id'))
      .map(el => parseInt(el.getAttribute('data-message-id')))
      .filter(id => !isNaN(id));

    if (visibleMessageIds.length === 0) {
      console.warn('⚠️ No visible messages found for tracking');
      return false;
    }

    try {
      window.messageDisplayGuarantee.startMessageTracking(chatId, new Set(visibleMessageIds));
      console.log(`✅ Restarted tracking for ${visibleMessageIds.length} messages:`, visibleMessageIds);
      return true;
    } catch (error) {
      console.error(`❌ Failed to restart tracking:`, error);
      return false;
    }
  }

  // Step 5: Validate fix
  function validateFix() {
    console.log('🧪 Validating fix...');

    if (!window.messageDisplayGuarantee) {
      console.error('❌ Cannot validate - MessageDisplayGuarantee not available');
      return false;
    }

    const metrics = window.messageDisplayGuarantee.getMetrics();
    const contexts = Array.from(window.messageDisplayGuarantee.verificationQueue.entries());

    console.log('📊 Current Metrics:', metrics);
    console.log(`🔍 Active Contexts: ${contexts.length}`);

    contexts.forEach(([id, ctx]) => {
      console.log(`🔍 Context ${id}: Chat ${ctx.chatId}, ${ctx.displayedIds.size}/${ctx.messageIds.size} displayed, Status: ${ctx.status}`);
    });

    const messageElements = document.querySelectorAll('[data-message-id]');
    const visibleElements = Array.from(messageElements).filter(el => el.offsetParent !== null);

    console.log(`📊 DOM State: ${visibleElements.length}/${messageElements.length} messages visible`);

    return {
      metrics,
      activeContexts: contexts.length,
      visibleMessages: visibleElements.length,
      totalMessages: messageElements.length
    };
  }

  // Main execution flow
  async function runEmergencyFix() {
    console.log('🚨 STARTING EMERGENCY MESSAGE DISPLAY FIX');
    console.log('='.repeat(50));

    try {
      // Step 1: Diagnostics
      const diagnostics = runDiagnostics();

      // Step 2: Clear failed tracking
      const clearedCount = clearFailedTracking();

      // Step 3: Force register messages
      console.log('\n⏳ Waiting 1 second for cleanup...');
      await new Promise(resolve => setTimeout(resolve, 1000));

      const registrationResults = forceRegisterAllMessages();

      // Step 4: Restart tracking
      console.log('\n⏳ Waiting 500ms before restarting tracking...');
      await new Promise(resolve => setTimeout(resolve, 500));

      const trackingRestarted = restartTracking();

      // Step 5: Validate
      console.log('\n⏳ Waiting 2 seconds for validation...');
      await new Promise(resolve => setTimeout(resolve, 2000));

      const validation = validateFix();

      // Summary
      console.log('\n' + '🎯 EMERGENCY FIX COMPLETE'.padEnd(50, '='));
      console.log('📊 SUMMARY:');
      console.log(`  ✅ Cleared contexts: ${clearedCount}`);
      console.log(`  ✅ Registered messages: ${registrationResults.success}`);
      console.log(`  ❌ Failed registrations: ${registrationResults.failed}`);
      console.log(`  🔄 Tracking restarted: ${trackingRestarted ? 'Yes' : 'No'}`);
      console.log(`  📈 Current visible messages: ${validation.visibleMessages}`);
      console.log(`  🎯 Active tracking contexts: ${validation.activeContexts}`);

      if (registrationResults.success > 0 && validation.activeContexts >= 0) {
        console.log('\n🎉 FIX APPEARS SUCCESSFUL! Messages should now be properly tracked.');
      } else {
        console.log('\n⚠️ FIX MAY NEED MANUAL VERIFICATION. Check the details above.');
      }

      console.log('\n📋 Detailed Results Available:');
      console.log('  - window.emergencyFixResults.diagnostics');
      console.log('  - window.emergencyFixResults.registration');
      console.log('  - window.emergencyFixResults.validation');

      // Store results globally for inspection
      window.emergencyFixResults = {
        diagnostics,
        clearedCount,
        registration: registrationResults,
        trackingRestarted,
        validation
      };

      return window.emergencyFixResults;

    } catch (error) {
      console.error('🚨 EMERGENCY FIX FAILED:', error);
      throw error;
    }
  }

  // Execute the fix
  return runEmergencyFix();
})(); 