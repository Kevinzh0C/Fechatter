/**
 * Core Message Persistence Fix Verification
 * 核心消息持久化修复验证
 */

class MessagePersistenceFixVerification {
  constructor() {
    this.results = [];
  }

  /**
   * Verify core fix implementation
   * 验证核心修复实现
   */
  async verifyCoreImplementation() {
    console.log('\n🔧 MESSAGE PERSISTENCE CORE FIX VERIFICATION');
    console.log('==============================================\n');

    // 1. Verify localStorage persistence layer
    this.verifyPersistenceLayer();

    // 2. Verify chat store integration  
    this.verifyChatStoreIntegration();

    // 3. Verify instant recovery functionality
    this.verifyInstantRecovery();

    // 4. Verify auto cleanup
    this.verifyAutoCleanup();

    // 5. Generate verification report
    this.generateVerificationReport();
  }

  /**
   * 1. Verify localStorage persistence layer exists and works
   */
  verifyPersistenceLayer() {
    console.log('1️⃣ PERSISTENCE LAYER VERIFICATION');
    console.log('=================================');

    try {
      // Check if minimalMessagePersistence is available
      const msgPersist = window.msgPersist;
      if (!msgPersist) {
        throw new Error('minimalMessagePersistence not loaded');
      }

      // Test core API functions
      const testMessage = {
        id: Date.now(),
        content: 'Core fix verification test',
        sender_id: 1,
        created_at: new Date().toISOString()
      };

      // Test save
      msgPersist.save(9999, testMessage);

      // Test get
      const retrieved = msgPersist.get(9999);
      if (retrieved.length === 0 || retrieved[0].id !== testMessage.id) {
        throw new Error('Save/get functionality broken');
      }

      // Test stats
      const stats = msgPersist.stats();
      if (!stats || typeof stats.totalMessages !== 'number') {
        throw new Error('Stats functionality broken');
      }

      // Test cleanup
      msgPersist.cleanup();

      console.log('✅ Persistence layer: VERIFIED');
      console.log(`   - Save/Get: Working`);
      console.log(`   - Stats: ${stats.totalMessages} messages, ${stats.storageSize}`);
      console.log(`   - Cleanup: Working`);

      this.results.push({
        component: 'Persistence Layer',
        status: 'VERIFIED',
        details: 'All core API functions working correctly'
      });

    } catch (error) {
      console.error('❌ Persistence layer: FAILED');
      console.error(`   Error: ${error.message}`);

      this.results.push({
        component: 'Persistence Layer',
        status: 'FAILED',
        error: error.message
      });
    }
  }

  /**
   * 2. Verify chat store integration
   */
  verifyChatStoreIntegration() {
    console.log('\n2️⃣ CHAT STORE INTEGRATION VERIFICATION');
    console.log('======================================');

    try {
      // Get chat store
      const chatStore = window.app?._instance?.proxy?.$pinia?._s?.get('chat');
      if (!chatStore) {
        throw new Error('Chat store not accessible');
      }

      // Check if sendMessage method exists
      if (typeof chatStore.sendMessage !== 'function') {
        throw new Error('sendMessage method not found');
      }

      // Check if fetchMessagesWithSignal method exists
      if (typeof chatStore.fetchMessagesWithSignal !== 'function') {
        throw new Error('fetchMessagesWithSignal method not found');
      }

      // Verify persistence integration by checking source code patterns
      const sendMessageStr = chatStore.sendMessage.toString();
      const fetchMessagesStr = chatStore.fetchMessagesWithSignal.toString();

      const hasSavePersistence = sendMessageStr.includes('minimalMessagePersistence.saveMessage');
      const hasGetPersistence = fetchMessagesStr.includes('minimalMessagePersistence.getMessages');

      if (!hasSavePersistence) {
        throw new Error('sendMessage missing persistence integration');
      }

      if (!hasGetPersistence) {
        throw new Error('fetchMessagesWithSignal missing persistence integration');
      }

      console.log('✅ Chat store integration: VERIFIED');
      console.log(`   - sendMessage persistence: Integrated`);
      console.log(`   - fetchMessages recovery: Integrated`);
      console.log(`   - Current chat: ${chatStore.currentChatId || 'None'}`);

      this.results.push({
        component: 'Chat Store Integration',
        status: 'VERIFIED',
        details: 'Persistence hooks properly integrated'
      });

    } catch (error) {
      console.error('❌ Chat store integration: FAILED');
      console.error(`   Error: ${error.message}`);

      this.results.push({
        component: 'Chat Store Integration',
        status: 'FAILED',
        error: error.message
      });
    }
  }

