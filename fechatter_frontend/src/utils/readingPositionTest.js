/**
 * Reading Position Test Suite
 * 测试阅读位置管理功能
 */

import { readingPositionManager } from './readingPositionManager';

export function testReadingPositionManager() {
  console.log('🧪 Testing Reading Position Manager');
  console.log('=====================================');
  
  const testChatId = 123;
  
  // Test 1: Check initial state
  console.log('\n📊 Test 1: Initial State');
  const isVisited = readingPositionManager.isChannelVisitedInSession(testChatId);
  console.log('- Channel visited in session:', isVisited);
  
  const strategy = readingPositionManager.getLoadingStrategy(testChatId);
  console.log('- Loading strategy:', strategy);
  
  // Test 2: Mark as visited
  console.log('\n📊 Test 2: Mark as Visited');
  readingPositionManager.markChannelAsVisited(testChatId);
  
  const isVisitedAfter = readingPositionManager.isChannelVisitedInSession(testChatId);
  console.log('- Channel visited after marking:', isVisitedAfter);
  
  const strategyAfter = readingPositionManager.getLoadingStrategy(testChatId);
  console.log('- Loading strategy after visit:', strategyAfter);
  
  // Test 3: Save reading position
  console.log('\n📊 Test 3: Save Reading Position');
  const testPosition = {
    messageId: 456,
    scrollOffset: 100,
    totalMessages: 50
  };
  
  readingPositionManager.saveReadingPosition(testChatId, testPosition);
  
  const savedPosition = readingPositionManager.getReadingPosition(testChatId);
  console.log('- Saved position:', savedPosition);
  
  // Test 4: Strategy with saved position
  console.log('\n📊 Test 4: Strategy with Saved Position');
  const strategyWithPosition = readingPositionManager.getLoadingStrategy(testChatId);
  console.log('- Strategy with saved position:', strategyWithPosition);
  
  // Test 5: Debug info
  console.log('\n📊 Test 5: Debug Information');
  const debugInfo = readingPositionManager.getDebugInfo();
  console.log('- Debug info:', debugInfo);
  
  console.log('\n✅ Reading Position Manager test completed');
  return debugInfo;
}

export function simulateChannelNavigation() {
  console.log('🧪 Simulating Channel Navigation');
  console.log('=================================');
  
  const channels = [101, 102, 103, 104, 105];
  
  channels.forEach((channelId, index) => {
    console.log(`\n🔄 Navigating to channel ${channelId}`);
    
    // Get strategy before visit
    const beforeStrategy = readingPositionManager.getLoadingStrategy(channelId);
    console.log('- Strategy before visit:', beforeStrategy.type, '-', beforeStrategy.reason);
    
    // Mark as visited
    readingPositionManager.markChannelAsVisited(channelId);
    
    // Simulate saving position (except for last one)
    if (index < channels.length - 1) {
      const position = {
        messageId: channelId * 10 + Math.floor(Math.random() * 50),
        scrollOffset: Math.floor(Math.random() * 200),
        totalMessages: Math.floor(Math.random() * 100) + 20
      };
      
      readingPositionManager.saveReadingPosition(channelId, position);
      console.log('- Saved position for message:', position.messageId);
    }
    
    // Get strategy after visit
    const afterStrategy = readingPositionManager.getLoadingStrategy(channelId);
    console.log('- Strategy after visit:', afterStrategy.type, '-', afterStrategy.reason);
  });
  
  console.log('\n📊 Final State:');
  console.log('- Session channels:', [...readingPositionManager.sessionChannels]);
  console.log('- Reading positions:', Object.keys(readingPositionManager.getReadingPositions()));
  
  console.log('\n✅ Channel navigation simulation completed');
}

export function testScrollPositionSaving() {
  console.log('🧪 Testing Scroll Position Saving');
  console.log('==================================');
  
  // Mock DOM elements for testing
  const mockMessages = [
    { id: 1, content: 'Message 1' },
    { id: 2, content: 'Message 2' },
    { id: 3, content: 'Message 3' },
    { id: 4, content: 'Message 4' },
    { id: 5, content: 'Message 5' }
  ];
  
  console.log('📝 Mock messages created:', mockMessages.length);
  
  // Simulate saving positions for different messages
  mockMessages.forEach(message => {
    const position = {
      messageId: message.id,
      scrollOffset: Math.floor(Math.random() * 100),
      totalMessages: mockMessages.length
    };
    
    readingPositionManager.saveReadingPosition(999, position);
    console.log(`- Saved position for message ${message.id}`);
  });
  
  const finalPosition = readingPositionManager.getReadingPosition(999);
  console.log('📍 Final saved position:', finalPosition);
  
  console.log('\n✅ Scroll position saving test completed');
}

export function clearTestData() {
  console.log('🧹 Clearing Test Data');
  console.log('=====================');
  
  // Clear session data
  sessionStorage.removeItem('fechatter_session_channels');
  
  // Clear reading positions
  localStorage.removeItem('fechatter_reading_positions');
  
  // Clear visit history
  localStorage.removeItem('fechatter_channel_visits');
  
  console.log('✅ Test data cleared');
}

// Export for global access in development
if (typeof window !== 'undefined' && import.meta.env.DEV) {
  window.testReadingPosition = testReadingPositionManager;
  window.simulateChannelNav = simulateChannelNavigation;
  window.testScrollPositionSaving = testScrollPositionSaving;
  window.clearReadingPositionTestData = clearTestData;
  
  console.log('🧪 Reading position test functions available:');
  console.log('   - window.testReadingPosition()');
  console.log('   - window.simulateChannelNav()');
  console.log('   - window.testScrollPositionSaving()');
  console.log('   - window.clearReadingPositionTestData()');
}