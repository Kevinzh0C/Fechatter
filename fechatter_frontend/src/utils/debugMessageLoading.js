/**
 * Debug tool for message loading issues
 */

export async function debugMessageLoading() {
  console.log('🐛 Debugging Message Loading...\n');

  try {
    // Import required modules
    const { useChatStore } = await import('@/stores/chat');
    const chatStore = useChatStore();

    // 1. Check current state
    console.log('📋 1. Current Chat State:');
    console.log('- Current chat ID:', chatStore.currentChatId);
    console.log('- Messages count:', chatStore.messages?.length || 0);
    console.log('- Loading state:', chatStore.loading);
    console.log('- Has message cache:', Object.keys(chatStore.messageCache || {}).length > 0);

    // 2. Check if fetchMessages exists
    console.log('\n📋 2. API Methods:');
    console.log('- fetchMessages exists:', typeof chatStore.fetchMessages === 'function');
    console.log('- setCurrentChat exists:', typeof chatStore.setCurrentChat === 'function');

    // 3. Test message loading
    if (chatStore.currentChatId) {
      console.log('\n📋 3. Testing message load for chat:', chatStore.currentChatId);
      try {
        const messages = await chatStore.fetchMessages(chatStore.currentChatId);
        console.log('✅ Messages loaded:', messages?.length || 0);
        if (messages && messages.length > 0) {
          console.log('First message:', {
            id: messages[0].id,
            content: messages[0].content?.substring(0, 50) + '...',
            sender: messages[0].sender_name
          });
        }
      } catch (error) {
        console.error('❌ Failed to load messages:', error.message);
      }
    } else {
      console.log('\n⚠️ No current chat selected');
    }

    // 4. Check message cache
    console.log('\n📋 4. Message Cache Status:');
    const cacheKeys = Object.keys(chatStore.messageCache || {});
    console.log('- Cached chats:', cacheKeys.length);
    cacheKeys.forEach(key => {
      const cache = chatStore.messageCache[key];
      console.log(`  - Chat ${key}: ${cache?.messages?.length || 0} messages`);
    });

    // 5. Test direct API call
    console.log('\n📋 5. Testing Direct API Call:');
    if (chatStore.currentChatId) {
      const { default: api } = await import('@/services/api');
      try {
        const response = await api.get(`/chat/${chatStore.currentChatId}/messages`);
        console.log('✅ Direct API call successful');
        console.log('- Status:', response.status);
        console.log('- Messages:', response.data?.data?.length || response.data?.length || 0);
      } catch (error) {
        console.error('❌ Direct API call failed:', error.message);
        if (error.response) {
          console.log('- Status:', error.response.status);
          console.log('- Error:', error.response.data);
        }
      }
    }

    return {
      currentChatId: chatStore.currentChatId,
      messageCount: chatStore.messages?.length || 0,
      cacheCount: cacheKeys.length,
      loading: chatStore.loading
    };

  } catch (error) {
    console.error('❌ Debug failed:', error);
    return { error: error.message };
  }
}

// Auto-run in development
if (import.meta.env.DEV) {
  window.debugMessageLoading = debugMessageLoading;
  console.log('📨 Message loading debug tool loaded - use window.debugMessageLoading()');
} 