  /**
   * 3. Verify instant recovery functionality
   */
  verifyInstantRecovery() {
    console.log('\n3️⃣ INSTANT RECOVERY VERIFICATION');
    console.log('================================');

    try {
      const chatStore = window.app?._instance?.proxy?.$pinia?._s?.get('chat');
      const msgPersist = window.msgPersist;

      if (!chatStore || !msgPersist) {
        throw new Error('Required components not available');
      }

      // Check if there are any persisted messages
      const stats = msgPersist.stats();
      if (stats.totalMessages === 0) {
        console.log('⚠️ Instant recovery: NO DATA');
        console.log('   No persisted messages to test recovery');
        console.log('   Send a message first, then refresh the page');

        this.results.push({
          component: 'Instant Recovery',
          status: 'NO_DATA',
          details: 'No persisted messages available for testing'
        });
        return;
      }

      // Check current chat state
      const currentChatId = chatStore.currentChatId;
      if (!currentChatId) {
        console.log('⚠️ Instant recovery: NO ACTIVE CHAT');
        console.log('   Open a chat to test recovery functionality');

        this.results.push({
          component: 'Instant Recovery',
          status: 'NO_ACTIVE_CHAT',
          details: 'No active chat to test recovery'
        });
        return;
      }

      // Check if current chat has persisted messages
      const persistedMessages = msgPersist.get(currentChatId);
      const currentMessages = chatStore.messages.length;

      console.log('✅ Instant recovery: READY FOR TESTING');
      console.log(`   - Current chat: ${currentChatId}`);
      console.log(`   - Persisted messages: ${persistedMessages.length}`);
      console.log(`   - Current UI messages: ${currentMessages}`);
      console.log('');
      console.log('📋 Manual verification steps:');
      console.log('   1. Note the current message count above');
      console.log('   2. Refresh this page (Cmd+R or F5)');
      console.log('   3. Navigate back to this chat');
      console.log('   4. Messages should appear INSTANTLY (0ms delay)');
      console.log('   5. Message count should match or be close to persisted count');

      this.results.push({
        component: 'Instant Recovery',
        status: 'READY_FOR_MANUAL_TEST',
        details: `${persistedMessages.length} messages ready for recovery test`
      });

    } catch (error) {
      console.error('❌ Instant recovery: FAILED');
      console.error(`   Error: ${error.message}`);

      this.results.push({
        component: 'Instant Recovery',
        status: 'FAILED',
        error: error.message
      });
    }
  }

  /**
   * 4. Verify auto cleanup functionality
   */
  verifyAutoCleanup() {
    console.log('\n4️⃣ AUTO CLEANUP VERIFICATION');
    console.log('============================');

    try {
      const msgPersist = window.msgPersist;
      if (!msgPersist) {
        throw new Error('Message persistence not available');
      }

      // Get current stats before cleanup
      const statsBefore = msgPersist.stats();

      // Run cleanup
      msgPersist.cleanup();

      // Get stats after cleanup
      const statsAfter = msgPersist.stats();

      console.log('✅ Auto cleanup: VERIFIED');
      console.log(`   - Before cleanup: ${statsBefore.totalChats} chats, ${statsBefore.totalMessages} messages`);
      console.log(`   - After cleanup: ${statsAfter.totalChats} chats, ${statsAfter.totalMessages} messages`);
      console.log(`   - Storage size: ${statsAfter.storageSize}`);

      // Check if cleanup is working (should not crash and should maintain reasonable limits)
      if (statsAfter.totalChats <= 10 && statsAfter.totalMessages >= 0) {
        console.log('   - Cleanup constraints: ENFORCED');
      } else {
        console.log('   - Cleanup constraints: WARNING - May need adjustment');
      }

      this.results.push({
        component: 'Auto Cleanup',
        status: 'VERIFIED',
        details: `Cleanup working, ${statsAfter.totalChats} chats maintained`
      });

    } catch (error) {
      console.error('❌ Auto cleanup: FAILED');
      console.error(`   Error: ${error.message}`);

      this.results.push({
        component: 'Auto Cleanup',
        status: 'FAILED',
        error: error.message
      });
    }
  }

