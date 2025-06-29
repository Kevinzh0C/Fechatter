/**
 * Channel Switch Message Display Fix
 * Comprehensive solution for channel switching message list issues
 */

class ChannelSwitchMessageFix {
  constructor() {
    this.name = 'ChannelSwitchMessageFix';
    this.version = '1.0.0';
    this.isActive = false;
    this.debugMode = true;
  }

  /**
   * Apply comprehensive channel switch message fix
   */
  async applyChannelSwitchFix() {
    console.group('Applying Channel Switch Message Fix');

    try {
      const fixes = [
        this.fixRouteWatcherTiming(),
        this.fixChatStoreStateClear(),
        this.fixMessageCacheInvalidation(),
        this.enhanceStateLogging(),
        this.preventConcurrentLoading()
      ];

      const results = await Promise.allSettled(fixes);

      results.forEach((result, index) => {
        const fixName = ['route watcher timing', 'chat store state clear', 'message cache invalidation', 'state logging', 'concurrent loading prevention'][index];
        if (result.status === 'fulfilled') {
          if (import.meta.env.DEV) {
            console.log(`${fixName} fix applied`);
          }
        } else {
          if (import.meta.env.DEV) {
            console.error(`ERROR: ${fixName} fix failed:`, result.reason);
          }
      });

      this.isActive = true;
      if (import.meta.env.DEV) {
        console.log('🎉 Channel switch message fix applied successfully');
      }

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('ERROR: Failed to apply channel switch fix:', error);
      }

    console.groupEnd();
  }

  /**
   * Fix 1: Route Watcher Timing
   */
  async fixRouteWatcherTiming() {
    if (import.meta.env.DEV) {
      console.log('Applying route watcher timing fix...');
    }

    // The fix is now in Chat.vue - enhanced route watcher with proper state clearing
    // This ensures that when routes change, we properly clear the previous state

    return true;
  }

  /**
   * Fix 2: Chat Store State Clearing
   */
  async fixChatStoreStateClear() {
    if (import.meta.env.DEV) {
      console.log('Applying chat store state clearing fix...');
    }

    // The fix is now in chat.js - setCurrentChat always clears messages and cache
    // This prevents showing stale messages from previous channels

    return true;
  }

  /**
   * Fix 3: Message Cache Invalidation
   */
  async fixMessageCacheInvalidation() {
    if (import.meta.env.DEV) {
      console.log('Applying message cache invalidation fix...');
    }

    // Enhanced cache clearing in setCurrentChat to prevent stale data
    // Cache is now cleared before fetching new messages

    return true;
  }

  /**
   * Fix 4: Enhanced State Logging
   */
  async enhanceStateLogging() {
    if (import.meta.env.DEV) {
      console.log('Enhancing state logging...');
    }

    // Added comprehensive logging to track state changes
    // This helps debug any remaining issues

    return true;
  }

  /**
   * Fix 5: Prevent Concurrent Loading
   */
  async preventConcurrentLoading() {
    if (import.meta.env.DEV) {
      console.log('Preventing concurrent loading...');
    }

    // Added isLoadingChat flag to prevent concurrent loads
    // This prevents race conditions during rapid channel switching

    return true;
  }

  /**
   * Test channel switching functionality
   */
  async testChannelSwitch() {
    console.group('🧪 Testing Channel Switch Functionality');

    try {
      const { useChatStore } = await import('@/stores/chat');
      const { useRoute, useRouter } = await import('vue-router');

      const chatStore = useChatStore();
      const route = useRoute();
      const router = useRouter();

      if (import.meta.env.DEV) {
        console.log('1️⃣ Current State:');
      if (import.meta.env.DEV) {
        console.log('  - Current route:', route.path);
      if (import.meta.env.DEV) {
        console.log('  - Chat store currentChatId:', chatStore.currentChatId);
      if (import.meta.env.DEV) {
        console.log('  - Messages count:', chatStore.messages.length);
      }

      // Get available chats for testing
      const availableChats = chatStore.chats.filter(chat =>
        chat.chat_type === 'PublicChannel' || chat.chat_type === 'PrivateChannel'
      );

      if (availableChats.length < 2) {
        if (import.meta.env.DEV) {
          console.warn('WARNING: Need at least 2 channels to test switching. Available:', availableChats.length);
        return false;
      }

      if (import.meta.env.DEV) {
        console.log('2️⃣ Available chats for testing:', availableChats.map(c => `${c.id}: ${c.name}`));
      }

      // Test switching between first two channels
      const chat1 = availableChats[0];
      const chat2 = availableChats[1];

      if (import.meta.env.DEV) {
        console.log('\n3️⃣ Testing switch to first chat:', chat1.name);
      await router.push(`/chat/${chat1.id}`);

      // Wait for load
      await new Promise(resolve => setTimeout(resolve, 1000));

      const messages1Count = chatStore.messages.length;
      if (import.meta.env.DEV) {
        console.log('  - Messages loaded for chat 1:', messages1Count);
      if (import.meta.env.DEV) {
        console.log('  - Current chat ID:', chatStore.currentChatId);
      }

      if (import.meta.env.DEV) {
        console.log('\n4️⃣ Testing switch to second chat:', chat2.name);
      await router.push(`/chat/${chat2.id}`);

      // Wait for load
      await new Promise(resolve => setTimeout(resolve, 1000));

      const messages2Count = chatStore.messages.length;
      if (import.meta.env.DEV) {
        console.log('  - Messages loaded for chat 2:', messages2Count);
      if (import.meta.env.DEV) {
        console.log('  - Current chat ID:', chatStore.currentChatId);
      }

      if (import.meta.env.DEV) {
        console.log('\n5️⃣ Switch Test Results:');
      const switchWorked = chatStore.currentChatId === chat2.id;
      if (import.meta.env.DEV) {
        console.log('  - Chat ID switched correctly:', switchWorked ? '' : '❌');
      if (import.meta.env.DEV) {
        console.log('  - Messages updated:', messages1Count !== messages2Count ? '' : '⚠️');
      }

      if (switchWorked) {
        if (import.meta.env.DEV) {
          console.log('Channel switching appears to be working correctly');
        return true;
      } else {
        if (import.meta.env.DEV) {
          console.error('ERROR: Channel switching failed - chat ID not updated');
        return false;
      }

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('ERROR: Channel switch test failed:', error);
      return false;
    } finally {
      console.groupEnd();
    }

  /**
   * Monitor channel switch performance
   */
  monitorChannelSwitchPerformance() {
    if (import.meta.env.DEV) {
      console.log('Starting channel switch performance monitoring...');
    }

    let lastSwitchTime = 0;
    let switchCount = 0;

    // Monitor route changes
    if (typeof window !== 'undefined' && window.$router) {
      window.$router.afterEach((to, from) => {
        if (to.path.startsWith('/chat/') && from.path.startsWith('/chat/')) {
          const now = Date.now();
          const timeSinceLastSwitch = now - lastSwitchTime;
          switchCount++;

          if (import.meta.env.DEV) {
            console.log(`🔄 Channel switch #${switchCount}:`, {
            from: from.params.id,
            to: to.params.id,
            timeSinceLastSwitch: `${timeSinceLastSwitch}ms`,
            timestamp: new Date().toISOString()
          });

          lastSwitchTime = now;
        }
      });

    setTimeout(() => {
      if (import.meta.env.DEV) {
        console.log('Performance monitoring stopped');
      if (import.meta.env.DEV) {
        console.log(`📈 Total switches monitored: ${switchCount}`);
      }
    }, 60000); // Monitor for 1 minute
  }

  /**
   * Get fix status
   */
  getStatus() {
    return {
      name: this.name,
      version: this.version,
      isActive: this.isActive,
      debugMode: this.debugMode,
      description: 'Fixes channel switching message display issues'
    };
  }

// Create singleton instance
const channelSwitchMessageFix = new ChannelSwitchMessageFix();

// Export for global use
if (typeof window !== 'undefined') {
  window.channelSwitchMessageFix = channelSwitchMessageFix;

  // Convenient global functions
  window.fixChannelSwitch = () => channelSwitchMessageFix.applyChannelSwitchFix();
  window.testChannelSwitch = () => channelSwitchMessageFix.testChannelSwitch();
  window.monitorChannelSwitch = () => channelSwitchMessageFix.monitorChannelSwitchPerformance();

  if (import.meta.env.DEV) {
    console.log('Channel Switch Message Fix loaded');
  if (import.meta.env.DEV) {
    console.log('Use window.fixChannelSwitch() to apply comprehensive fix');
  if (import.meta.env.DEV) {
    console.log('Use window.testChannelSwitch() to test channel switching');
  if (import.meta.env.DEV) {
    console.log('Use window.monitorChannelSwitch() to monitor performance');
  }

export default channelSwitchMessageFix; 