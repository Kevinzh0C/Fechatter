/**
 * Message Debugger - Track message flow from frontend to backend
 * 
 * Usage: 
 * - Import and call debugMessageFlow() in console
 * - Or use window.debugMessage() after a message is sent
 */

import { useChatStore } from '@/stores/chat';
import { useAuthStore } from '@/stores/auth';

export class MessageDebugger {
  constructor() {
    this.logs = [];
    this.startTime = Date.now();
  }

  log(stage, data) {
    const timestamp = Date.now() - this.startTime;
    const entry = {
      timestamp,
      stage,
      data,
      time: new Date().toISOString()
    };
    this.logs.push(entry);
    console.log(`[${timestamp}ms] ${stage}:`, data);
  }

  async debugLastMessage() {
    const chatStore = useChatStore();
    const authStore = useAuthStore();

    console.group('🔍 Message Flow Debug Report');

    // 1. Check optimistic messages
    console.group('📝 Optimistic Messages');
    console.log('Pending count:', chatStore.pendingOptimisticMessages.size);
    if (chatStore.pendingOptimisticMessages.size > 0) {
      chatStore.pendingOptimisticMessages.forEach((msg, tempId) => {
        console.log(`- ${tempId}:`, {
          chatId: msg.chatId,
          content: msg.content.substring(0, 50) + '...',
          timestamp: new Date(msg.timestamp).toLocaleTimeString(),
          realId: msg.realId || 'Not yet assigned'
        });
      });
    }
    console.groupEnd();

    // 2. Check current chat messages
    console.group('💬 Current Chat Messages');
    const currentChatId = chatStore.currentChatId;
    console.log('Current chat ID:', currentChatId);

    if (currentChatId && chatStore.messages.length > 0) {
      // Show last 5 messages
      const recentMessages = chatStore.messages.slice(-5);
      console.table(recentMessages.map(msg => ({
        id: msg.id,
        tempId: msg.temp_id,
        content: msg.content.substring(0, 50) + '...',
        status: msg.status || 'sent',
        sender: msg.sender?.fullname || msg.sender_id,
        time: new Date(msg.created_at).toLocaleTimeString()
      })));
    }
    console.groupEnd();

    // 3. Check SSE connection
    console.group('🔌 SSE Connection Status');
    try {
      const sseService = await import('@/services/sse');
      const connectionState = sseService.default.getConnectionState();
      console.log('Connected:', connectionState.isConnected);
      console.log('Latency:', connectionState.latency);
      console.log('Reconnect attempts:', connectionState.reconnectAttempts);
    } catch (e) {
      console.error('SSE service not available:', e.message);
    }
    console.groupEnd();

    // 4. Check API errors
    console.group('❌ Recent API Errors');
    // Check localStorage for recent errors
    const recentErrors = JSON.parse(localStorage.getItem('api_errors') || '[]');
    if (recentErrors.length > 0) {
      console.table(recentErrors.slice(-5));
    } else {
      console.log('No recent API errors');
    }
    console.groupEnd();

    // 5. Network requests
    console.group('🌐 Recent Network Requests');
    console.log('Check Network tab for:');
    console.log('- POST /api/chat/{id}/messages (message send)');
    console.log('- GET /api/events (SSE connection)');
    console.log('- Any 4xx or 5xx errors');
    console.groupEnd();

    console.groupEnd();

    return this.generateReport();
  }

  generateReport() {
    const chatStore = useChatStore();

    return {
      summary: {
        pendingMessages: chatStore.pendingOptimisticMessages.size,
        currentChatMessages: chatStore.messages.length,
        hasUnsentMessages: Array.from(chatStore.messages).some(m => m.status === 'sending' || m.status === 'failed')
      },
      recommendations: this.getRecommendations()
    };
  }

  getRecommendations() {
    const chatStore = useChatStore();
    const recommendations = [];

    // Check for stuck messages
    if (chatStore.pendingOptimisticMessages.size > 0) {
      const now = Date.now();
      chatStore.pendingOptimisticMessages.forEach((msg, tempId) => {
        if (now - msg.timestamp > 30000) { // 30 seconds
          recommendations.push(`Message ${tempId} has been pending for over 30 seconds - may be stuck`);
        }
      });
    }

    // Check for failed messages
    const failedMessages = chatStore.messages.filter(m => m.status === 'failed');
    if (failedMessages.length > 0) {
      recommendations.push(`Found ${failedMessages.length} failed messages - check network/API errors`);
    }

    return recommendations;
  }
}

// Global debug function
export async function debugMessageFlow() {
  const messageDebugger = new MessageDebugger();
  return await messageDebugger.debugLastMessage();
}

// Attach to window for easy console access
if (typeof window !== 'undefined') {
  window.debugMessage = debugMessageFlow;
  window.MessageDebugger = MessageDebugger;

  console.log('💡 Message debugger loaded. Use window.debugMessage() to debug message flow');
} 