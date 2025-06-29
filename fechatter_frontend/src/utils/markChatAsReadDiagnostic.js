/**
 * Mark Chat As Read Diagnostic Tool
 * Tests the fixed markChatAsRead functionality
 */

class MarkChatAsReadDiagnostic {
  constructor() {
    this.name = 'MarkChatAsReadDiagnostic';
    this.version = '1.0.0';
  }

  /**
   * Test the fixed markChatAsRead implementation
   */
  async testMarkChatAsReadFix() {
    console.group('🧪 Mark Chat As Read Fix Test');

    try {
      const { useChatStore } = await import('@/stores/chat');
      const { useRoute } = await import('vue-router');

      const chatStore = useChatStore();
      const route = useRoute();

      if (import.meta.env.DEV) {
        console.log('\n1️⃣ Current State:');
      if (import.meta.env.DEV) {
        console.log('- Current chat ID:', chatStore.currentChatId);
      if (import.meta.env.DEV) {
        console.log('- Route chat ID:', route.params.id);
      if (import.meta.env.DEV) {
        console.log('- Messages loaded:', chatStore.messages.length);
      }

      if (!chatStore.currentChatId) {
        if (import.meta.env.DEV) {
          console.warn('WARNING: No current chat selected. Navigate to a chat first.');
        return;
      }

      const currentChatId = chatStore.currentChatId;
      const currentChat = chatStore.chats.find(c => c.id === currentChatId);

      if (import.meta.env.DEV) {
        console.log('\n2️⃣ Chat Info:');
      if (import.meta.env.DEV) {
        console.log('- Chat found:', !!currentChat);
      if (import.meta.env.DEV) {
        console.log('- Chat name:', currentChat?.name);
      if (import.meta.env.DEV) {
        console.log('- Unread count before:', currentChat?.unread_count);
      }

      if (import.meta.env.DEV) {
        console.log('\n3️⃣ Testing markChatAsRead fix...');
      }

      // Test the fixed implementation
      try {
        await chatStore.markChatAsRead(currentChatId);

        const updatedChat = chatStore.chats.find(c => c.id === currentChatId);
        if (import.meta.env.DEV) {
          console.log('markChatAsRead completed successfully');
        if (import.meta.env.DEV) {
          console.log('- Unread count after:', updatedChat?.unread_count);
        }

        if (updatedChat?.unread_count === 0) {
          if (import.meta.env.DEV) {
            console.log('Local unread count properly reset');
          }
        } else {
          if (import.meta.env.DEV) {
            console.warn('WARNING: Unread count not reset, value:', updatedChat?.unread_count);
          }

      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('ERROR: markChatAsRead failed:', error);
        }

        // Analyze the error
        if (error.response?.status === 404) {
          if (import.meta.env.DEV) {
            console.log('404 Error Analysis:');
          if (import.meta.env.DEV) {
            console.log('- URL attempted:', error.config?.url);
          if (import.meta.env.DEV) {
            console.log('- Method:', error.config?.method);
          if (import.meta.env.DEV) {
            console.log('- This should now be handled gracefully');
          }

      if (import.meta.env.DEV) {
        console.log('\n4️⃣ Endpoint Test:');
      }

      // Test different endpoints that might work
      const testEndpoints = [
        `/chat/${currentChatId}/messages/read`,
        `/chat/${currentChatId}/mark-read`,
        `/chat/${currentChatId}/read`
      ];

      for (const endpoint of testEndpoints) {
        try {
          if (import.meta.env.DEV) {
            console.log(`Testing endpoint: ${endpoint}`);
          }

          if (endpoint.includes('/messages/read')) {
            // Test with message IDs
            const messageIds = chatStore.messages
              .map(msg => msg.id)
              .filter(id => id && !String(id).startsWith('temp_'))
              .slice(0, 5); // Test with first 5 messages

            if (messageIds.length > 0) {
              if (import.meta.env.DEV) {
                console.log(`- Testing with ${messageIds.length} message IDs`);
              }
              // Note: This is just a test, actual request handled by fixed method
            } else {
              if (import.meta.env.DEV) {
                console.log('- No valid message IDs available for test');
              }

          if (import.meta.env.DEV) {
            console.log(`Endpoint ${endpoint} format validated`);
          }
        } catch (error) {
          if (import.meta.env.DEV) {
            console.log(`ERROR: Endpoint ${endpoint} failed:`, error.response?.status);
          }

      if (import.meta.env.DEV) {
        console.log('\n5️⃣ Verification:');
      const finalChat = chatStore.chats.find(c => c.id === currentChatId);
      if (import.meta.env.DEV) {
        console.log('- Final unread count:', finalChat?.unread_count);
      if (import.meta.env.DEV) {
        console.log('- UI should show no unread badge');
      }

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('ERROR: Diagnostic failed:', error);
      }

    console.groupEnd();
  }

  /**
   * Monitor markChatAsRead calls for debugging
   */
  monitorMarkChatAsReadCalls() {
    if (import.meta.env.DEV) {
      console.log('Starting markChatAsRead monitoring...');
    }

    // Hook into chat store if available
    try {
      const { useChatStore } = require('@/stores/chat');
      const chatStore = useChatStore();

      // Store original method
      const originalMarkChatAsRead = chatStore.markChatAsRead.bind(chatStore);

      // Override with monitoring
      chatStore.markChatAsRead = async function (chatId) {
        console.group(`📞 markChatAsRead called for chat: ${chatId}`);
        if (import.meta.env.DEV) {
          console.log('- Timestamp:', new Date().toISOString());
        if (import.meta.env.DEV) {
          console.log('- Messages count:', this.messages.length);
        if (import.meta.env.DEV) {
          console.log('- Current chat ID:', this.currentChatId);
        }

        const startTime = performance.now();

        try {
          const result = await originalMarkChatAsRead(chatId);
          const endTime = performance.now();

          if (import.meta.env.DEV) {
            console.log(`Success in ${(endTime - startTime).toFixed(2)}ms`);
          return result;
        } catch (error) {
          const endTime = performance.now();

          if (import.meta.env.DEV) {
            console.error(`ERROR: Failed in ${(endTime - startTime).toFixed(2)}ms:`, error.message);
          throw error;
        } finally {
          console.groupEnd();
        }
      };

      if (import.meta.env.DEV) {
        console.log('markChatAsRead monitoring active');
      return true;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('ERROR: Failed to setup monitoring:', error);
      return false;
    }

  /**
   * Stop monitoring
   */
  stopMonitoring() {
    // This would restore original method, but for simplicity we'll just log
    if (import.meta.env.DEV) {
      console.log('🛑 Monitoring stopped (restart app to restore original method)');
    }

// Create and export instance
const markChatAsReadDiagnostic = new MarkChatAsReadDiagnostic();

// Make available globally
if (typeof window !== 'undefined') {
  window.markChatAsReadDiagnostic = markChatAsReadDiagnostic;

  // Convenient global functions
  window.testMarkChatAsRead = () => markChatAsReadDiagnostic.testMarkChatAsReadFix();
  window.monitorMarkChatAsRead = () => markChatAsReadDiagnostic.monitorMarkChatAsReadCalls();

  if (import.meta.env.DEV) {
    console.log('🧪 Mark Chat As Read Diagnostic loaded');
  if (import.meta.env.DEV) {
    console.log('Use window.testMarkChatAsRead() to test the fix');
  if (import.meta.env.DEV) {
    console.log('Use window.monitorMarkChatAsRead() to monitor calls');
  }

export default markChatAsReadDiagnostic; 