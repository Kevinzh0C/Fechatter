/**
 * Strict Channel Validation Test
 * Verify that messages ONLY appear in their correct channels
 */

async function testStrictChannelValidation() {
  console.group('🔍 Strict Channel Validation Test');

  try {
    const { useChatStore } = await import('@/stores/chat');
    const { useRoute, useRouter } = await import('vue-router');
    const chatStore = useChatStore();
    const route = useRoute();
    const router = useRouter();

    console.log('\n📊 Test Setup');
    console.log('  - Current route:', route.path);
    console.log('  - Current chat ID:', chatStore.currentChatId);
    console.log('  - Total chats:', chatStore.chats.length);
    console.log('  - Validator enabled:', window.channelMessageValidator?.validationEnabled);

    // Get test channels
    const testChats = chatStore.chats.slice(0, 3);
    if (testChats.length < 2) {
      console.error('❌ Need at least 2 channels to test');
      return;
    }

    console.log('\n🧪 Testing Channels:', testChats.map(c => ({
      id: c.id,
      name: c.name
    })));

    const results = [];

    // Test each channel
    for (let i = 0; i < testChats.length; i++) {
      const chat = testChats[i];
      console.log(`\n🔄 Testing Channel ${i + 1}: ${chat.name} (ID: ${chat.id})`);

      // Navigate to the channel
      await router.push(`/chat/${chat.id}`);

      // Wait for navigation
      await new Promise(resolve => setTimeout(resolve, 500));

      // Force a fresh load
      await chatStore.setCurrentChat(chat.id);

      // Wait for messages to load
      await new Promise(resolve => setTimeout(resolve, 1000));

      // Validate loaded messages
      const validation = {
        chatId: chat.id,
        chatName: chat.name,
        totalMessages: chatStore.messages.length,
        validMessages: 0,
        invalidMessages: 0,
        wrongChannelMessages: []
      };

      // Check each message
      chatStore.messages.forEach(msg => {
        const msgChatId = parseInt(msg.chat_id, 10);
        const expectedChatId = parseInt(chat.id, 10);

        if (msgChatId === expectedChatId) {
          validation.validMessages++;
        } else {
          validation.invalidMessages++;
          validation.wrongChannelMessages.push({
            id: msg.id,
            content: msg.content?.substring(0, 30) + '...',
            belongsTo: msgChatId,
            displayedIn: expectedChatId
          });
        }
      });

      results.push(validation);

      // Log immediate results
      if (validation.invalidMessages > 0) {
        console.error(`❌ Found ${validation.invalidMessages} messages from wrong channel!`);
        console.table(validation.wrongChannelMessages);
      } else {
        console.log(`✅ All ${validation.validMessages} messages belong to correct channel`);
      }
    }

    // Summary
    console.log('\n📊 Test Summary');
    console.table(results.map(r => ({
      channel: r.chatName,
      messages: r.totalMessages,
      valid: r.validMessages,
      invalid: r.invalidMessages,
      status: r.invalidMessages === 0 ? '✅ PASS' : '❌ FAIL'
    })));

    // Check validator statistics
    if (window.channelMessageValidator) {
      const report = window.channelMessageValidator.getReport();
      console.log('\n🛡️ Validator Report');
      console.log('  - Total violations:', report.violationCount);
      console.log('  - Summary:', report.summary);
    }

    // Overall result
    const totalInvalid = results.reduce((sum, r) => sum + r.invalidMessages, 0);
    if (totalInvalid === 0) {
      console.log('\n✅ SUCCESS: All messages displayed in correct channels!');
    } else {
      console.error(`\n❌ FAILURE: ${totalInvalid} messages displayed in wrong channels!`);
    }

    return results;

  } catch (error) {
    console.error('❌ Test failed:', error);
  }

  console.groupEnd();
}

// Test extension blocking
async function testExtensionBlocking() {
  console.group('🛡️ Extension Blocking Test');

  try {
    if (!window.extensionBlocker) {
      console.error('❌ Extension blocker not loaded');
      return;
    }

    const stats = window.extensionBlocker.getStats();
    console.log('\n📊 Blocker Statistics');
    console.log('  - Initialized:', stats.initialized);
    console.log('  - Blocked requests:', stats.blockedCount);
    console.log('  - Detected extensions:', stats.detectedExtensions);

    // Simulate extension request
    console.log('\n🧪 Testing extension request blocking...');

    try {
      await fetch('chrome-extension://fake/test.js');
      console.error('❌ Extension request was NOT blocked!');
    } catch (error) {
      console.log('✅ Extension request successfully blocked:', error.message);
    }

    // Check for extension elements
    const extensionElements = document.querySelectorAll('[class*="translate"], [class*="grammarly"], [id*="extension"]');
    console.log(`\n🔍 Found ${extensionElements.length} potential extension elements`);

    if (extensionElements.length > 0) {
      console.log('Elements:', Array.from(extensionElements).map(el => ({
        tag: el.tagName,
        class: el.className,
        id: el.id
      })));
    }

  } catch (error) {
    console.error('❌ Test failed:', error);
  }

  console.groupEnd();
}

// Quick fix for current channel
async function fixCurrentChannel() {
  console.group('🔧 Fixing Current Channel');

  try {
    const { useChatStore } = await import('@/stores/chat');
    const { useRoute } = await import('vue-router');
    const chatStore = useChatStore();
    const route = useRoute();

    const routeChatId = route.params.id ? parseInt(route.params.id, 10) : null;

    if (!routeChatId) {
      console.error('❌ No chat ID in route');
      return;
    }

    console.log('🔄 Forcing refresh for chat:', routeChatId);

    // Clear messages
    chatStore.messages.length = 0;

    // Clear cache for this chat
    if (chatStore.messageCache[routeChatId]) {
      delete chatStore.messageCache[routeChatId];
      sessionStorage.setItem('messageCache', JSON.stringify(chatStore.messageCache));
    }

    // Force reload
    chatStore.currentChatId = null; // Reset to force reload
    await chatStore.setCurrentChat(routeChatId);

    console.log('✅ Channel fixed. Messages:', chatStore.messages.length);

    // Validate
    const invalid = chatStore.messages.filter(m =>
      parseInt(m.chat_id, 10) !== routeChatId
    );

    if (invalid.length > 0) {
      console.error(`❌ Still have ${invalid.length} invalid messages!`);
    } else {
      console.log('✅ All messages belong to correct channel');
    }

  } catch (error) {
    console.error('❌ Fix failed:', error);
  }

  console.groupEnd();
}

// Export functions
if (typeof window !== 'undefined') {
  window.testStrictChannelValidation = testStrictChannelValidation;
  window.testExtensionBlocking = testExtensionBlocking;
  window.fixCurrentChannel = fixCurrentChannel;

  console.log('🔍 Strict Channel Validation Test loaded:');
  console.log('  - window.testStrictChannelValidation() - Test channel message correspondence');
  console.log('  - window.testExtensionBlocking() - Test extension blocking');
  console.log('  - window.fixCurrentChannel() - Fix current channel messages');
}

export { testStrictChannelValidation, testExtensionBlocking, fixCurrentChannel }; 