/**
 * Test Message Display Functionality
 * Verifies the complete message loading chain
 */

async function testMessageDisplay() {
  console.group('🧪 Testing Message Display Chain');

  try {
    const { useChatStore } = await import('@/stores/chat');
    const { useRoute } = await import('vue-router');
    const chatStore = useChatStore();
    const route = useRoute();

    console.log('\n1️⃣ Current State Check');
    console.log('  - Current route:', route.path);
    console.log('  - Route params.id:', route.params.id);
    console.log('  - Store currentChatId:', chatStore.currentChatId);
    console.log('  - Messages count:', chatStore.messages.length);
    console.log('  - Loading state:', chatStore.loading);

    console.log('\n2️⃣ Chat Store State');
    console.log('  - Total chats:', chatStore.chats.length);
    console.log('  - Message cache keys:', Object.keys(chatStore.messageCache));

    // Check if we're on a chat page
    if (route.params.id) {
      const chatId = parseInt(route.params.id, 10);

      console.log('\n3️⃣ Testing Message Load for Chat', chatId);

      // Force reload messages
      console.log('  - Calling setCurrentChat...');
      await chatStore.setCurrentChat(chatId);

      console.log('\n4️⃣ After Load Results');
      console.log('  - Store currentChatId:', chatStore.currentChatId);
      console.log('  - Messages loaded:', chatStore.messages.length);
      console.log('  - Loading state:', chatStore.loading);

      if (chatStore.messages.length > 0) {
        console.log('  - First message:', {
          id: chatStore.messages[0].id,
          content: chatStore.messages[0].content.substring(0, 50) + '...',
          sender: chatStore.messages[0].sender?.fullname
        });
        console.log('  - Last message:', {
          id: chatStore.messages[chatStore.messages.length - 1].id,
          content: chatStore.messages[chatStore.messages.length - 1].content.substring(0, 50) + '...',
          sender: chatStore.messages[chatStore.messages.length - 1].sender?.fullname
        });
      }

      // Check Vue component
      console.log('\n5️⃣ Vue Component Check');
      const messageList = document.querySelector('.message-list');
      const messageItems = document.querySelectorAll('.message-item');
      console.log('  - Message list element found:', !!messageList);
      console.log('  - Message items in DOM:', messageItems.length);

      // Check for loading state
      const loadingElement = document.querySelector('.loading-state');
      console.log('  - Loading state visible:', !!loadingElement);

      // Summary
      console.log('\n📊 Summary');
      if (chatStore.messages.length > 0 && messageItems.length > 0) {
        console.log('  ✅ Messages are loaded and displayed correctly');
      } else if (chatStore.messages.length > 0 && messageItems.length === 0) {
        console.log('  ⚠️ Messages loaded in store but not rendered in DOM');
        console.log('  - Check Vue component reactivity');
        console.log('  - Check MessageList component props');
      } else if (chatStore.messages.length === 0) {
        console.log('  ❌ No messages loaded');
        console.log('  - Check API response');
        console.log('  - Check network requests');
        console.log('  - Check error logs');
      }

      // Network check
      console.log('\n6️⃣ Recent Network Requests');
      const messageRequests = performance.getEntriesByType('resource')
        .filter(entry => entry.name.includes('/messages'))
        .slice(-3);

      messageRequests.forEach((req, i) => {
        console.log(`  Request ${i + 1}:`, {
          url: req.name,
          duration: req.duration.toFixed(2) + 'ms',
          status: req.responseStatus || 'unknown'
        });
      });

    } else {
      console.log('\n⚠️ Not on a chat page. Navigate to a chat first.');
    }

  } catch (error) {
    console.error('❌ Test failed:', error);
  }

  console.groupEnd();
}

// Manual message injection for testing
async function injectTestMessage() {
  try {
    const { useChatStore } = await import('@/stores/chat');
    const { useAuthStore } = await import('@/stores/auth');
    const chatStore = useChatStore();
    const authStore = useAuthStore();

    if (!chatStore.currentChatId) {
      console.error('❌ No current chat selected');
      return;
    }

    const testMessage = {
      id: Date.now(),
      chat_id: chatStore.currentChatId,
      content: `Test message injected at ${new Date().toLocaleTimeString()}`,
      sender_id: authStore.user?.id || 1,
      sender: authStore.user || { id: 1, fullname: 'Test User' },
      created_at: new Date().toISOString(),
      _timestamp: Date.now()
    };

    chatStore.messages.push(chatStore.normalizeMessage(testMessage));
    console.log('✅ Test message injected. Check if it appears in the UI.');

  } catch (error) {
    console.error('❌ Failed to inject message:', error);
  }
}

// Force refresh messages
async function forceRefreshMessages() {
  try {
    const { useChatStore } = await import('@/stores/chat');
    const { useRoute } = await import('vue-router');
    const chatStore = useChatStore();
    const route = useRoute();

    if (!route.params.id) {
      console.error('❌ Not on a chat page');
      return;
    }

    const chatId = parseInt(route.params.id, 10);

    console.log('🔄 Force refreshing messages for chat', chatId);

    // Clear cache
    delete chatStore.messageCache[chatId];

    // Clear current messages
    chatStore.messages.length = 0;

    // Reload
    await chatStore.fetchMessages(chatId);

    console.log('✅ Messages refreshed:', chatStore.messages.length, 'messages loaded');

  } catch (error) {
    console.error('❌ Failed to refresh:', error);
  }
}

// Export functions
if (typeof window !== 'undefined') {
  window.testMessageDisplay = testMessageDisplay;
  window.injectTestMessage = injectTestMessage;
  window.forceRefreshMessages = forceRefreshMessages;

  console.log('💬 Message display test loaded:');
  console.log('  - window.testMessageDisplay() - Run complete diagnostic');
  console.log('  - window.injectTestMessage() - Inject a test message');
  console.log('  - window.forceRefreshMessages() - Force reload messages');
}

export { testMessageDisplay, injectTestMessage, forceRefreshMessages }; 