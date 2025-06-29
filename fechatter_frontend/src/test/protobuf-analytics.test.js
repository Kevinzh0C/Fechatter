/**
 * Test protobuf analytics implementation
 */

import { completeAnalytics } from '../lib/analytics-protobuf-complete.js';

// Test configuration
const testConfig = {
  enabled: true,
  debug: true,
  endpoint: 'http://127.0.0.1:6690',
  fallback_to_json: true,
};

// Create test client
const testAnalytics = new (await import('../lib/analytics-protobuf-complete.js')).CompleteProtobufAnalyticsClient(testConfig);

async function runProtobufTests() {
  console.log('🧪 Running Protobuf Analytics Tests...');

  // Test 1: Check client status
  console.log('\nClient Status:');
  const status = testAnalytics.getStatus();
  console.log(JSON.stringify(status, null, 2));

  // Test 2: Test app start event
  console.log('\nTesting App Start Event...');
  try {
    await testAnalytics.trackAppStart();
    console.log('App start event tracked successfully');
  } catch (error) {
    console.error('ERROR: App start event failed:', error);
  }

  // Test 3: Test user login event
  console.log('\nUSER: Testing User Login Event...');
  try {
    await testAnalytics.trackUserLogin('test@example.com', 'password');
    console.log('User login event tracked successfully');
  } catch (error) {
    console.error('ERROR: User login event failed:', error);
  }

  // Test 4: Test message sent event
  console.log('\nMESSAGE: Testing Message Sent Event...');
  try {
    await testAnalytics.trackMessageSent('chat_123', 'Hello world! @user', []);
    console.log('Message sent event tracked successfully');
  } catch (error) {
    console.error('ERROR: Message sent event failed:', error);
  }

  // Test 5: Test navigation event
  console.log('\n🧭 Testing Navigation Event...');
  try {
    const startTime = Date.now() - 1000; // Simulate 1 second navigation
    await testAnalytics.trackNavigation('/login', '/chat', startTime);
    console.log('Navigation event tracked successfully');
  } catch (error) {
    console.error('ERROR: Navigation event failed:', error);
  }

  // Test 6: Test error event
  console.log('\nWARNING: Testing Error Event...');
  try {
    const testError = new Error('Test error message');
    testError.stack = 'Error: Test error\n    at test:1:1';
    await testAnalytics.trackError(testError, 'test-context', 'TestError');
    console.log('Error event tracked successfully');
  } catch (error) {
    console.error('ERROR: Error event failed:', error);
  }

  // Test 7: Test search event
  console.log('\nTesting Search Event...');
  try {
    await testAnalytics.trackSearch('global', 'test query', 5, 250, true);
    console.log('Search event tracked successfully');
  } catch (error) {
    console.error('ERROR: Search event failed:', error);
  }

  // Test 8: Test file upload event
  console.log('\n📁 Testing File Upload Event...');
  try {
    // Create a mock file object
    const mockFile = {
      type: 'image/jpeg',
      size: 1024 * 1024, // 1MB
      name: 'test.jpg'
    };
    await testAnalytics.trackFileUpload(mockFile, 'drag-drop', 2500);
    console.log('File upload event tracked successfully');
  } catch (error) {
    console.error('ERROR: File upload event failed:', error);
  }

  // Test 9: Flush pending events
  console.log('\n🔄 Testing Batch Flush...');
  try {
    await testAnalytics.flush();
    console.log('Batch flush completed successfully');
  } catch (error) {
    console.error('ERROR: Batch flush failed:', error);
  }

  // Test 10: Final status check
  console.log('\nFinal Client Status:');
  const finalStatus = testAnalytics.getStatus();
  console.log(JSON.stringify(finalStatus, null, 2));

  console.log('\n🎉 Protobuf Analytics Tests Completed!');

  // Clean up
  testAnalytics.destroy();
}

// Run tests if this file is executed directly
if (typeof window !== 'undefined') {
  // Browser environment
  window.runProtobufTests = runProtobufTests;
  console.log('Protobuf tests loaded. Run window.runProtobufTests() to execute.');
} else {
  // Node.js environment
  runProtobufTests().catch(console.error);
} 