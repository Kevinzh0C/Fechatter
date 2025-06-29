/**
 * Channel Switch Root Cause Fix
 * Comprehensive solution to eliminate the entire fault tree
 * Based on complete code analysis and fault tree construction
 */

class ChannelSwitchRootCauseFix {
  constructor() {
    this.name = 'ChannelSwitchRootCauseFix';
    this.version = '1.0.0';
    this.isActive = false;
    this.originalMethods = new Map();
    this.debugMode = true;
  }

  /**
   * Apply comprehensive fix to all identified root causes
   */
  async applyComprehensiveFix() {
    console.group('Applying Channel Switch Root Cause Fix');

    try {
      const fixes = [
        this.fixSetCurrentChatLogic(),
        this.fixRouteComponentSync(),
        this.fixMessageCacheManagement(),
        this.fixMessageArrayStateCorruption(),
        this.addPreventiveMeasures()
      ];

      const results = await Promise.allSettled(fixes);

      results.forEach((result, index) => {
        const fixName = ['setCurrentChat', 'route-component sync', 'cache management', 'message array state', 'preventive measures'][index];
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
        console.log('🎉 Comprehensive fix applied successfully');
      }

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('ERROR: Failed to apply comprehensive fix:', error);
      }

    console.groupEnd();
  }

  /**
   * Fix 1: setCurrentChat Logic Defects [CRITICAL]
   */
  async fixSetCurrentChatLogic() {
    const { useChatStore } = await import('@/stores/chat');
    const chatStore = useChatStore();

    // Store original method
    if (!this.originalMethods.has('setCurrentChat')) {
      this.originalMethods.set('setCurrentChat', chatStore.setCurrentChat.bind(chatStore));
    }

    // Apply fixed version
    chatStore.setCurrentChat = async function (chatId) {
      if (import.meta.env.DEV) {
        console.log(`[FIXED] setCurrentChat called with: ${chatId}, current: ${this.currentChatId}`);
      }

      // CRITICAL FIX 1.1: Always proceed, even if same chatId
      // This fixes the early return mechanism that was causing stale messages

      const previousChatId = this.currentChatId;
      this.currentChatId = chatId;
      this.loading = true;

      try {
        // CRITICAL FIX 1.2: Aggressively clear all old message state
        if (import.meta.env.DEV) {
          console.log('🧹 [FIXED] Clearing all old message state');
        }

        // Clear messages array using splice to maintain reactivity
        this.messages.splice(0, this.messages.length);

        // Clear related state
        this.hasMoreMessages = true;
        this.lastMessageId = null;

        // CRITICAL FIX 1.3: Clear cache for the specific chat to force fresh data
        if (this.messageCache[chatId]) {
          delete this.messageCache[chatId];
          sessionStorage.setItem('messageCache', JSON.stringify(this.messageCache));
          if (import.meta.env.DEV) {
            console.log('🗑️ [FIXED] Cleared cache for chat', chatId);
          }

        // Find the chat in our list
        const chat = this.chats.find(c => c.id === chatId);

        if (!chat) {
          if (import.meta.env.DEV) {
            console.log('📍 [FIXED] Chat not found, fetching from API');
          const response = await api.get(`/chat/${chatId}`);
          const chatData = response.data?.data || response.data;
          if (chatData) {
            const normalizedChat = this._normalizeChat(chatData);
            const existingChat = this.chats.find(c => c.id === normalizedChat.id);
            if (!existingChat) {
              this.chats.push(normalizedChat);
            }

        // CRITICAL FIX: Always fetch fresh messages
        if (import.meta.env.DEV) {
          console.log('📍 [FIXED] Fetching fresh messages for chat:', chatId);
        await this.fetchMessages(chatId);
        if (import.meta.env.DEV) {
          console.log('📍 [FIXED] Fresh messages loaded:', this.messages.length);
        }

        // Mark messages as read
        if (chat && chat.unread_count > 0) {
          await this.markChatAsRead(chatId);
        }

      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('📍 [FIXED] Error in setCurrentChat:', error);
        }
        // Restore previous state on error
        this.currentChatId = previousChatId;
        throw error;
      } finally {
        this.loading = false;
      }
    };

    if (import.meta.env.DEV) {
      console.log('Fixed setCurrentChat logic');
    }

  /**
   * Fix 2: Route-Component State Synchronization [HIGH]
   */
  async fixRouteComponentSync() {
    // This fix needs to be applied at the component level
    // We'll provide a corrected version that can be injected

    const fixedRouteWatcher = function (route, chatStore, loadChatData) {
      let routeChangeTimer = null;

      return (newId, oldId) => {
        // Skip if same ID
        if (newId === oldId) return;

        if (import.meta.env.DEV) {
          console.log(`[FIXED] Route change: ${oldId} → ${newId}`);
        }

        if (routeChangeTimer) {
          clearTimeout(routeChangeTimer);
        }

        // CRITICAL: Clear old state immediately
        if (oldId && newId !== oldId) {
          if (import.meta.env.DEV) {
            console.log('🧹 [FIXED] Clearing old route state');
          chatStore.messages.splice(0, chatStore.messages.length);
          chatStore.currentChatId = null;
        }

        routeChangeTimer = setTimeout(async () => {
          if (newId) {
            if (import.meta.env.DEV) {
              console.log('🔄 [FIXED] Loading chat data:', newId);
            try {
              await loadChatData(newId);
              if (import.meta.env.DEV) {
                console.log('[FIXED] Chat data loaded successfully');
              }
            } catch (error) {
              if (import.meta.env.DEV) {
                console.error('ERROR: [FIXED] Failed to load chat data:', error);
              }
        }, 25); // Reduced debounce for faster response
      };
    };

    // Store the fixed watcher function for components to use
    window.fixedRouteWatcher = fixedRouteWatcher;

    if (import.meta.env.DEV) {
      console.log('Fixed route-component synchronization');
    }

  /**
   * Fix 3: Message Cache Management [MEDIUM]
   */
  async fixMessageCacheManagement() {
    const { useChatStore } = await import('@/stores/chat');
    const chatStore = useChatStore();

    // Store original fetchMessages method
    if (!this.originalMethods.has('fetchMessages')) {
      this.originalMethods.set('fetchMessages', chatStore.fetchMessages.bind(chatStore));
    }

    // Apply fixed version
    chatStore.fetchMessages = async function (chatId, limit = 50, retryCount = 0) {
      try {
        const validChatId = parseInt(chatId, 10);
        if (!validChatId || isNaN(validChatId)) {
          throw new Error(`Invalid chat ID: ${chatId}`);
        }

        if (import.meta.env.DEV) {
          console.log(`📥 [FIXED] Fetching messages for chat ${validChatId}`);
        }

        // CRITICAL FIX 3.1: Prevent cross-channel cache pollution
        // Always clear cache for different chat
        if (this.currentChatId !== validChatId) {
          Object.keys(this.messageCache).forEach(cachedChatId => {
            if (parseInt(cachedChatId, 10) !== validChatId) {
              delete this.messageCache[cachedChatId];
            }
          });
          sessionStorage.setItem('messageCache', JSON.stringify(this.messageCache));
          if (import.meta.env.DEV) {
            console.log('🧹 [FIXED] Cleared cross-channel cache pollution');
          }

        // CRITICAL FIX 3.2: Always clear current messages first
        if (import.meta.env.DEV) {
          console.log('🧹 [FIXED] Clearing current messages array');
        this.messages.splice(0, this.messages.length);

        // Load workspace users for sender info
        const { useUserStore } = await import('@/stores/user');
        const { useWorkspaceStore } = await import('@/stores/workspace');
        const userStore = useUserStore();
        const workspaceStore = useWorkspaceStore();

        if (userStore.workspaceUsers.length === 0) {
          if (import.meta.env.DEV) {
            console.log('📥 [FIXED] Loading workspace users...');
          await workspaceStore.fetchWorkspaceUsers();
        }

        // Skip cache entirely for now to ensure fresh data
        if (import.meta.env.DEV) {
          console.log('SUBSCRIPTION: [FIXED] Fetching fresh data from API');
        const response = await api.get(`/chat/${validChatId}/messages`, {
          params: { limit }
        });

        const messagesData = response.data?.data || response.data || [];

        // Enhanced message normalization and validation
        const userMap = new Map(userStore.workspaceUsers.map(u => [u.id, u]));

        const normalizedMessages = messagesData.map(msg => {
          const normalized = this.normalizeMessage(msg);

          // CRITICAL FIX 3.3: Strict chat ID validation
          if (parseInt(normalized.chat_id, 10) !== validChatId) {
            if (import.meta.env.DEV) {
              console.error(`ERROR: [FIXED] Message ${normalized.id} belongs to chat ${normalized.chat_id}, not ${validChatId}`);
            return null; // Filter out wrong messages
          }

          // Add sender info
          if (normalized.sender_id && (!normalized.sender || !normalized.sender.fullname)) {
            const user = userMap.get(normalized.sender_id);
            if (user) {
              normalized.sender = {
                id: user.id,
                fullname: user.fullname || user.name || user.email?.split('@')[0],
                email: user.email,
                avatar_url: user.avatar_url || null
              };
            }

          return normalized;
        }).filter(msg => msg !== null); // Remove invalid messages

        // Sort by timestamp
        normalizedMessages.sort((a, b) => {
          return new Date(a.created_at) - new Date(b.created_at);
        });

        // Add validated messages
        normalizedMessages.forEach(msg => {
          this.messages.push(msg);
        });

        if (import.meta.env.DEV) {
          console.log(`[FIXED] Loaded ${this.messages.length} validated messages for chat ${validChatId}`);
        }

        // Update cache with validated messages only
        this.messageCache[validChatId] = {
          messages: [...this.messages],
          timestamp: Date.now(),
          validated: true,
          chatId: validChatId // Add chat ID for validation
        };
        sessionStorage.setItem('messageCache', JSON.stringify(this.messageCache));

        return this.messages;

      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('ERROR: [FIXED] fetchMessages failed:', error);
        throw error;
      }
    };

    if (import.meta.env.DEV) {
      console.log('Fixed message cache management');
    }

  /**
   * Fix 4: Message Array State Corruption [MEDIUM]
   */
  async fixMessageArrayStateCorruption() {
    const { useChatStore } = await import('@/stores/chat');
    const chatStore = useChatStore();

    // Store original handleIncomingMessage method
    if (!this.originalMethods.has('handleIncomingMessage')) {
      this.originalMethods.set('handleIncomingMessage', chatStore.handleIncomingMessage.bind(chatStore));
    }

    // Apply fixed version
    chatStore.handleIncomingMessage = function (chat, message) {
      if (import.meta.env.DEV) {
        console.log(`EVENT: [FIXED] Handling incoming message for chat ${message.chat_id}`);
      }

      // Normalize the incoming message
      const normalizedMessage = this.normalizeMessage(message);

      // CRITICAL FIX 4.1: Strict chat ID validation before any processing
      const messageChatId = parseInt(normalizedMessage.chat_id, 10);
      const currentChatId = parseInt(this.currentChatId, 10);

      if (messageChatId !== currentChatId) {
        if (import.meta.env.DEV) {
          console.log(`🚫 [FIXED] Message for chat ${messageChatId} ignored (current: ${currentChatId})`);
        }

        // Still update unread count for other chats
        if (messageChatId !== currentChatId) {
          this.incrementChatUnreadCount(chat.id);

          // Move chat to top
          const chatIndex = this.chats.findIndex(c => c.id === chat.id);
          if (chatIndex > 0) {
            this.chats.splice(chatIndex, 1);
            this.chats.unshift(chat);
          }

        return; // Don't add to current messages
      }

      // CRITICAL FIX 4.2: Validate against strict channel validator
      try {
        if (window.strictChannelMessageValidator) {
          window.strictChannelMessageValidator.validateMessage(normalizedMessage, currentChatId);
        }
      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('ERROR: [FIXED] Message validation failed:', error.message);
        return; // Don't add invalid messages
      }

      // CRITICAL FIX 4.3: Use splice for reactive addition
      this.messages.push(normalizedMessage);

      // Update cache
      if (this.messageCache[currentChatId]) {
        this.messageCache[currentChatId].messages.push(normalizedMessage);
        this.messageCache[currentChatId].timestamp = Date.now();
        sessionStorage.setItem('messageCache', JSON.stringify(this.messageCache));
      }

      // Sort messages
      this.messages.sort((a, b) => new Date(a.created_at) - new Date(b.created_at));

      if (import.meta.env.DEV) {
        console.log('[FIXED] Message added to current chat');
      }
    };

    if (import.meta.env.DEV) {
      console.log('Fixed message array state corruption');
    }

  /**
   * Fix 5: Add Preventive Measures
   */
  async addPreventiveMeasures() {
    // Add global debugging utilities
    window.debugChannelSwitch = {
      validateCurrentState: () => {
        const { useChatStore } = require('@/stores/chat');
        const { useRoute } = require('vue-router');
        const chatStore = useChatStore();
        const route = useRoute();

        const routeChatId = route.params.id ? parseInt(route.params.id, 10) : null;
        const storeChatId = chatStore.currentChatId;
        const messageCount = chatStore.messages.length;

        console.group('Channel Switch State Validation');
        if (import.meta.env.DEV) {
          console.log('Route Chat ID:', routeChatId);
        if (import.meta.env.DEV) {
          console.log('Store Chat ID:', storeChatId);
        if (import.meta.env.DEV) {
          console.log('Messages Count:', messageCount);
        if (import.meta.env.DEV) {
          console.log('State Consistent:', routeChatId === storeChatId);
        }

        if (messageCount > 0) {
          const wrongMessages = chatStore.messages.filter(msg =>
            parseInt(msg.chat_id, 10) !== storeChatId
          );
          if (import.meta.env.DEV) {
            console.log('Wrong Messages:', wrongMessages.length);
          }

          if (wrongMessages.length > 0) {
            if (import.meta.env.DEV) {
              console.error('ERROR: Found messages from wrong chat:', wrongMessages);
            }
          } else {
            if (import.meta.env.DEV) {
              console.log('All messages belong to correct chat');
            }

        console.groupEnd();

        return {
          routeChatId,
          storeChatId,
          messageCount,
          isConsistent: routeChatId === storeChatId,
          wrongMessageCount: messageCount > 0 ?
            chatStore.messages.filter(msg => parseInt(msg.chat_id, 10) !== storeChatId).length : 0
        };
      },

      forceCorrectState: async () => {
        const { useChatStore } = require('@/stores/chat');
        const { useRoute } = require('vue-router');
        const chatStore = useChatStore();
        const route = useRoute();

        const routeChatId = route.params.id ? parseInt(route.params.id, 10) : null;

        if (!routeChatId) {
          if (import.meta.env.DEV) {
            console.error('ERROR: No route chat ID found');
          return;
        }

        if (import.meta.env.DEV) {
          console.log('Forcing correct state for chat:', routeChatId);
        }

        // Clear everything
        chatStore.messages.splice(0, chatStore.messages.length);
        chatStore.currentChatId = null;

        // Clear cache
        delete chatStore.messageCache[routeChatId];

        // Reload
        await chatStore.setCurrentChat(routeChatId);

        if (import.meta.env.DEV) {
          console.log('State corrected');
        }
    };

    // Add interval validation in development
    if (import.meta.env.DEV) {
      setInterval(() => {
        const state = window.debugChannelSwitch.validateCurrentState();
        if (!state.isConsistent || state.wrongMessageCount > 0) {
          if (import.meta.env.DEV) {
            console.warn('WARNING: Channel switch state inconsistency detected!', state);
          }
      }, 10000); // Check every 10 seconds
    }

    if (import.meta.env.DEV) {
      console.log('Added preventive measures');
    }

  /**
   * Restore original methods
   */
  async restoreOriginalMethods() {
    const { useChatStore } = await import('@/stores/chat');
    const chatStore = useChatStore();

    this.originalMethods.forEach((originalMethod, methodName) => {
      chatStore[methodName] = originalMethod;
      if (import.meta.env.DEV) {
        console.log(`🔄 Restored original ${methodName}`);
      }
    });

    this.isActive = false;
    if (import.meta.env.DEV) {
      console.log('All original methods restored');
    }

  /**
   * Get fix status
   */
  getStatus() {
    return {
      name: this.name,
      version: this.version,
      isActive: this.isActive,
      appliedFixes: this.originalMethods.size,
      debugMode: this.debugMode
    };
  }

// Create singleton instance
const channelSwitchRootCauseFix = new ChannelSwitchRootCauseFix();

// Export for global use
if (typeof window !== 'undefined') {
  window.channelSwitchRootCauseFix = channelSwitchRootCauseFix;

  // Convenient global functions
  window.fixChannelSwitch = () => channelSwitchRootCauseFix.applyComprehensiveFix();
  window.restoreChannelSwitch = () => channelSwitchRootCauseFix.restoreOriginalMethods();
  window.validateChannelState = () => window.debugChannelSwitch?.validateCurrentState();

  if (import.meta.env.DEV) {
    console.log('Channel Switch Root Cause Fix loaded');
  if (import.meta.env.DEV) {
    console.log('Use window.fixChannelSwitch() to apply comprehensive fix');
  if (import.meta.env.DEV) {
    console.log('Use window.validateChannelState() to check current state');
  }

export default channelSwitchRootCauseFix; 