/**
 * Network Message Diagnostic Tool
 * Professional-grade diagnostic for message loading issues
 * Covers: Browser Network Layer → Vue Reactivity → SSE
 */

export async function diagnoseMessageLoading() {
  console.log('🔬 Professional Message Loading Diagnostic');
  console.log('='.repeat(50));

  const diagnostics = {
    networkLayer: {},
    vueReactivity: {},
    sseConnection: {},
    recommendations: []
  };

  try {
    // Import required modules
    const { useChatStore } = await import('@/stores/chat');
    const { default: api } = await import('@/services/api');
    const chatStore = useChatStore();

    // ========================================
    // Layer 1: Browser Network Layer
    // ========================================
    console.log('\n📡 Layer 1: Browser Network Layer\n');

    // 1.1 Check API connectivity
    console.log('1.1 Testing API connectivity...');
    try {
      const startTime = performance.now();
      const response = await api.get('/workspace');
      const latency = performance.now() - startTime;

      diagnostics.networkLayer.apiConnectivity = {
        status: 'OK',
        latency: `${latency.toFixed(2)}ms`,
        statusCode: response.status
      };
      console.log(`✅ API reachable (${latency.toFixed(2)}ms)`);
    } catch (error) {
      diagnostics.networkLayer.apiConnectivity = {
        status: 'FAILED',
        error: error.message,
        statusCode: error.response?.status
      };
      console.error('❌ API unreachable:', error.message);
      diagnostics.recommendations.push('Check network connection and API server status');
    }

    // 1.2 Check current chat messages endpoint
    if (chatStore.currentChatId) {
      console.log('\n1.2 Testing message fetch endpoint...');
      try {
        const startTime = performance.now();
        const response = await api.get(`/chat/${chatStore.currentChatId}/messages`);
        const latency = performance.now() - startTime;

        diagnostics.networkLayer.messageFetch = {
          status: 'OK',
          latency: `${latency.toFixed(2)}ms`,
          messageCount: response.data?.data?.length || response.data?.length || 0,
          responseSize: JSON.stringify(response.data).length
        };
        console.log(`✅ Messages fetched: ${diagnostics.networkLayer.messageFetch.messageCount} messages in ${latency.toFixed(2)}ms`);
      } catch (error) {
        diagnostics.networkLayer.messageFetch = {
          status: 'FAILED',
          error: error.message,
          statusCode: error.response?.status
        };
        console.error('❌ Message fetch failed:', error.message);
        diagnostics.recommendations.push('Check message API endpoint permissions');
      }
    }

    // 1.3 Check request headers
    console.log('\n1.3 Checking request configuration...');
    diagnostics.networkLayer.requestConfig = {
      baseURL: api.defaults.baseURL,
      hasAuthHeader: !!api.defaults.headers.common['Authorization'],
      authHeaderPreview: api.defaults.headers.common['Authorization'] ?
        api.defaults.headers.common['Authorization'].substring(0, 20) + '...' : 'None',
      timeout: api.defaults.timeout
    };
    console.log('Request config:', diagnostics.networkLayer.requestConfig);

    // ========================================
    // Layer 2: Vue Reactivity System
    // ========================================
    console.log('\n⚡ Layer 2: Vue Reactivity System\n');

    // 2.1 Check store state
    console.log('2.1 Checking chat store state...');
    diagnostics.vueReactivity.storeState = {
      currentChatId: chatStore.currentChatId,
      messagesCount: chatStore.messages?.length || 0,
      loading: chatStore.loading,
      hasMessageCache: Object.keys(chatStore.messageCache || {}).length > 0,
      cacheSize: Object.keys(chatStore.messageCache || {}).length,
      isMessagesReactive: Array.isArray(chatStore.messages)
    };
    console.log('Store state:', diagnostics.vueReactivity.storeState);

    // 2.2 Test reactivity
    console.log('\n2.2 Testing Vue reactivity...');
    const originalCount = chatStore.messages.length;
    const testMessage = {
      id: `test_${Date.now()}`,
      content: 'Reactivity test message',
      sender_id: 'test',
      created_at: new Date().toISOString(),
      chat_id: chatStore.currentChatId
    };

    // Test push reactivity
    chatStore.messages.push(testMessage);
    const afterPushCount = chatStore.messages.length;
    const pushWorked = afterPushCount === originalCount + 1;

    // Clean up test message
    chatStore.messages.pop();

    diagnostics.vueReactivity.pushTest = {
      success: pushWorked,
      beforeCount: originalCount,
      afterCount: afterPushCount
    };
    console.log(`${pushWorked ? '✅' : '❌'} Array push reactivity: ${pushWorked ? 'Working' : 'Failed'}`);

    // 2.3 Check message normalization
    console.log('\n2.3 Checking message processing...');
    if (chatStore.messages.length > 0) {
      const sampleMessage = chatStore.messages[0];
      diagnostics.vueReactivity.messageStructure = {
        hasId: !!sampleMessage.id,
        hasContent: !!sampleMessage.content,
        hasSenderId: !!sampleMessage.sender_id,
        hasCreatedAt: !!sampleMessage.created_at,
        hasTimestamp: !!sampleMessage._timestamp,
        hasSenderName: !!sampleMessage.sender_name
      };
      console.log('Message structure:', diagnostics.vueReactivity.messageStructure);
    }

    // ========================================
    // Layer 3: SSE (Server-Sent Events)
    // ========================================
    console.log('\n🌐 Layer 3: SSE Connection\n');

    // 3.1 Check SSE service
    console.log('3.1 Checking SSE service...');
    try {
      // Import SSE services
      const sseModules = {
        minimal: null,
        standard: null,
        enhanced: null
      };

      try {
        sseModules.minimal = await import('@/services/sse-minimal');
        console.log('✅ SSE Minimal loaded');
      } catch (e) {
        console.log('⚠️ SSE Minimal not found');
      }

      try {
        sseModules.standard = await import('@/services/sse');
        console.log('✅ SSE Standard loaded');
      } catch (e) {
        console.log('⚠️ SSE Standard not found');
      }

      try {
        sseModules.enhanced = await import('@/services/sse-enhanced');
        console.log('✅ SSE Enhanced loaded');
      } catch (e) {
        console.log('⚠️ SSE Enhanced not found');
      }

      // Check which SSE is active
      const activeSSE = sseModules.minimal?.default || sseModules.standard?.default || sseModules.enhanced?.default;

      if (activeSSE) {
        diagnostics.sseConnection.status = {
          connected: activeSSE.connected || false,
          readyState: activeSSE.eventSource?.readyState,
          url: activeSSE.eventSource?.url || activeSSE.sseUrl,
          reconnectAttempts: activeSSE.reconnectAttempts || 0
        };
        console.log('SSE status:', diagnostics.sseConnection.status);

        if (!diagnostics.sseConnection.status.connected) {
          diagnostics.recommendations.push('SSE not connected - real-time messages won\'t work');
        }
      } else {
        diagnostics.sseConnection.status = { error: 'No SSE service found' };
        diagnostics.recommendations.push('SSE service not initialized');
      }
    } catch (error) {
      diagnostics.sseConnection.status = { error: error.message };
      console.error('❌ SSE check failed:', error);
    }

    // 3.2 Check message flow
    console.log('\n3.2 Checking message flow configuration...');
    diagnostics.sseConnection.messageFlow = {
      hasAddRealtimeMessage: typeof chatStore.addRealtimeMessage === 'function',
      hasHandleIncomingMessage: typeof chatStore.handleIncomingMessage === 'function',
      hasPendingOptimisticMessages: chatStore.pendingOptimisticMessages?.size || 0,
      messageBatcherActive: !!chatStore._batchWatcher
    };
    console.log('Message flow:', diagnostics.sseConnection.messageFlow);

    // ========================================
    // Root Cause Analysis
    // ========================================
    console.log('\n🔍 Root Cause Analysis\n');

    // Analyze findings
    if (diagnostics.networkLayer.apiConnectivity?.status === 'FAILED') {
      diagnostics.recommendations.push('Primary issue: Cannot reach API server');
    }

    if (diagnostics.networkLayer.messageFetch?.status === 'FAILED') {
      diagnostics.recommendations.push('Cannot fetch messages from API - check authentication');
    }

    if (!diagnostics.vueReactivity.storeState.isMessagesReactive) {
      diagnostics.recommendations.push('Messages array is not reactive - Vue won\'t update UI');
    }

    if (diagnostics.vueReactivity.storeState.messagesCount === 0 &&
      diagnostics.networkLayer.messageFetch?.messageCount > 0) {
      diagnostics.recommendations.push('Messages fetched but not stored - check fetchMessages implementation');
    }

    if (!diagnostics.sseConnection.status?.connected) {
      diagnostics.recommendations.push('SSE disconnected - new messages won\'t appear in real-time');
    }

    // ========================================
    // Recommendations
    // ========================================
    console.log('\n💡 Recommendations:\n');
    if (diagnostics.recommendations.length === 0) {
      diagnostics.recommendations.push('All systems appear functional - check browser console for errors');
    }

    diagnostics.recommendations.forEach((rec, index) => {
      console.log(`${index + 1}. ${rec}`);
    });

    // ========================================
    // Quick Fix Attempts
    // ========================================
    console.log('\n🔧 Attempting quick fixes...\n');

    // Fix 1: Force refresh messages
    if (chatStore.currentChatId) {
      console.log('Attempting force refresh...');
      try {
        await chatStore.fetchMessages(chatStore.currentChatId);
        console.log('✅ Force refresh completed');
      } catch (error) {
        console.error('❌ Force refresh failed:', error.message);
      }
    }

    // Fix 2: Clear cache
    console.log('\nClearing message cache...');
    chatStore.messageCache = {};
    sessionStorage.removeItem('messageCache');
    console.log('✅ Cache cleared');

    return diagnostics;

  } catch (error) {
    console.error('❌ Diagnostic failed:', error);
    return { error: error.message };
  }
}

// Auto-register in development
if (import.meta.env.DEV) {
  window.diagnoseMessages = diagnoseMessageLoading;
  console.log('🔬 Network message diagnostic loaded - use window.diagnoseMessages()');
} 