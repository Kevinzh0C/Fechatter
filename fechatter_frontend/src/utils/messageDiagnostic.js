/**
 * Message Diagnostic Tool
 * Comprehensive diagnostic for message loading issues
 * Production-grade implementation following Occam's Razor principle
 */

import { useChatStore } from '@/stores/chat';
import { useAuthStore } from '@/stores/auth';
import api from '@/services/api';

async function diagnoseMessages() {
  console.group('🔍 Message Loading Diagnostic');

  const chatStore = useChatStore();
  const authStore = useAuthStore();

  // Check authentication
  console.log('1️⃣ Authentication Status:');
  console.log('  - Authenticated:', authStore.isAuthenticated);
  console.log('  - User:', authStore.user?.email || 'No user');
  console.log('  - Token exists:', !!authStore.token);

  // Check current chat
  console.log('\n2️⃣ Current Chat Status:');
  console.log('  - Current chat ID:', chatStore.currentChatId);
  console.log('  - Current chat:', chatStore.getCurrentChat);
  console.log('  - Messages count:', chatStore.messages.length);
  console.log('  - Loading state:', chatStore.loading);

  // Check message cache
  console.log('\n3️⃣ Message Cache:');
  const cacheKeys = Object.keys(chatStore.messageCache);
  console.log('  - Cached chats:', cacheKeys.length);
  if (chatStore.currentChatId && chatStore.messageCache[chatStore.currentChatId]) {
    const cache = chatStore.messageCache[chatStore.currentChatId];
    console.log('  - Current chat cached:', true);
    console.log('  - Cache timestamp:', new Date(cache.timestamp).toLocaleString());
    console.log('  - Cached messages:', cache.messages?.length || 0);
  } else {
    console.log('  - Current chat cached:', false);
  }

  // Check extension conflicts
  console.log('\n4️⃣ Extension Conflict Check:');
  if (window.extensionConflictHandler) {
    const report = window.extensionConflictHandler.showConflictReport();
    console.log('  - Detected extensions:', report.detectedExtensions);
    console.log('  - Error counts:', report.errorCounts);
  } else {
    console.log('  - Extension handler not loaded');
  }

  // Test message loading
  if (chatStore.currentChatId) {
    console.log('\n5️⃣ Testing Message Load:');
    try {
      const startTime = performance.now();

      // Direct API test
      console.log('  - Testing direct API call...');
      const response = await api.get(`/chat/${chatStore.currentChatId}/messages`, {
        params: { limit: 10 }
      });

      const apiTime = performance.now() - startTime;
      console.log(`  ✅ API call successful (${apiTime.toFixed(2)}ms)`);
      console.log('  - Response status:', response.status);
      console.log('  - Messages returned:', response.data?.data?.length || 0);

      // Store method test
      console.log('\n  - Testing store fetchMessages...');
      const storeStartTime = performance.now();
      await chatStore.fetchMessages(chatStore.currentChatId, 10);
      const storeTime = performance.now() - storeStartTime;

      console.log(`  ✅ Store fetch successful (${storeTime.toFixed(2)}ms)`);
      console.log('  - Messages in store:', chatStore.messages.length);

    } catch (error) {
      console.error('  ❌ Message loading failed:', error);
      console.log('  - Error type:', error.constructor.name);
      console.log('  - Error message:', error.message);
      console.log('  - Response status:', error.response?.status);
      console.log('  - Response data:', error.response?.data);
    }
  } else {
    console.log('\n5️⃣ No current chat selected for testing');
  }

  // Check request isolation
  console.log('\n6️⃣ Request Isolation Status:');
  if (window.requestIsolation) {
    console.log('  - Active requests:', window.requestIsolation.getActiveRequestCount());
    console.log('  - Request isolation available:', true);
  } else {
    console.log('  - Request isolation not loaded');
  }

  // Provide recommendations
  console.log('\n📋 Recommendations:');

  if (!authStore.isAuthenticated) {
    console.log('  ⚠️ User not authenticated - login required');
  }

  if (!chatStore.currentChatId) {
    console.log('  ⚠️ No chat selected - select a chat first');
  }

  if (chatStore.messages.length === 0 && chatStore.currentChatId) {
    console.log('  ⚠️ No messages loaded - possible issues:');
    console.log('     - Extension interference (try incognito mode)');
    console.log('     - Network connectivity issues');
    console.log('     - API server not responding');
    console.log('     - Authentication token expired');
  }

  if (window.extensionConflictHandler?.detectedConflicts?.size > 0) {
    console.log('  ⚠️ Browser extensions detected - may cause issues');
  }

  console.groupEnd();

  return {
    authenticated: authStore.isAuthenticated,
    currentChatId: chatStore.currentChatId,
    messageCount: chatStore.messages.length,
    cacheStatus: cacheKeys.length > 0,
    extensionConflicts: window.extensionConflictHandler?.detectedConflicts?.size || 0
  };
}

// Quick message refresh function
async function refreshMessages(chatId = null) {
  const chatStore = useChatStore();
  const targetChatId = chatId || chatStore.currentChatId;

  if (!targetChatId) {
    console.error('❌ No chat ID provided or selected');
    return;
  }

  console.log(`🔄 Refreshing messages for chat ${targetChatId}...`);

  try {
    // Clear cache for this chat
    if (chatStore.messageCache[targetChatId]) {
      delete chatStore.messageCache[targetChatId];
      sessionStorage.setItem('messageCache', JSON.stringify(chatStore.messageCache));
    }

    // Fetch fresh messages
    await chatStore.fetchMessages(targetChatId);
    console.log(`✅ Messages refreshed: ${chatStore.messages.length} messages loaded`);

  } catch (error) {
    console.error('❌ Failed to refresh messages:', error);
  }
}

// Clear all message caches
function clearMessageCache() {
  const chatStore = useChatStore();

  console.log('🧹 Clearing all message caches...');

  // Clear store cache
  chatStore.messageCache = {};

  // Clear session storage
  sessionStorage.removeItem('messageCache');

  // Clear current messages
  chatStore.messages.length = 0;

  console.log('✅ Message cache cleared');
}

// Export functions
window.diagnoseMessages = diagnoseMessages;
window.refreshMessages = refreshMessages;
window.clearMessageCache = clearMessageCache;

// Auto-run basic diagnostic
console.log('💡 Message diagnostic loaded. Available commands:');
console.log('  - window.diagnoseMessages() - Full diagnostic');
console.log('  - window.refreshMessages(chatId?) - Refresh messages');
console.log('  - window.clearMessageCache() - Clear all caches'); 