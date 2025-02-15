/**
 * Message Order Fix
 * 消息排序修复 - 确保最新消息在底部显示（符合聊天应用标准）
 */

class MessageOrderFix {
  constructor() {
    this.testResults = {
      currentOrder: 'unknown',
      expectedOrder: 'latest_at_bottom',
      issues: [],
      fixed: false
    };

    console.log('🔧 Message Order Fix initialized');
  }

  /**
   * Analyze current message order
   * 分析当前消息顺序
   */
  analyzeCurrentOrder() {
    console.log('\n🔍 MESSAGE ORDER ANALYSIS');
    console.log('=========================\n');

    const chatStore = this.getChatStore();
    if (!chatStore) {
      console.log('❌ Chat store not available');
      return;
    }

    const messages = chatStore.messages || [];
    console.log(`📊 Analyzing ${messages.length} messages`);

    if (messages.length < 2) {
      console.log('⚠️ Need at least 2 messages to analyze order');
      return;
    }

    // Check current sort order
    const firstMessage = messages[0];
    const lastMessage = messages[messages.length - 1];

    const firstTime = new Date(firstMessage.created_at).getTime();
    const lastTime = new Date(lastMessage.created_at).getTime();

    console.log('📅 Message timestamps:');
    console.log(`   First message: ${firstMessage.created_at} (${this.formatTime(firstMessage.created_at)})`);
    console.log(`   Last message: ${lastMessage.created_at} (${this.formatTime(lastMessage.created_at)})`);

    // Determine current order
    let currentOrder = 'unknown';
    if (firstTime < lastTime) {
      currentOrder = 'chronological'; // 时间升序，最新在底部 ✅
      console.log('✅ Current order: CHRONOLOGICAL (oldest→newest) - CORRECT for chat apps');
    } else if (firstTime > lastTime) {
      currentOrder = 'reverse_chronological'; // 时间降序，最新在顶部 ❌
      console.log('❌ Current order: REVERSE CHRONOLOGICAL (newest→oldest) - WRONG for chat apps');
    } else {
      currentOrder = 'same_time';
      console.log('⚠️ Current order: SAME TIME - Cannot determine');
    }

    this.testResults.currentOrder = currentOrder;

    // Analyze entire array for consistency
    this.analyzeFullArray(messages);

    // Generate recommendations
    this.generateRecommendations(currentOrder);
  }

  /**
   * Analyze entire message array for ordering consistency
   */
  analyzeFullArray(messages) {
    console.log('\n📊 Full Array Analysis:');

    let ascendingCount = 0;
    let descendingCount = 0;
    let sameTimeCount = 0;

    for (let i = 0; i < messages.length - 1; i++) {
      const currentTime = new Date(messages[i].created_at).getTime();
      const nextTime = new Date(messages[i + 1].created_at).getTime();

      if (currentTime < nextTime) {
        ascendingCount++;
      } else if (currentTime > nextTime) {
        descendingCount++;
      } else {
        sameTimeCount++;
      }
    }

    console.log(`   Ascending pairs: ${ascendingCount} (oldest→newest)`);
    console.log(`   Descending pairs: ${descendingCount} (newest→oldest)`);
    console.log(`   Same time pairs: ${sameTimeCount}`);

    const totalPairs = messages.length - 1;
    const ascendingPercentage = (ascendingCount / totalPairs * 100).toFixed(1);
    const descendingPercentage = (descendingCount / totalPairs * 100).toFixed(1);

    console.log(`   Order consistency: ${ascendingPercentage}% ascending, ${descendingPercentage}% descending`);

    // Determine array consistency
    if (ascendingCount > descendingCount * 2) {
      console.log('✅ Array is mostly CHRONOLOGICAL (correct for chat)');
    } else if (descendingCount > ascendingCount * 2) {
      console.log('❌ Array is mostly REVERSE CHRONOLOGICAL (wrong for chat)');
      this.testResults.issues.push('Messages are sorted newest→oldest, should be oldest→newest');
    } else {
      console.log('⚠️ Array has MIXED ordering (inconsistent)');
      this.testResults.issues.push('Message ordering is inconsistent');
    }
  }

  /**
   * Generate recommendations based on analysis
   */
  generateRecommendations(currentOrder) {
    console.log('\n💡 RECOMMENDATIONS:');
    console.log('===================');

    if (currentOrder === 'reverse_chronological') {
      console.log('🔧 ISSUE: Messages are displaying newest→oldest');
      console.log('📋 SOLUTION: Sort messages by created_at ASC (oldest→newest)');
      console.log('🎯 EXPECTED: User sees oldest messages at top, newest at bottom');
      console.log('💬 STANDARD: Like WhatsApp, WeChat, Telegram, Discord');

      console.log('\n🚀 Quick Fix:');
      console.log('   window.messageOrderFix.sortMessagesCorrectly()');
    } else if (currentOrder === 'chronological') {
      console.log('✅ Messages are correctly ordered oldest→newest');
      console.log('✅ This is the standard for chat applications');
    } else {
      console.log('⚠️ Cannot determine clear ordering pattern');
      console.log('🔧 Recommend: Apply consistent chronological sorting');
    }
  }

