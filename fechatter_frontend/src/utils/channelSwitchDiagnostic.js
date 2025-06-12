/**
 * Channel Switch Diagnostic
 * Diagnose why all channels show the same message list
 */

async function diagnoseChannelSwitch() {
  console.group('🔄 Channel Switch Diagnostic');

  try {
    const { useChatStore } = await import('@/stores/chat');
    const { useRoute } = await import('vue-router');
    const chatStore = useChatStore();
    const route = useRoute();

    console.log('\n1️⃣ Current State');
    console.log('  - Route chat ID:', route.params.id);
    console.log('  - Store currentChatId:', chatStore.currentChatId);
    console.log('  - Messages count:', chatStore.messages.length);
    console.log('  - Loading:', chatStore.loading);

    if (chatStore.messages.length > 0) {
      const firstMsg = chatStore.messages[0];
      const lastMsg = chatStore.messages[chatStore.messages.length - 1];
      console.log('  - First message chat_id:', firstMsg.chat_id);
      console.log('  - Last message chat_id:', lastMsg.chat_id);

      // Check if all messages belong to the same chat
      const chatIds = new Set(chatStore.messages.map(msg => msg.chat_id));
      console.log('  - Unique chat IDs in messages:', Array.from(chatIds));
    }

    console.log('\n2️⃣ Message Cache Analysis');
    console.log('  - Cache keys:', Object.keys(chatStore.messageCache));
    Object.entries(chatStore.messageCache).forEach(([chatId, cache]) => {
      console.log(`  - Chat ${chatId}: ${cache.messages.length} cached messages`);
    });

    console.log('\n3️⃣ Issue Identification');

    // Check if route and store are out of sync
    const routeChatId = route.params.id ? parseInt(route.params.id, 10) : null;
    if (routeChatId && routeChatId !== chatStore.currentChatId) {
      console.error('  ❌ Route and store are out of sync!');
      console.log(`     Route says chat ${routeChatId}, store says chat ${chatStore.currentChatId}`);
    }

    // Check if messages belong to wrong chat
    if (chatStore.messages.length > 0 && routeChatId) {
      const wrongMessages = chatStore.messages.filter(msg => msg.chat_id !== routeChatId);
      if (wrongMessages.length > 0) {
        console.error(`  ❌ ${wrongMessages.length} messages belong to wrong chat!`);
        console.log('     These messages should not be displayed for current chat');
      }
    }

    console.log('\n4️⃣ Root Cause');
    console.log('  The issue is in setCurrentChat method:');
    console.log('  - It has an early return if currentChatId === chatId');
    console.log('  - This prevents fetchMessages from being called');
    console.log('  - Messages from previous chat remain in the store');

    console.log('\n5️⃣ Quick Fix');
    console.log('  Run: window.forceChannelSwitch() to properly load current channel');

  } catch (error) {
    console.error('❌ Diagnostic failed:', error);
  }

  console.groupEnd();
}

// Force proper channel switch
async function forceChannelSwitch(chatId = null) {
  try {
    const { useChatStore } = await import('@/stores/chat');
    const { useRoute } = await import('vue-router');
    const chatStore = useChatStore();
    const route = useRoute();

    const targetChatId = chatId || (route.params.id ? parseInt(route.params.id, 10) : null);

    if (!targetChatId) {
      console.error('❌ No chat ID provided');
      return;
    }

    console.log(`🔄 Force switching to channel ${targetChatId}`);

    // Force reset current chat ID to bypass the check
    chatStore.currentChatId = null;

    // Clear messages
    chatStore.messages.length = 0;

    // Now call setCurrentChat - it won't skip because currentChatId is null
    await chatStore.setCurrentChat(targetChatId);

    console.log('✅ Channel switch complete');
    console.log('  - New currentChatId:', chatStore.currentChatId);
    console.log('  - Messages loaded:', chatStore.messages.length);

    // Verify messages belong to correct chat
    if (chatStore.messages.length > 0) {
      const wrongMessages = chatStore.messages.filter(msg => msg.chat_id !== targetChatId);
      if (wrongMessages.length === 0) {
        console.log('  ✅ All messages belong to correct chat');
      } else {
        console.error(`  ❌ Still have ${wrongMessages.length} messages from wrong chat`);
      }
    }

  } catch (error) {
    console.error('❌ Force switch failed:', error);
  }
}

// Test switching between channels
async function testChannelSwitching() {
  console.group('🧪 Testing Channel Switching');

  try {
    const { useChatStore } = await import('@/stores/chat');
    const chatStore = useChatStore();

    // Get available chats
    const chats = chatStore.chats.slice(0, 3); // Test with first 3 chats

    if (chats.length < 2) {
      console.error('❌ Need at least 2 chats to test switching');
      return;
    }

    console.log(`Testing with ${chats.length} chats:`, chats.map(c => ({ id: c.id, name: c.name })));

    for (let i = 0; i < chats.length; i++) {
      const chat = chats[i];
      console.log(`\n🔄 Switching to chat ${chat.id} (${chat.name})`);

      await chatStore.setCurrentChat(chat.id);

      console.log('  - Current ID:', chatStore.currentChatId);
      console.log('  - Messages:', chatStore.messages.length);

      if (chatStore.messages.length > 0) {
        const wrongMessages = chatStore.messages.filter(msg => msg.chat_id !== chat.id);
        if (wrongMessages.length > 0) {
          console.error(`  ❌ Found ${wrongMessages.length} messages from wrong chat!`);
        } else {
          console.log('  ✅ All messages belong to correct chat');
        }
      }

      // Wait a bit before next switch
      await new Promise(resolve => setTimeout(resolve, 500));
    }

    console.log('\n📊 Test Summary');
    console.log('If you see wrong messages, the issue is confirmed.');
    console.log('Run window.fixChannelSwitching() to apply the fix.');

  } catch (error) {
    console.error('❌ Test failed:', error);
  }

  console.groupEnd();
}

// Export functions
if (typeof window !== 'undefined') {
  window.diagnoseChannelSwitch = diagnoseChannelSwitch;
  window.forceChannelSwitch = forceChannelSwitch;
  window.testChannelSwitching = testChannelSwitching;

  console.log('🔄 Channel switch diagnostic loaded:');
  console.log('  - window.diagnoseChannelSwitch() - Diagnose the issue');
  console.log('  - window.forceChannelSwitch(chatId?) - Force proper channel switch');
  console.log('  - window.testChannelSwitching() - Test switching between channels');
}

export { diagnoseChannelSwitch, forceChannelSwitch, testChannelSwitching }; 