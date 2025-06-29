/**
 * Test Message Persistence Functionality
 * 测试消息持久化功能的完整性
 */

class MessagePersistenceTest {
  constructor() {
    this.testResults = [];
    this.testMessage = `Test persistence message ${Date.now()}`;
  }

  /**
   * 运行完整的持久化测试套件
   */
  async runCompleteTest() {
    if (import.meta.env.DEV) {
      console.log('\n🧪 COMPREHENSIVE MESSAGE PERSISTENCE TEST');
    if (import.meta.env.DEV) {
      console.log('==========================================\n');
    }

    try {
      // 1. 基础功能测试
      await this.testBasicPersistence();

      // 2. 发送消息测试
      await this.testMessageSending();

      // 3. 刷新恢复测试
      this.testRefreshRecovery();

      // 4. 多聊天测试
      await this.testMultipleChats();

      // 5. 错误处理测试
      this.testErrorHandling();

      // 6. 性能测试
      this.testPerformance();

      // 7. 生成测试报告
      this.generateTestReport();

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Test suite failed:', error);
      this.testResults.push({
        test: 'Test Suite',
        status: 'FAILED',
        error: error.message
      });

  /**
   * 1. 基础持久化功能测试
   */
  async testBasicPersistence() {
    if (import.meta.env.DEV) {
      console.log('1️⃣ BASIC PERSISTENCE TEST');
    if (import.meta.env.DEV) {
      console.log('==========================');
    }

    try {
      const msgPersist = window.msgPersist;
      if (!msgPersist) {
        throw new Error('window.msgPersist not available');
      }

      // 测试保存和获取
      const testMessage = {
        id: 12345,
        content: 'Test message',
        sender_id: 1,
        created_at: new Date().toISOString()
      };

      if (import.meta.env.DEV) {
        console.log('📝 Testing save/get functionality...');
      msgPersist.save(999, testMessage);
      const retrieved = msgPersist.get(999);

      if (retrieved.length === 1 && retrieved[0].id === 12345) {
        if (import.meta.env.DEV) {
          console.log('✅ Basic save/get: PASSED');
        this.testResults.push({
          test: 'Basic Persistence',
          status: 'PASSED',
          details: 'Save and retrieve working correctly'
        });
      } else {
        throw new Error('Save/get functionality failed');
      }

      // 测试统计功能
      const stats = msgPersist.stats();
      if (import.meta.env.DEV) {
        console.log('📊 Storage stats:', stats);
      }

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Basic persistence test failed:', error);
      this.testResults.push({
        test: 'Basic Persistence',
        status: 'FAILED',
        error: error.message
      });

  /**
   * 2. 消息发送测试
   */
  async testMessageSending() {
    if (import.meta.env.DEV) {
      console.log('\n2️⃣ MESSAGE SENDING TEST');
    if (import.meta.env.DEV) {
      console.log('=======================');
    }

    try {
      const chatStore = window.app._instance.proxy.$pinia._s.get('chat');
      const currentChatId = chatStore.currentChatId;

      if (!currentChatId) {
        if (import.meta.env.DEV) {
          console.log('⚠️ No chat selected. Please open a chat first.');
        this.testResults.push({
          test: 'Message Sending',
          status: 'SKIPPED',
          details: 'No chat selected'
        });
        return;
      }

      if (import.meta.env.DEV) {
        console.log(`📤 Sending test message to chat ${currentChatId}...`);
      }

      // 记录发送前的状态
      const beforeMessages = chatStore.messages.length;
      const beforePersisted = window.msgPersist.get(currentChatId).length;

      // 发送测试消息
      await chatStore.sendMessage(currentChatId, {
        content: this.testMessage
      });

      // 等待一下确保处理完成
      await new Promise(resolve => setTimeout(resolve, 1000));

      // 验证结果
      const afterMessages = chatStore.messages.length;
      const afterPersisted = window.msgPersist.get(currentChatId).length;

      const uiUpdated = afterMessages > beforeMessages;
      const persistenceUpdated = afterPersisted > beforePersisted;

      if (uiUpdated && persistenceUpdated) {
        if (import.meta.env.DEV) {
          console.log('✅ Message sending: PASSED');
        if (import.meta.env.DEV) {
          console.log(`   UI: ${beforeMessages} → ${afterMessages} messages`);
        if (import.meta.env.DEV) {
          console.log(`   Storage: ${beforePersisted} → ${afterPersisted} messages`);
        }

        this.testResults.push({
          test: 'Message Sending',
          status: 'PASSED',
          details: `UI and persistence both updated correctly`
        });
      } else {
        throw new Error(`UI updated: ${uiUpdated}, Persistence updated: ${persistenceUpdated}`);
      }

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Message sending test failed:', error);
      this.testResults.push({
        test: 'Message Sending',
        status: 'FAILED',
        error: error.message
      });

  /**
   * 3. 刷新恢复测试
   */
  testRefreshRecovery() {
    if (import.meta.env.DEV) {
      console.log('\n3️⃣ REFRESH RECOVERY TEST');
    if (import.meta.env.DEV) {
      console.log('========================');
    }

    const chatStore = window.app._instance.proxy.$pinia._s.get('chat');
    const currentChatId = chatStore.currentChatId;

    if (!currentChatId) {
      if (import.meta.env.DEV) {
        console.log('⚠️ No chat selected.');
      this.testResults.push({
        test: 'Refresh Recovery',
        status: 'SKIPPED',
        details: 'No chat selected'
      });
      return;
    }

    // 检查当前消息状态
    const currentMessages = chatStore.messages.length;
    const persistedMessages = window.msgPersist.get(currentChatId).length;

    if (import.meta.env.DEV) {
      console.log(`📊 Current state:`);
    if (import.meta.env.DEV) {
      console.log(`   UI messages: ${currentMessages}`);
    if (import.meta.env.DEV) {
      console.log(`   Persisted messages: ${persistedMessages}`);
    }

    if (persistedMessages > 0) {
      if (import.meta.env.DEV) {
        console.log('✅ Refresh recovery: READY');
      if (import.meta.env.DEV) {
        console.log('📋 Manual verification steps:');
      if (import.meta.env.DEV) {
        console.log('   1. Refresh this page (Cmd+R or F5)');
      if (import.meta.env.DEV) {
        console.log('   2. Navigate back to this chat');
      if (import.meta.env.DEV) {
        console.log(`   3. Look for your test message: "${this.testMessage}"`);
      if (import.meta.env.DEV) {
        console.log('   4. If message appears instantly → SUCCESS');
      if (import.meta.env.DEV) {
        console.log('   5. If message missing → FAILURE');
      }

      this.testResults.push({
        test: 'Refresh Recovery',
        status: 'MANUAL_VERIFY',
        details: `${persistedMessages} messages ready for recovery test`
      });
    } else {
      if (import.meta.env.DEV) {
        console.log('⚠️ No persisted messages to test recovery');
      this.testResults.push({
        test: 'Refresh Recovery',
        status: 'NO_DATA',
        details: 'No persisted messages available'
      });

  /**
   * 4. 多聊天测试
   */
  async testMultipleChats() {
    if (import.meta.env.DEV) {
      console.log('\n4️⃣ MULTIPLE CHATS TEST');
    if (import.meta.env.DEV) {
      console.log('======================');
    }

    try {
      const stats = window.msgPersist.stats();
      const chatCount = stats.totalChats;
      const messageCount = stats.totalMessages;

      if (import.meta.env.DEV) {
        console.log(`📊 Multi-chat statistics:`);
      if (import.meta.env.DEV) {
        console.log(`   Total chats with messages: ${chatCount}`);
      if (import.meta.env.DEV) {
        console.log(`   Total persisted messages: ${messageCount}`);
      if (import.meta.env.DEV) {
        console.log(`   Storage size: ${stats.storageSize}`);
      }

      // 显示每个聊天的详情
      stats.chats.forEach(chat => {
        if (import.meta.env.DEV) {
          console.log(`   Chat ${chat.chatId}: ${chat.messageCount} messages`);
        }
      });

      if (chatCount > 0) {
        if (import.meta.env.DEV) {
          console.log('✅ Multiple chats: PASSED');
        this.testResults.push({
          test: 'Multiple Chats',
          status: 'PASSED',
          details: `${chatCount} chats, ${messageCount} total messages`
        });
      } else {
        if (import.meta.env.DEV) {
          console.log('⚠️ No chats found in storage');
        this.testResults.push({
          test: 'Multiple Chats',
          status: 'NO_DATA',
          details: 'No chats in storage'
        });

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Multiple chats test failed:', error);
      this.testResults.push({
        test: 'Multiple Chats',
        status: 'FAILED',
        error: error.message
      });

  /**
   * 5. 错误处理测试
   */
  testErrorHandling() {
    if (import.meta.env.DEV) {
      console.log('\n5️⃣ ERROR HANDLING TEST');
    if (import.meta.env.DEV) {
      console.log('======================');
    }

    try {
      const msgPersist = window.msgPersist;

      // 测试无效输入
      if (import.meta.env.DEV) {
        console.log('🧪 Testing invalid inputs...');
      }

      // 这些不应该崩溃
      msgPersist.save(null, null);
      msgPersist.save(undefined, {});
      msgPersist.get(null);
      msgPersist.get(undefined);

      if (import.meta.env.DEV) {
        console.log('✅ Error handling: PASSED');
      this.testResults.push({
        test: 'Error Handling',
        status: 'PASSED',
        details: 'Invalid inputs handled gracefully'
      });

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Error handling test failed:', error);
      this.testResults.push({
        test: 'Error Handling',
        status: 'FAILED',
        error: error.message
      });

  /**
   * 6. 性能测试
   */
  testPerformance() {
    if (import.meta.env.DEV) {
      console.log('\n6️⃣ PERFORMANCE TEST');
    if (import.meta.env.DEV) {
      console.log('===================');
    }

    try {
      const msgPersist = window.msgPersist;
      const testMessage = {
        id: Date.now(),
        content: 'Performance test message',
        sender_id: 1,
        created_at: new Date().toISOString()
      };

      // 测试保存性能
      const saveStart = performance.now();
      msgPersist.save(9999, testMessage);
      const saveTime = performance.now() - saveStart;

      // 测试读取性能
      const getStart = performance.now();
      const retrieved = msgPersist.get(9999);
      const getTime = performance.now() - getStart;

      // 测试统计性能
      const statsStart = performance.now();
      const stats = msgPersist.stats();
      const statsTime = performance.now() - statsStart;

      if (import.meta.env.DEV) {
        console.log(`⏱️ Performance results:`);
      if (import.meta.env.DEV) {
        console.log(`   Save: ${saveTime.toFixed(2)}ms`);
      if (import.meta.env.DEV) {
        console.log(`   Get: ${getTime.toFixed(2)}ms`);
      if (import.meta.env.DEV) {
        console.log(`   Stats: ${statsTime.toFixed(2)}ms`);
      }

      // 验证性能目标 (< 5ms for basic operations)
      const allUnder5ms = saveTime < 5 && getTime < 5 && statsTime < 10;

      if (allUnder5ms) {
        if (import.meta.env.DEV) {
          console.log('✅ Performance: EXCELLENT');
        this.testResults.push({
          test: 'Performance',
          status: 'PASSED',
          details: `Save: ${saveTime.toFixed(2)}ms, Get: ${getTime.toFixed(2)}ms`
        });
      } else {
        if (import.meta.env.DEV) {
          console.log('⚠️ Performance: ACCEPTABLE (but could be better)');
        this.testResults.push({
          test: 'Performance',
          status: 'WARNING',
          details: `Some operations > 5ms threshold`
        });

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ Performance test failed:', error);
      this.testResults.push({
        test: 'Performance',
        status: 'FAILED',
        error: error.message
      });

  /**
   * 7. 生成测试报告
   */
  generateTestReport() {
    if (import.meta.env.DEV) {
      console.log('\n📊 FINAL TEST REPORT');
    if (import.meta.env.DEV) {
      console.log('====================\n');
    }

    const passed = this.testResults.filter(r => r.status === 'PASSED').length;
    const failed = this.testResults.filter(r => r.status === 'FAILED').length;
    const warnings = this.testResults.filter(r => r.status === 'WARNING').length;
    const skipped = this.testResults.filter(r => r.status === 'SKIPPED').length;
    const manual = this.testResults.filter(r => r.status === 'MANUAL_VERIFY').length;

    if (import.meta.env.DEV) {
      console.log('🔍 Test Results Summary:');
    this.testResults.forEach(result => {
      const emoji = {
        'PASSED': '✅',
        'FAILED': '❌',
        'WARNING': '⚠️',
        'SKIPPED': '⏭️',
        'MANUAL_VERIFY': '📋',
        'NO_DATA': '📭'
      }[result.status] || '❓';

      if (import.meta.env.DEV) {
        console.log(`   ${emoji} ${result.test}: ${result.status}`);
      if (result.details) {
        if (import.meta.env.DEV) {
          console.log(`      ${result.details}`);
      if (result.error) {
        if (import.meta.env.DEV) {
          console.log(`      Error: ${result.error}`);
        }
    });

    if (import.meta.env.DEV) {
      console.log(`\n📊 Summary: ${passed} passed, ${failed} failed, ${warnings} warnings, ${skipped} skipped, ${manual} manual`);
    }

    // 整体状态评估
    if (failed === 0 && passed >= 3) {
      if (import.meta.env.DEV) {
        console.log('🎉 OVERALL STATUS: SUCCESS - Message persistence is working correctly!');
      }
    } else if (failed === 0) {
      if (import.meta.env.DEV) {
        console.log('👍 OVERALL STATUS: GOOD - Basic functionality working');
      }
    } else {
      if (import.meta.env.DEV) {
        console.log('⚠️ OVERALL STATUS: ISSUES DETECTED - Review failed tests');
      }

    return {
      passed,
      failed,
      warnings,
      skipped,
      manual,
      overall: failed === 0 ? 'SUCCESS' : 'ISSUES'
    };
  }

  /**
   * 快速验证功能
   */
  quickVerify() {
    if (import.meta.env.DEV) {
      console.log('\n⚡ QUICK VERIFICATION');
    if (import.meta.env.DEV) {
      console.log('====================');
    }

    const chatStore = window.app._instance.proxy.$pinia._s.get('chat');
    const msgPersist = window.msgPersist;

    if (!msgPersist) {
      if (import.meta.env.DEV) {
        console.log('❌ Message persistence not loaded');
      return false;
    }

    if (!chatStore.currentChatId) {
      if (import.meta.env.DEV) {
        console.log('⚠️ No chat selected');
      return false;
    }

    const stats = msgPersist.stats();
    if (import.meta.env.DEV) {
      console.log(`✅ Message persistence loaded - ${stats.totalMessages} messages in ${stats.totalChats} chats`);
    }

    return true;
  }

// 创建全局实例并暴露到window
const messagePersistenceTest = new MessagePersistenceTest();

if (typeof window !== 'undefined') {
  window.testPersistence = {
    run: () => messagePersistenceTest.runCompleteTest(),
    quick: () => messagePersistenceTest.quickVerify(),
    sending: () => messagePersistenceTest.testMessageSending(),
    refresh: () => messagePersistenceTest.testRefreshRecovery()
  };

  if (import.meta.env.DEV) {
    console.log('🧪 Message Persistence Test loaded');
  if (import.meta.env.DEV) {
    console.log('   Commands:');
  if (import.meta.env.DEV) {
    console.log('   - window.testPersistence.run() - Full test suite');
  if (import.meta.env.DEV) {
    console.log('   - window.testPersistence.quick() - Quick verification');
  if (import.meta.env.DEV) {
    console.log('   - window.testPersistence.sending() - Test message sending');
  }

export default messagePersistenceTest; 