  /**
   * Sort messages in correct chronological order (oldest→newest)
   */
  sortMessagesCorrectly() {
    console.log('\n🔧 FIXING MESSAGE ORDER');
    console.log('=======================');

    const chatStore = this.getChatStore();
    if (!chatStore) {
      console.log('❌ Cannot fix - chat store not available');
      return false;
    }

    const messages = chatStore.messages || [];
    if (messages.length === 0) {
      console.log('⚠️ No messages to sort');
      return false;
    }

    console.log(`🔧 Sorting ${messages.length} messages chronologically...`);

    // Store original order for comparison
    const originalFirst = messages[0];
    const originalLast = messages[messages.length - 1];

    // Sort messages by created_at ascending (oldest first, newest last)
    const sortedMessages = [...messages].sort((a, b) => {
      const timeA = new Date(a.created_at).getTime();
      const timeB = new Date(b.created_at).getTime();
      return timeA - timeB; // Ascending order
    });

    // Update store
    chatStore.messages = sortedMessages;

    // Verify the fix
    const newFirst = chatStore.messages[0];
    const newLast = chatStore.messages[chatStore.messages.length - 1];

    console.log('📊 Before/After comparison:');
    console.log(`   Original first: ${this.formatTime(originalFirst.created_at)}`);
    console.log(`   New first: ${this.formatTime(newFirst.created_at)}`);
    console.log(`   Original last: ${this.formatTime(originalLast.created_at)}`);
    console.log(`   New last: ${this.formatTime(newLast.created_at)}`);

    // Verify chronological order
    const firstTime = new Date(newFirst.created_at).getTime();
    const lastTime = new Date(newLast.created_at).getTime();

    if (firstTime <= lastTime) {
      console.log('✅ Messages now correctly sorted oldest→newest');
      console.log('✅ Newest messages will appear at bottom (standard chat behavior)');
      this.testResults.fixed = true;
      return true;
    } else {
      console.log('❌ Sort failed - still incorrect order');
      return false;
    }
  }

  /**
   * Apply fix to chat store and all caches
   */
  applyFixGlobally() {
    console.log('\n🌐 APPLYING GLOBAL MESSAGE ORDER FIX');
    console.log('====================================');

    const chatStore = this.getChatStore();
    if (!chatStore) {
      console.log('❌ Cannot apply global fix - chat store not available');
      return false;
    }

    let fixCount = 0;

    // Fix current messages
    if (this.sortMessagesCorrectly()) {
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

    // Fix localStorage persistence if available
    if (window.msgPersist) {
      console.log('🔧 Fixing localStorage persistence...');
      try {
        // This will trigger a re-sort in persistence layer
        window.msgPersist.fixMessageOrder();
        fixCount++;
      } catch (error) {
        console.warn('⚠️ Could not fix localStorage persistence:', error.message);
      }
    }

    console.log(`✅ Applied fixes to ${fixCount} locations`);
    return fixCount > 0;
  }

  /**
   * Test the fix by creating sample messages
   */
  testWithSampleMessages() {
    console.log('\n🧪 TESTING WITH SAMPLE MESSAGES');
    console.log('===============================');

    const chatStore = this.getChatStore();
    if (!chatStore) {
      console.log('❌ Cannot test - chat store not available');
      return;
    }

    // Create sample messages with different timestamps
    const now = Date.now();
    const sampleMessages = [
      {
        id: 1,
        content: 'Message from 10 minutes ago',
        created_at: new Date(now - 10 * 60 * 1000).toISOString(),
        sender: { id: 1, fullname: 'User 1' }
      },
      {
        id: 2,
        content: 'Message from 5 minutes ago',
        created_at: new Date(now - 5 * 60 * 1000).toISOString(),
        sender: { id: 2, fullname: 'User 2' }
      },
      {
        id: 3,
        content: 'Message from 1 minute ago',
        created_at: new Date(now - 1 * 60 * 1000).toISOString(),
        sender: { id: 1, fullname: 'User 1' }
      },
      {
        id: 4,
        content: 'Most recent message',
        created_at: new Date(now).toISOString(),
        sender: { id: 2, fullname: 'User 2' }
      }
    ];

    // Shuffle messages to simulate wrong order
    const shuffledMessages = [...sampleMessages].sort(() => Math.random() - 0.5);

    console.log('📝 Created sample messages (shuffled):');
    shuffledMessages.forEach((msg, index) => {
      console.log(`   ${index + 1}. "${msg.content}" - ${this.formatTime(msg.created_at)}`);
    });

    // Apply to store
    chatStore.messages = shuffledMessages;

    // Analyze and fix
    this.analyzeCurrentOrder();
    const fixed = this.sortMessagesCorrectly();

    if (fixed) {
      console.log('\n✅ Test completed successfully:');
      chatStore.messages.forEach((msg, index) => {
        console.log(`   ${index + 1}. "${msg.content}" - ${this.formatTime(msg.created_at)}`);
      });
    }
  }

  /**
   * Format time for display
   */
  formatTime(timestamp) {
    try {
      const date = new Date(timestamp);
      return date.toLocaleTimeString() + ' ' + date.toLocaleDateString();
    } catch (error) {
      return timestamp;
    }
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
}

// Create global instance
const messageOrderFix = new MessageOrderFix();

// Expose to window
if (typeof window !== 'undefined') {
  window.messageOrderFix = {
    analyze: () => messageOrderFix.analyzeCurrentOrder(),
    fix: () => messageOrderFix.sortMessagesCorrectly(),
    fixGlobal: () => messageOrderFix.applyFixGlobally(),
    test: () => messageOrderFix.testWithSampleMessages()
  };

  console.log('🔧 Message Order Fix loaded');
  console.log('   Commands:');
  console.log('   - window.messageOrderFix.analyze() - Analyze current order');
  console.log('   - window.messageOrderFix.fix() - Fix message order');
  console.log('   - window.messageOrderFix.fixGlobal() - Fix globally');
  console.log('   - window.messageOrderFix.test() - Test with samples');
}

export default messageOrderFix; 