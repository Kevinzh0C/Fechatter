/**
 * Message Loading Diagnostic Tool
 * Comprehensive analysis of message loading chain
 */

async function diagnoseMessageLoading() {
  console.group('🔍 Message Loading Diagnostic');

  try {
    const { useChatStore } = await import('@/stores/chat');
    const { useRoute, useRouter } = await import('vue-router');
    const chatStore = useChatStore();
    const route = useRoute();
    const router = useRouter();

    console.log('\n1️⃣ Current State Analysis');
    console.log('  - Route path:', route.path);
    console.log('  - Route params.id:', route.params.id);
    console.log('  - Chat store currentChatId:', chatStore.currentChatId);
    console.log('  - Messages loaded:', chatStore.messages.length);
    console.log('  - Loading state:', chatStore.loading);

    console.log('\n2️⃣ Function Call Chain Test');

    if (!route.params.id) {
      console.error('❌ Not on a chat page. Navigate to a chat first.');
      return;
    }

    const chatId = parseInt(route.params.id, 10);

    // Test setCurrentChat
    console.log('\n  Testing setCurrentChat...');
    console.log('  - Current ID before:', chatStore.currentChatId);

    // Force reset to test
    chatStore.currentChatId = null;

    try {
      await chatStore.setCurrentChat(chatId);
      console.log('  ✅ setCurrentChat completed');
      console.log('  - Current ID after:', chatStore.currentChatId);
      console.log('  - Messages loaded:', chatStore.messages.length);
    } catch (error) {
      console.error('  ❌ setCurrentChat failed:', error);
    }

    console.log('\n3️⃣ Direct fetchMessages Test');
    try {
      console.log('  - Calling fetchMessages directly...');
      const messages = await chatStore.fetchMessages(chatId);
      console.log('  ✅ fetchMessages returned:', messages.length, 'messages');
    } catch (error) {
      console.error('  ❌ fetchMessages failed:', error);
    }

    console.log('\n4️⃣ API Request Analysis');
    // Check recent network requests
    const resources = performance.getEntriesByType('resource');
    const chatRequests = resources.filter(r =>
      r.name.includes('/chat/') &&
      r.startTime > (performance.now() - 5000) // Last 5 seconds
    );

    console.log('  Recent chat-related requests:');
    chatRequests.forEach(req => {
      const url = new URL(req.name);
      console.log(`  - ${url.pathname}: ${req.duration.toFixed(2)}ms`);
    });

    console.log('\n5️⃣ Cache Status');
    console.log('  - Message cache keys:', Object.keys(chatStore.messageCache));
    console.log('  - Has cache for current chat:', !!chatStore.messageCache[chatId]);
    if (chatStore.messageCache[chatId]) {
      console.log('  - Cached messages:', chatStore.messageCache[chatId].messages.length);
      console.log('  - Cache age:',
        ((Date.now() - chatStore.messageCache[chatId].timestamp) / 1000).toFixed(1),
        'seconds'
      );
    }

    console.log('\n6️⃣ Component Integration Check');
    const messageList = document.querySelector('.message-list');
    const messageItems = document.querySelectorAll('.message-item');
    const loadingState = document.querySelector('.loading-state');

    console.log('  - MessageList component found:', !!messageList);
    console.log('  - Message items in DOM:', messageItems.length);
    console.log('  - Loading state visible:', !!loadingState);

    console.log('\n📊 Diagnosis Summary');
    if (chatStore.messages.length > 0 && messageItems.length > 0) {
      console.log('  ✅ Messages are loading and displaying correctly');
    } else if (chatStore.messages.length > 0 && messageItems.length === 0) {
      console.log('  ⚠️ Messages in store but not rendered');
      console.log('  Possible causes:');
      console.log('  - Vue reactivity issue');
      console.log('  - MessageList component not receiving props');
      console.log('  - Component not mounted');
    } else if (chatStore.messages.length === 0) {
      console.log('  ❌ No messages in store');
      console.log('  Possible causes:');
      console.log('  - API request failed');
      console.log('  - setCurrentChat not called properly');
      console.log('  - fetchMessages threw an error');
    }

    // Provide fix suggestions
    console.log('\n🔧 Suggested Actions');
    if (chatStore.messages.length === 0) {
      console.log('  1. Run: window.forceMessageReload()');
      console.log('  2. Check browser console for errors');
      console.log('  3. Check Network tab for failed requests');
      console.log('  4. Try: window.testMessageDisplay()');
    }

  } catch (error) {
    console.error('❌ Diagnostic failed:', error);
  }

  console.groupEnd();
}

// Force reload messages for current chat
async function forceMessageReload() {
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

    console.log('🔄 Force reloading messages for chat', chatId);

    // Clear cache
    delete chatStore.messageCache[chatId];

    // Reset current chat
    chatStore.currentChatId = null;
    chatStore.messages.length = 0;

    // Reload
    await chatStore.setCurrentChat(chatId);

    console.log('✅ Reload complete:', chatStore.messages.length, 'messages loaded');

  } catch (error) {
    console.error('❌ Force reload failed:', error);
  }
}

// Manual API test
async function testMessageAPI() {
  try {
    const { default: api } = await import('@/services/api');
    const { useRoute } = await import('vue-router');
    const route = useRoute();

    if (!route.params.id) {
      console.error('❌ Not on a chat page');
      return;
    }

    const chatId = parseInt(route.params.id, 10);

    console.log('🧪 Testing message API directly');
    console.log('  - Requesting: /chat/' + chatId + '/messages');

    const response = await api.get(`/chat/${chatId}/messages`);

    console.log('✅ API Response:', {
      status: response.status,
      dataStructure: response.data ? Object.keys(response.data) : null,
      messageCount: response.data?.data?.length || response.data?.length || 0
    });

    if (response.data?.data) {
      console.log('  - First message:', response.data.data[0]);
    }

  } catch (error) {
    console.error('❌ API test failed:', error.response || error);
  }
}

// Export functions
if (typeof window !== 'undefined') {
  window.diagnoseMessageLoading = diagnoseMessageLoading;
  window.forceMessageReload = forceMessageReload;
  window.testMessageAPI = testMessageAPI;

  console.log('🔍 Message loading diagnostic loaded:');
  console.log('  - window.diagnoseMessageLoading() - Run full diagnostic');
  console.log('  - window.forceMessageReload() - Force reload messages');
  console.log('  - window.testMessageAPI() - Test API directly');
}

export { diagnoseMessageLoading, forceMessageReload, testMessageAPI }; 