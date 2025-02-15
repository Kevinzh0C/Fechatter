/**
 * Request Isolation Test Tool
 * Verifies that request isolation properly handles extension interference
 */

import requestIsolation from './requestIsolation';
import { useAuthStore } from '@/stores/auth';
import { useChatStore } from '@/stores/chat';

async function testRequestIsolation() {
  console.group('🛡️ Request Isolation Test');

  try {
    // Test 1: Simulate normal request
    console.log('Test 1: Normal request...');
    const result1 = await requestIsolation.executeIsolatedRequest(
      async ({ signal }) => {
        // Simulate API call
        return new Promise((resolve) => {
          setTimeout(() => resolve({ success: true, data: 'test' }), 100);
        });
      }
    );
    console.log('✅ Normal request successful:', result1);

    // Test 2: Simulate extension interference
    console.log('\nTest 2: Extension interference simulation...');
    let retryCount = 0;
    const result2 = await requestIsolation.executeIsolatedRequest(
      async ({ signal }) => {
        retryCount++;
        if (retryCount < 3) {
          // Simulate extension error on first attempts
          throw new Error('A listener chat/2:1 indicated an asynchronous response by returning true, but the message channel closed');
        }
        return { success: true, data: 'recovered after retries', retries: retryCount };
      }
    );
    console.log('✅ Recovered from extension interference:', result2);

    // Test 3: Request queuing
    console.log('\nTest 3: Request queuing test...');
    const promises = [];
    for (let i = 0; i < 3; i++) {
      promises.push(
        requestIsolation.queueRequest('test-queue', async () => {
          console.log(`  Request ${i + 1} executing...`);
          await new Promise(resolve => setTimeout(resolve, 500));
          return { request: i + 1 };
        })
      );
    }
    const results = await Promise.all(promises);
    console.log('✅ Queue test results:', results);

    // Test 4: Real chat message loading with isolation
    console.log('\nTest 4: Real message loading test...');
    const chatStore = useChatStore();
    const authStore = useAuthStore();

    if (authStore.isAuthenticated && chatStore.currentChatId) {
      console.log('  Testing message fetch for chat:', chatStore.currentChatId);
      await chatStore.fetchMessages(chatStore.currentChatId);
      console.log('✅ Messages loaded:', chatStore.messages.length);
    } else {
      console.log('⚠️ Not authenticated or no current chat');
    }

    // Test 5: Active request monitoring
    console.log('\nTest 5: Active request monitoring...');
    console.log('Active requests:', requestIsolation.getActiveRequestCount());

  } catch (error) {
    console.error('❌ Test failed:', error);
  } finally {
    console.groupEnd();
  }
}

// Additional test for extension conflict patterns
function testExtensionPatterns() {
  console.group('🔍 Extension Pattern Test');

  const testErrors = [
    'A listener chat/2:1 indicated an asynchronous response by returning true',
    'message channel closed before a response was received',
    'Failed to fetch at chrome-extension://abcd/content.js',
    'Extension context invalidated',
    'Normal error without extension'
  ];

  testErrors.forEach((error, index) => {
    const isExtension = requestIsolation.isExtensionInterference(new Error(error));
    console.log(`Test ${index + 1}: "${error.substring(0, 50)}..." - Extension: ${isExtension ? '✅' : '❌'}`);
  });

  console.groupEnd();
}

// Export test functions
window.testRequestIsolation = testRequestIsolation;
window.testExtensionPatterns = testExtensionPatterns;

// Auto-run pattern test
console.log('💡 Running extension pattern test...');
testExtensionPatterns(); 