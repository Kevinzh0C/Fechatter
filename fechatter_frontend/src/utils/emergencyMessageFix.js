/**
 * Emergency Message Display Fix
 * Quick solution to get messages showing immediately
 */

export async function emergencyMessageFix() {
  console.log('🚨 Applying Emergency Message Fix...');

  try {
    // Import stores
    const { useChatStore } = await import('@/stores/chat');
    const chatStore = useChatStore();

    // 1. Force clear cache
    console.log('1️⃣ Clearing all caches...');
    chatStore.messageCache = {};
    sessionStorage.removeItem('messageCache');

    // 2. Reset current chat state
    console.log('2️⃣ Resetting chat state...');
    const currentId = chatStore.currentChatId;
    if (!currentId) {
      console.error('❌ No chat selected!');
      return false;
    }

    // 3. Force message array to be reactive
    console.log('3️⃣ Ensuring messages array reactivity...');
    if (!Array.isArray(chatStore.messages)) {
      chatStore.messages = [];
    }

    // 4. Direct API fetch with immediate assignment
    console.log('4️⃣ Fetching messages directly...');
    const { default: api } = await import('@/services/api');

    try {
      const response = await api.get(`/chat/${currentId}/messages`);
      const rawMessages = response.data?.data || response.data || [];

      console.log(`✅ Fetched ${rawMessages.length} messages from API`);

      // 5. Normalize and assign with Vue reactivity
      console.log('5️⃣ Normalizing and assigning messages...');

      // Clear existing messages
      chatStore.messages.length = 0;

      // Process each message
      const normalizedMessages = rawMessages.map(msg => {
        const normalized = {
          id: msg.id,
          content: msg.content || '',
          sender_id: msg.sender_id,
          sender_name: msg.sender?.fullname || msg.sender_name || 'Unknown',
          sender: msg.sender || { fullname: msg.sender_name || 'Unknown' },
          created_at: msg.created_at,
          chat_id: msg.chat_id || currentId,
          files: msg.files || [],
          status: 'sent',
          _timestamp: new Date(msg.created_at).getTime()
        };
        return normalized;
      });

      // Sort by timestamp
      normalizedMessages.sort((a, b) => a._timestamp - b._timestamp);

      // Push each message individually to trigger reactivity
      normalizedMessages.forEach(msg => {
        chatStore.messages.push(msg);
      });

      console.log(`✅ Added ${chatStore.messages.length} messages to store`);

      // 6. Force Vue to update
      console.log('6️⃣ Forcing Vue update...');

      // Trigger a dummy reactive update
      chatStore.currentChatId = null;
      await new Promise(resolve => setTimeout(resolve, 10));
      chatStore.currentChatId = currentId;

      // 7. Check if messages are visible
      console.log('7️⃣ Verifying messages...');
      console.log('Current messages:', chatStore.messages.length);
      console.log('First message:', chatStore.messages[0]);
      console.log('Last message:', chatStore.messages[chatStore.messages.length - 1]);

      // 8. Connect SSE for real-time updates
      console.log('8️⃣ Checking SSE connection...');
      try {
        const minimalSSE = await import('@/services/sse-minimal');
        if (minimalSSE.default && !minimalSSE.default.connected) {
          console.log('Connecting SSE...');
          const { useAuthStore } = await import('@/stores/auth');
          const authStore = useAuthStore();
          if (authStore.token) {
            minimalSSE.default.connect(authStore.token);
          }
        }
      } catch (e) {
        console.warn('SSE connection skipped:', e.message);
      }

      console.log('✅ Emergency fix applied successfully!');

      // Return diagnostic info
      return {
        success: true,
        messageCount: chatStore.messages.length,
        currentChatId: chatStore.currentChatId,
        firstMessage: chatStore.messages[0]?.content,
        lastMessage: chatStore.messages[chatStore.messages.length - 1]?.content
      };

    } catch (apiError) {
      console.error('❌ API Error:', apiError);
      throw apiError;
    }

  } catch (error) {
    console.error('❌ Emergency fix failed:', error);
    return { success: false, error: error.message };
  }
}

// Quick helper to manually add a test message
export function addTestMessage() {
  try {
    const { useChatStore } = require('@/stores/chat');
    const { useAuthStore } = require('@/stores/auth');
    const chatStore = useChatStore();
    const authStore = useAuthStore();

    const testMessage = {
      id: `test_${Date.now()}`,
      content: `Test message at ${new Date().toLocaleTimeString()}`,
      sender_id: authStore.user?.id || 'test_user',
      sender_name: authStore.user?.fullname || 'Test User',
      sender: authStore.user || { fullname: 'Test User' },
      created_at: new Date().toISOString(),
      chat_id: chatStore.currentChatId,
      files: [],
      status: 'sent',
      _timestamp: Date.now()
    };

    chatStore.messages.push(testMessage);
    console.log('✅ Test message added:', testMessage);

    return testMessage;
  } catch (error) {
    console.error('❌ Failed to add test message:', error);
  }
}

// Auto-register in development
if (import.meta.env.DEV) {
  window.fixMessages = emergencyMessageFix;
  window.addTestMessage = addTestMessage;
  console.log('🚨 Emergency message fix loaded:');
  console.log('   - window.fixMessages() to fix message display');
  console.log('   - window.addTestMessage() to add a test message');
} 