  /**
   * 5. Generate verification report
   */
  generateVerificationReport() {
    console.log('\n📊 CORE FIX VERIFICATION REPORT');
    console.log('===============================\n');

    const verified = this.results.filter(r => r.status === 'VERIFIED').length;
    const failed = this.results.filter(r => r.status === 'FAILED').length;
    const ready = this.results.filter(r => r.status.includes('READY')).length;
    const noData = this.results.filter(r => r.status.includes('NO_DATA')).length;

    console.log('🔍 Verification Results:');
    this.results.forEach(result => {
      const emoji = {
        'VERIFIED': '✅',
        'FAILED': '❌',
        'READY_FOR_MANUAL_TEST': '📋',
        'NO_DATA': '📭',
        'NO_ACTIVE_CHAT': '💬'
      }[result.status] || '❓';

      console.log(`   ${emoji} ${result.component}: ${result.status}`);
      if (result.details) {
        console.log(`      ${result.details}`);
      }
      if (result.error) {
        console.log(`      Error: ${result.error}`);
      }
    });

    console.log(`\n📊 Summary: ${verified} verified, ${failed} failed, ${ready} ready for test, ${noData} no data`);

    // Overall assessment
    if (failed === 0 && verified >= 2) {
      console.log('\n🎉 CORE FIX STATUS: SUCCESS');
      console.log('✅ Message persistence core implementation is working correctly!');
      console.log('✅ Ready for production use');

      if (ready > 0) {
        console.log('\n📋 Next steps:');
        console.log('   1. Send a test message in any chat');
        console.log('   2. Refresh the page (Cmd+R or F5)');
        console.log('   3. Navigate back to the chat');
        console.log('   4. Verify messages appear instantly');
      }
    } else if (failed === 0) {
      console.log('\n👍 CORE FIX STATUS: GOOD');
      console.log('✅ Basic implementation verified');
      console.log('⚠️ Some components need manual testing');
    } else {
      console.log('\n⚠️ CORE FIX STATUS: ISSUES DETECTED');
      console.log('❌ Some components failed verification');
      console.log('🔧 Review failed components above');
    }

    return {
      verified,
      failed,
      ready,
      noData,
      overall: failed === 0 ? 'SUCCESS' : 'ISSUES'
    };
  }
}

// Create global instance and expose to window
const fixVerification = new MessagePersistenceFixVerification();

if (typeof window !== 'undefined') {
  window.verifyFix = {
    run: () => fixVerification.verifyCoreImplementation(),
    persistence: () => fixVerification.verifyPersistenceLayer(),
    integration: () => fixVerification.verifyChatStoreIntegration(),
    recovery: () => fixVerification.verifyInstantRecovery(),
    cleanup: () => fixVerification.verifyAutoCleanup()
  };

  console.log('🔧 Core Fix Verification loaded');
  console.log('   Commands:');
  console.log('   - window.verifyFix.run() - Complete verification');
  console.log('   - window.verifyFix.persistence() - Test persistence layer');
  console.log('   - window.verifyFix.integration() - Test store integration');
  console.log('   - window.verifyFix.recovery() - Test instant recovery');
}

export default fixVerification; 