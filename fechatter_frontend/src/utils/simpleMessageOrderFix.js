/**
 * Simple Message Order Fix
 * 简单消息排序修复 - 直接修复逻辑错误
 */

class SimpleMessageOrderFix {
  constructor() {
    console.log('🔧 Simple Message Order Fix initialized');
  }

  /**
   * Fix current messages immediately
   * 立即修复当前消息排序
   */
  fixNow() {
    console.log('\n🔧 FIXING MESSAGE ORDER NOW');
    console.log('===========================');

    const chatStore = this.getChatStore();
    if (!chatStore) {
      console.log('❌ Chat store not found');
      return false;
    }

    const messages = chatStore.messages || [];
    if (messages.length === 0) {
      console.log('⚠️ No messages to fix');
      return false;
    }

    console.log(`🔧 Current messages: ${messages.length}`);
    console.log(`   First: ${this.formatTime(messages[0].created_at)} - "${messages[0].content?.substring(0, 30)}..."`);
    console.log(`   Last:  ${this.formatTime(messages[messages.length - 1].created_at)} - "${messages[messages.length - 1].content?.substring(0, 30)}..."`);

    // Sort messages chronologically (oldest first)
    messages.sort((a, b) => {
      const timeA = new Date(a.created_at).getTime();
      const timeB = new Date(b.created_at).getTime();
      return timeA - timeB; // Ascending order
    });

    console.log(`✅ Fixed messages:`);
    console.log(`   First: ${this.formatTime(messages[0].created_at)} - "${messages[0].content?.substring(0, 30)}..."`);
    console.log(`   Last:  ${this.formatTime(messages[messages.length - 1].created_at)} - "${messages[messages.length - 1].content?.substring(0, 30)}..."`);

    // Trigger reactivity
    chatStore.messages = [...messages];

    console.log('✅ Message order fixed! Messages now display oldest→newest');
    return true;
  }

  /**
   * Fix all cached messages
   * 修复所有缓存的消息
   */
  fixAllCaches() {
    console.log('\n🔧 FIXING ALL MESSAGE CACHES');
    console.log('=============================');

    const chatStore = this.getChatStore();
    if (!chatStore) {
      console.log('❌ Chat store not found');
      return false;
    }

    let fixCount = 0;

    // Fix current messages
    if (this.fixNow()) {
      fixCount++;
    }

    // Fix message cache
    const messageCache = chatStore.messageCache || {};
    Object.keys(messageCache).forEach(chatId => {
      const cache = messageCache[chatId];
      if (cache && cache.messages && cache.messages.length > 0) {
        console.log(`🔧 Fixing cache for chat ${chatId} (${cache.messages.length} messages)`);

        cache.messages.sort((a, b) => {
          const timeA = new Date(a.created_at).getTime();
          const timeB = new Date(b.created_at).getTime();
          return timeA - timeB; // Ascending order
        });

        fixCount++;
      }
    });

    console.log(`✅ Fixed ${fixCount} message collections`);
    return fixCount > 0;
  }

  /**
   * Verify current message order
   * 验证当前消息顺序
   */
  verify() {
    console.log('\n🔍 VERIFYING MESSAGE ORDER');
    console.log('==========================');

    const chatStore = this.getChatStore();
    if (!chatStore) {
      console.log('❌ Chat store not found');
      return false;
    }

    const messages = chatStore.messages || [];
    if (messages.length < 2) {
      console.log('⚠️ Not enough messages to verify order');
      return true;
    }

    let isCorrect = true;
    let violations = 0;

    for (let i = 1; i < messages.length; i++) {
      const prevTime = new Date(messages[i - 1].created_at).getTime();
      const currTime = new Date(messages[i].created_at).getTime();

      if (prevTime > currTime) {
        isCorrect = false;
        violations++;
        console.log(`❌ Order violation at position ${i}:`);
        console.log(`   Previous: ${this.formatTime(messages[i - 1].created_at)}`);
        console.log(`   Current:  ${this.formatTime(messages[i].created_at)}`);
      }
    }

    if (isCorrect) {
      console.log('✅ Message order is CORRECT (oldest→newest)');
      console.log(`   First: ${this.formatTime(messages[0].created_at)}`);
      console.log(`   Last:  ${this.formatTime(messages[messages.length - 1].created_at)}`);
    } else {
      console.log(`❌ Message order is WRONG (${violations} violations)`);
      console.log('   Expected: oldest→newest');
      console.log('   Run window.simpleFix.fixNow() to correct');
    }

    return isCorrect;
  }

  /**
   * Get chat store reference
   */
  getChatStore() {
    try {
      return window.app?._instance?.proxy?.$pinia?._s?.get('chat');
    } catch (error) {
      return null;
    }
  }

  /**
   * Format time for display
   */
  formatTime(timestamp) {
    try {
      return new Date(timestamp).toLocaleTimeString();
    } catch (error) {
      return timestamp;
    }
  }
}

// Create global instance
const simpleMessageOrderFix = new SimpleMessageOrderFix();

// Export for use
export default simpleMessageOrderFix;

// Expose to window
if (typeof window !== 'undefined') {
  window.simpleFix = {
    fix: () => simpleMessageOrderFix.fixNow(),
    fixAll: () => simpleMessageOrderFix.fixAllCaches(),
    verify: () => simpleMessageOrderFix.verify(),
    run: () => {
      console.log('🔧 Running simple message order fix...');
      const result = simpleMessageOrderFix.fixAllCaches();
      simpleMessageOrderFix.verify();
      return result;
    }
  };

  console.log('🔧 Simple Message Order Fix loaded');
  console.log('   Commands:');
  console.log('   - window.simpleFix.run() - Fix all and verify');
  console.log('   - window.simpleFix.fix() - Fix current messages');
  console.log('   - window.simpleFix.verify() - Check order');
} 