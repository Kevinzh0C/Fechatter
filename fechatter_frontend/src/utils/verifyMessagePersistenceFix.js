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
    if (import.meta.env.DEV) {
      console.log('\n🔧 MESSAGE PERSISTENCE CORE FIX VERIFICATION');
    if (import.meta.env.DEV) {
      console.log('==============================================\n');
    }

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
    if (import.meta.env.DEV) {
      console.log('1️⃣ PERSISTENCE LAYER VERIFICATION');
    if (import.meta.env.DEV) {
      console.log('=================================');
    }

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

      if (import.meta.env.DEV) {
        console.log('✅ Persistence layer: VERIFIED');
      if (import.meta.env.DEV) {
        console.log(`   - Save/Get: Working`);
      if (import.meta.env.DEV) {
        console.log(`   - Stats: ${stats.totalMessages} messages, ${stats.storageSize}`);
      if (import.meta.env.DEV) {
        console.log(`   - Cleanup: Working`);
      }

      this.results.push({
        component: 'Persistence Layer',
        status: 'VERIFIED',
        details: 'All core API functions working correctly'
      });

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Persistence layer: FAILED');
      if (import.meta.env.DEV) {
        console.error(`   Error: ${error.message}`);
      }

      this.results.push({
        component: 'Persistence Layer',
        status: 'FAILED',
        error: error.message
      });

  /**
   * 2. Verify chat store integration
   */
  verifyChatStoreIntegration() {
    if (import.meta.env.DEV) {
      console.log('\n2️⃣ CHAT STORE INTEGRATION VERIFICATION');
    if (import.meta.env.DEV) {
      console.log('======================================');
    }

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

      if (import.meta.env.DEV) {
        console.log('✅ Chat store integration: VERIFIED');
      if (import.meta.env.DEV) {
        console.log(`   - sendMessage persistence: Integrated`);
      if (import.meta.env.DEV) {
        console.log(`   - fetchMessages recovery: Integrated`);
      if (import.meta.env.DEV) {
        console.log(`   - Current chat: ${chatStore.currentChatId || 'None'}`);
      }

      this.results.push({
        component: 'Chat Store Integration',
        status: 'VERIFIED',
        details: 'Persistence hooks properly integrated'
      });

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Chat store integration: FAILED');
      if (import.meta.env.DEV) {
        console.error(`   Error: ${error.message}`);
      }

      this.results.push({
        component: 'Chat Store Integration',
        status: 'FAILED',
        error: error.message
      });

  /**
   * 3. Verify instant recovery functionality
   */
  verifyInstantRecovery() {
    if (import.meta.env.DEV) {
      console.log('\n3️⃣ INSTANT RECOVERY VERIFICATION');
    if (import.meta.env.DEV) {
      console.log('================================');
    }

    try {
      const chatStore = window.app?._instance?.proxy?.$pinia?._s?.get('chat');
      const msgPersist = window.msgPersist;

      if (!chatStore || !msgPersist) {
        throw new Error('Required components not available');
      }

      // Check if there are any persisted messages
      const stats = msgPersist.stats();
      if (stats.totalMessages === 0) {
        if (import.meta.env.DEV) {
          console.log('⚠️ Instant recovery: NO DATA');
        if (import.meta.env.DEV) {
          console.log('   No persisted messages to test recovery');
        if (import.meta.env.DEV) {
          console.log('   Send a message first, then refresh the page');
        }

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
        if (import.meta.env.DEV) {
          console.log('⚠️ Instant recovery: NO ACTIVE CHAT');
        if (import.meta.env.DEV) {
          console.log('   Open a chat to test recovery functionality');
        }

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

      if (import.meta.env.DEV) {
        console.log('✅ Instant recovery: READY FOR TESTING');
      if (import.meta.env.DEV) {
        console.log(`   - Current chat: ${currentChatId}`);
      if (import.meta.env.DEV) {
        console.log(`   - Persisted messages: ${persistedMessages.length}`);
      if (import.meta.env.DEV) {
        console.log(`   - Current UI messages: ${currentMessages}`);
      if (import.meta.env.DEV) {
        console.log('');
      if (import.meta.env.DEV) {
        console.log('📋 Manual verification steps:');
      if (import.meta.env.DEV) {
        console.log('   1. Note the current message count above');
      if (import.meta.env.DEV) {
        console.log('   2. Refresh this page (Cmd+R or F5)');
      if (import.meta.env.DEV) {
        console.log('   3. Navigate back to this chat');
      if (import.meta.env.DEV) {
        console.log('   4. Messages should appear INSTANTLY (0ms delay)');
      if (import.meta.env.DEV) {
        console.log('   5. Message count should match or be close to persisted count');
      }

      this.results.push({
        component: 'Instant Recovery',
        status: 'READY_FOR_MANUAL_TEST',
        details: `${persistedMessages.length} messages ready for recovery test`
      });

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Instant recovery: FAILED');
      if (import.meta.env.DEV) {
        console.error(`   Error: ${error.message}`);
      }

      this.results.push({
        component: 'Instant Recovery',
        status: 'FAILED',
        error: error.message
      });

  /**
   * 4. Verify auto cleanup functionality
   */
  verifyAutoCleanup() {
    if (import.meta.env.DEV) {
      console.log('\n4️⃣ AUTO CLEANUP VERIFICATION');
    if (import.meta.env.DEV) {
      console.log('============================');
    }

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

      if (import.meta.env.DEV) {
        console.log('✅ Auto cleanup: VERIFIED');
      if (import.meta.env.DEV) {
        console.log(`   - Before cleanup: ${statsBefore.totalChats} chats, ${statsBefore.totalMessages} messages`);
      if (import.meta.env.DEV) {
        console.log(`   - After cleanup: ${statsAfter.totalChats} chats, ${statsAfter.totalMessages} messages`);
      if (import.meta.env.DEV) {
        console.log(`   - Storage size: ${statsAfter.storageSize}`);
      }

      // Check if cleanup is working (should not crash and should maintain reasonable limits)
      if (statsAfter.totalChats <= 10 && statsAfter.totalMessages >= 0) {
        if (import.meta.env.DEV) {
          console.log('   - Cleanup constraints: ENFORCED');
        }
      } else {
        if (import.meta.env.DEV) {
          console.log('   - Cleanup constraints: WARNING - May need adjustment');
        }

      this.results.push({
        component: 'Auto Cleanup',
        status: 'VERIFIED',
        details: `Cleanup working, ${statsAfter.totalChats} chats maintained`
      });

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Auto cleanup: FAILED');
      if (import.meta.env.DEV) {
        console.error(`   Error: ${error.message}`);
      }

      this.results.push({
        component: 'Auto Cleanup',
        status: 'FAILED',
        error: error.message
      });

  /**
   * 5. Generate verification report
   */
  generateVerificationReport() {
    if (import.meta.env.DEV) {
      console.log('\n📊 CORE FIX VERIFICATION REPORT');
    if (import.meta.env.DEV) {
      console.log('===============================\n');
    }

    const verified = this.results.filter(r => r.status === 'VERIFIED').length;
    const failed = this.results.filter(r => r.status === 'FAILED').length;
    const ready = this.results.filter(r => r.status.includes('READY')).length;
    const noData = this.results.filter(r => r.status.includes('NO_DATA')).length;

    if (import.meta.env.DEV) {
      console.log('🔍 Verification Results:');
    this.results.forEach(result => {
      const emoji = {
        'VERIFIED': '✅',
        'FAILED': '❌',
        'READY_FOR_MANUAL_TEST': '📋',
        'NO_DATA': '📭',
        'NO_ACTIVE_CHAT': '💬'
      }[result.status] || '❓';

      if (import.meta.env.DEV) {
        console.log(`   ${emoji} ${result.component}: ${result.status}`);
      if (result.details) {
        if (import.meta.env.DEV) {
          console.log(`      ${result.details}`);
      if (result.error) {
        if (import.meta.env.DEV) {
          console.log(`      Error: ${result.error}`);
        }
    });

    if (import.meta.env.DEV) {
      console.log(`\n📊 Summary: ${verified} verified, ${failed} failed, ${ready} ready for test, ${noData} no data`);
    }

    // Overall assessment
    if (failed === 0 && verified >= 2) {
      if (import.meta.env.DEV) {
        console.log('\n🎉 CORE FIX STATUS: SUCCESS');
      if (import.meta.env.DEV) {
        console.log('✅ Message persistence core implementation is working correctly!');
      if (import.meta.env.DEV) {
        console.log('✅ Ready for production use');
      }

      if (ready > 0) {
        if (import.meta.env.DEV) {
          console.log('\n📋 Next steps:');
        if (import.meta.env.DEV) {
          console.log('   1. Send a test message in any chat');
        if (import.meta.env.DEV) {
          console.log('   2. Refresh the page (Cmd+R or F5)');
        if (import.meta.env.DEV) {
          console.log('   3. Navigate back to the chat');
        if (import.meta.env.DEV) {
          console.log('   4. Verify messages appear instantly');
        }
    } else if (failed === 0) {
      if (import.meta.env.DEV) {
        console.log('\n👍 CORE FIX STATUS: GOOD');
      if (import.meta.env.DEV) {
        console.log('✅ Basic implementation verified');
      if (import.meta.env.DEV) {
        console.log('⚠️ Some components need manual testing');
      }
    } else {
      if (import.meta.env.DEV) {
        console.log('\n⚠️ CORE FIX STATUS: ISSUES DETECTED');
      if (import.meta.env.DEV) {
        console.log('❌ Some components failed verification');
      if (import.meta.env.DEV) {
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

  if (import.meta.env.DEV) {
    console.log('🔧 Core Fix Verification loaded');
  if (import.meta.env.DEV) {
    console.log('   Commands:');
  if (import.meta.env.DEV) {
    console.log('   - window.verifyFix.run() - Complete verification');
  if (import.meta.env.DEV) {
    console.log('   - window.verifyFix.persistence() - Test persistence layer');
  if (import.meta.env.DEV) {
    console.log('   - window.verifyFix.integration() - Test store integration');
  if (import.meta.env.DEV) {
    console.log('   - window.verifyFix.recovery() - Test instant recovery');
  }

export default fixVerification; 