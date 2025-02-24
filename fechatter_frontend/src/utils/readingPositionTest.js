/**
 * Reading Position Test Suite
 * 测试阅读位置管理功能
 */

import { readingPositionManager } from './readingPositionManager';

export function testReadingPositionManager() {
  if (import.meta.env.DEV) {
    console.log('🧪 Testing Reading Position Manager');
  if (import.meta.env.DEV) {
    console.log('=====================================');
  }
  
  const testChatId = 123;
  
  // Test 1: Check initial state
  if (import.meta.env.DEV) {
    console.log('\n📊 Test 1: Initial State');
  const isVisited = readingPositionManager.isChannelVisitedInSession(testChatId);
  if (import.meta.env.DEV) {
    console.log('- Channel visited in session:', isVisited);
  }
  
  const strategy = readingPositionManager.getLoadingStrategy(testChatId);
  if (import.meta.env.DEV) {
    console.log('- Loading strategy:', strategy);
  }
  
  // Test 2: Mark as visited
  if (import.meta.env.DEV) {
    console.log('\n📊 Test 2: Mark as Visited');
  readingPositionManager.markChannelAsVisited(testChatId);
  
  const isVisitedAfter = readingPositionManager.isChannelVisitedInSession(testChatId);
  if (import.meta.env.DEV) {
    console.log('- Channel visited after marking:', isVisitedAfter);
  }
  
  const strategyAfter = readingPositionManager.getLoadingStrategy(testChatId);
  if (import.meta.env.DEV) {
    console.log('- Loading strategy after visit:', strategyAfter);
  }
  
  // Test 3: Save reading position
  if (import.meta.env.DEV) {
    console.log('\n📊 Test 3: Save Reading Position');
  const testPosition = {
    messageId: 456,
    scrollOffset: 100,
    totalMessages: 50
  };
  
  readingPositionManager.saveReadingPosition(testChatId, testPosition);
  
  const savedPosition = readingPositionManager.getReadingPosition(testChatId);
  if (import.meta.env.DEV) {
    console.log('- Saved position:', savedPosition);
  }
  
  // Test 4: Strategy with saved position
  if (import.meta.env.DEV) {
    console.log('\n📊 Test 4: Strategy with Saved Position');
  const strategyWithPosition = readingPositionManager.getLoadingStrategy(testChatId);
  if (import.meta.env.DEV) {
    console.log('- Strategy with saved position:', strategyWithPosition);
  }
  
  // Test 5: Debug info
  if (import.meta.env.DEV) {
    console.log('\n📊 Test 5: Debug Information');
  const debugInfo = readingPositionManager.getDebugInfo();
  if (import.meta.env.DEV) {
    console.log('- Debug info:', debugInfo);
  }
  
  if (import.meta.env.DEV) {
    console.log('\n✅ Reading Position Manager test completed');
  return debugInfo;
}

export function simulateChannelNavigation() {
  if (import.meta.env.DEV) {
    console.log('🧪 Simulating Channel Navigation');
  if (import.meta.env.DEV) {
    console.log('=================================');
  }
  
  const channels = [101, 102, 103, 104, 105];
  
  channels.forEach((channelId, index) => {
    if (import.meta.env.DEV) {
      console.log(`\n🔄 Navigating to channel ${channelId}`);
    }
    
    // Get strategy before visit
    const beforeStrategy = readingPositionManager.getLoadingStrategy(channelId);
    if (import.meta.env.DEV) {
      console.log('- Strategy before visit:', beforeStrategy.type, '-', beforeStrategy.reason);
    }
    
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
      if (import.meta.env.DEV) {
        console.log('- Saved position for message:', position.messageId);
      }
    
    // Get strategy after visit
    const afterStrategy = readingPositionManager.getLoadingStrategy(channelId);
    if (import.meta.env.DEV) {
      console.log('- Strategy after visit:', afterStrategy.type, '-', afterStrategy.reason);
    }
  });
  
  if (import.meta.env.DEV) {
    console.log('\n📊 Final State:');
  if (import.meta.env.DEV) {
    console.log('- Session channels:', [...readingPositionManager.sessionChannels]);
  if (import.meta.env.DEV) {
    console.log('- Reading positions:', Object.keys(readingPositionManager.getReadingPositions()));
  }
  
  if (import.meta.env.DEV) {
    console.log('\n✅ Channel navigation simulation completed');
  }

export function testScrollPositionSaving() {
  if (import.meta.env.DEV) {
    console.log('🧪 Testing Scroll Position Saving');
  if (import.meta.env.DEV) {
    console.log('==================================');
  }
  
  // Mock DOM elements for testing
  const mockMessages = [
    { id: 1, content: 'Message 1' },
    { id: 2, content: 'Message 2' },
    { id: 3, content: 'Message 3' },
    { id: 4, content: 'Message 4' },
    { id: 5, content: 'Message 5' }
  ];
  
  if (import.meta.env.DEV) {
    console.log('📝 Mock messages created:', mockMessages.length);
  }
  
  // Simulate saving positions for different messages
  mockMessages.forEach(message => {
    const position = {
      messageId: message.id,
      scrollOffset: Math.floor(Math.random() * 100),
      totalMessages: mockMessages.length
    };
    
    readingPositionManager.saveReadingPosition(999, position);
    if (import.meta.env.DEV) {
      console.log(`- Saved position for message ${message.id}`);
    }
  });
  
  const finalPosition = readingPositionManager.getReadingPosition(999);
  if (import.meta.env.DEV) {
    console.log('📍 Final saved position:', finalPosition);
  }
  
  if (import.meta.env.DEV) {
    console.log('\n✅ Scroll position saving test completed');
  }

export function clearTestData() {
  if (import.meta.env.DEV) {
    console.log('🧹 Clearing Test Data');
  if (import.meta.env.DEV) {
    console.log('=====================');
  }
  
  // Clear session data
  sessionStorage.removeItem('fechatter_session_channels');
  
  // Clear reading positions
  localStorage.removeItem('fechatter_reading_positions');
  
  // Clear visit history
  localStorage.removeItem('fechatter_channel_visits');
  
  if (import.meta.env.DEV) {
    console.log('✅ Test data cleared');
  }

// Export for global access in development
if (typeof window !== 'undefined' && import.meta.env.DEV) {
  window.testReadingPosition = testReadingPositionManager;
  window.simulateChannelNav = simulateChannelNavigation;
  window.testScrollPositionSaving = testScrollPositionSaving;
  window.clearReadingPositionTestData = clearTestData;
  
  if (import.meta.env.DEV) {
    console.log('🧪 Reading position test functions available:');
  if (import.meta.env.DEV) {
    console.log('   - window.testReadingPosition()');
  if (import.meta.env.DEV) {
    console.log('   - window.simulateChannelNav()');
  if (import.meta.env.DEV) {
    console.log('   - window.testScrollPositionSaving()');
  if (import.meta.env.DEV) {
    console.log('   - window.clearReadingPositionTestData()');
  }
}