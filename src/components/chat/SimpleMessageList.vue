<template>
  <div class="simple-message-list" ref="scrollContainer" @scroll="handleScroll">
    
    <!-- Auto Load More Indicator -->
    <Transition name="load-indicator" mode="out-in">
      <div v-if="showLoadingIndicator" class="auto-load-indicator">
        <div class="load-indicator-content">
          <div class="loading-spinner"></div>
          <span class="loading-text">Loading earlier messages...</span>
        </div>
      </div>
    </Transition>

    <!-- Traditional Loading Indicator for initial load -->
    <div v-if="loading && messages.length === 0" class="loading-indicator">
      <div class="loading-text">Loading messages...</div>
    </div>

    <!-- Messages List -->
    <div class="messages-wrapper">
      <!-- Debug Info -->
      <div v-if="showDebugInfo" class="debug-info">
        Messages: {{ messages?.length || 0 }} | Chat: {{ chatId }}
      </div>
      
      <div v-for="message in messages" :key="message.id" :data-message-id="message.id" class="message-wrapper"
        :ref="el => registerMessageElement(message.id, el)">
        <MessageItem :message="message" :current-user-id="currentUserId" :chat-id="chatId"
          @user-profile-opened="$emit('user-profile-opened', $event)" @dm-created="$emit('dm-created', $event)" />
      </div>
      
      <!-- No Messages Debug -->
      <div v-if="(!messages || messages.length === 0) && !loading" class="no-messages-debug">
        <p>No messages found for chat {{ chatId }}</p>
        <p>Loading: {{ loading }}</p>
        <p>Has more: {{ hasMoreMessages }}</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, nextTick, onMounted, watch, computed, onUnmounted } from 'vue';
import MessageItem from './MessageItem.vue';

const props = defineProps({
  messages: {
    type: Array,
    default: () => []
  },
  currentUserId: {
    type: Number,
    required: true
  },
  loading: {
    type: Boolean,
    default: false
  },
  chatId: {
    type: [Number, String],
    default: null
  },
  hasMoreMessages: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['user-profile-opened', 'dm-created', 'load-more-messages']);

const scrollContainer = ref(null);
const showLoadingIndicator = ref(false);
const isAutoLoading = ref(false);
const autoLoadCooldown = ref(false);

// Development mode detection
const isDev = import.meta.env.DEV;
const showDebugInfo = computed(() => isDev);

// Message element registration for display tracking
const messageElements = ref(new Map());

const registerMessageElement = (messageId, el) => {
  if (el) {
    messageElements.value.set(messageId, el);
    // Notify message display guarantee system
    if (window.messageDisplayGuarantee) {
      nextTick(() => {
        try {
          window.messageDisplayGuarantee.markMessageDisplayed(messageId, el, props.chatId);
          console.log(`✅ [SimpleMessageList] Registered message ${messageId} for display tracking`);
        } catch (error) {
          console.warn(`⚠️ [SimpleMessageList] Failed to register message ${messageId}:`, error);
        }
      });
    }
  } else {
    messageElements.value.delete(messageId);
  }
};

// Initial load state management
const initialLoadState = ref({
  isInitialLoad: true,
  hasScrolledToBottom: false,
  chatId: null
});

// Scroll to bottom
const scrollToBottom = (smooth = false, force = false) => {
  nextTick(() => {
    if (scrollContainer.value) {
      const container = scrollContainer.value;
      const targetScrollTop = container.scrollHeight - container.clientHeight;

      if (smooth && !force) {
        container.scrollTo({
          top: targetScrollTop,
          behavior: 'smooth'
        });
      } else {
        container.scrollTop = targetScrollTop;
      }

      console.log('🎯 [SimpleMessageList] 初始加载完成 - 静默滚动到底部');
    }
  });
};

// Auto load more messages
const triggerAutoLoad = async () => {
  if (isAutoLoading.value || autoLoadCooldown.value || !props.hasMoreMessages) {
    return;
  }

  console.log('🔄 [SimpleMessageList] 触发自动加载历史消息');
  
  isAutoLoading.value = true;
  showLoadingIndicator.value = true;
  autoLoadCooldown.value = true;

  try {
    emit('load-more-messages');
  } catch (error) {
    console.error('❌ [SimpleMessageList] 自动加载失败:', error);
    isAutoLoading.value = false;
    showLoadingIndicator.value = false;
  }

  // Cooldown period
  setTimeout(() => {
    autoLoadCooldown.value = false;
  }, 1000);
};

// Scroll event handler
const handleScroll = () => {
  const container = scrollContainer.value;
  if (!container) return;

  const scrollTop = container.scrollTop;
  const threshold = 50; // 50px from top

  // Check if need to auto load more messages
  if (scrollTop <= threshold && props.hasMoreMessages && !isAutoLoading.value && !autoLoadCooldown.value) {
    triggerAutoLoad();
  }
};

// Watch chat ID changes
watch(() => props.chatId, (newChatId, oldChatId) => {
  if (newChatId !== oldChatId) {
    console.log('🔄 [SimpleMessageList] Chat changed:', { from: oldChatId, to: newChatId });
    
    // Reset state
    initialLoadState.value = {
      isInitialLoad: true,
      hasScrolledToBottom: false,
      chatId: newChatId
    };
    
    isAutoLoading.value = false;
    showLoadingIndicator.value = false;
    autoLoadCooldown.value = false;
    messageElements.value.clear();

    // If switching from another chat, show completion message
    if (oldChatId !== null) {
      console.log('🎯 [SimpleMessageList] 频道切换完成，静默滚动到底部');
    }
  }
});

// Watch messages changes
watch(() => props.messages?.length, (newLength, oldLength) => {
  if (!newLength || newLength === 0) {
    return;
  }

  // Initial load
  if (initialLoadState.value.isInitialLoad && newLength > 0) {
    console.log('🎯 [SimpleMessageList] 初始进入频道，静默显示最新消息');
    scrollToBottom(false, true);
    initialLoadState.value.isInitialLoad = false;
    initialLoadState.value.hasScrolledToBottom = true;
    return;
  }

  // Auto load completed
  if (isAutoLoading.value && oldLength && newLength > oldLength) {
    setTimeout(() => {
      isAutoLoading.value = false;
      showLoadingIndicator.value = false;
    }, 500);
  }
});

// Component mounted
onMounted(() => {
  console.log('ℹ️ [SimpleMessageList] Component mounted for chat', props.chatId, {
    messageCount: props.messages?.length || 0,
    hasMessages: props.messages && props.messages.length > 0
  });

  // Initialize state
  initialLoadState.value = {
    isInitialLoad: true,
    hasScrolledToBottom: false,
    chatId: props.chatId
  };

  // If messages already exist, scroll to bottom immediately
  if (props.messages && props.messages.length > 0) {
    nextTick(() => {
      scrollToBottom(false, true);
      initialLoadState.value.isInitialLoad = false;
      initialLoadState.value.hasScrolledToBottom = true;
      console.log('🎯 [SimpleMessageList] 初始加载完成，用户滚动功能已启用');
    });
  }
});

// Component unmounted cleanup
onUnmounted(() => {
  isAutoLoading.value = false;
  showLoadingIndicator.value = false;
  messageElements.value.clear();
});

// Expose methods for external use
defineExpose({
  scrollToBottom,
  triggerAutoLoad,
  isAutoLoading: () => isAutoLoading.value,
  getMessageElements: () => messageElements.value
});
</script>

<style scoped>
.simple-message-list {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  background: #fdfcfa;
  position: relative;
}

.auto-load-indicator {
  position: absolute;
  top: 10px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 100;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(8px);
  border-radius: 20px;
  padding: 8px 16px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
  border: 1px solid rgba(124, 58, 237, 0.2);
}

.load-indicator-content {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #7c3aed;
  font-size: 13px;
  font-weight: 500;
}

.loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid #e0e7ff;
  border-top-color: #7c3aed;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.loading-indicator {
  text-align: center;
  padding: 20px;
  color: #666;
}

.messages-wrapper {
  width: 100%;
  max-width: 900px;
  margin: 0 auto;
  padding: 0 2rem;
  min-height: 100%;
  /* 🔧 CRITICAL FIX: Use 100% instead of 100vh to fit container */
  transform: translateZ(0);
  /* Create new layer for better performance */
}

.debug-info {
  position: sticky;
  top: 0;
  background: rgba(255, 255, 0, 0.1);
  padding: 4px 8px;
  font-size: 12px;
  color: #666;
  border-radius: 4px;
  margin-bottom: 8px;
  z-index: 10;
}

.no-messages-debug {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  height: 200px;
  color: #999;
  font-style: italic;
  text-align: center;
}

.message-wrapper {
  margin-bottom: 8px;
  transition: background-color 0.15s ease;
  border-radius: 8px;
  padding: 4px 0;
}

.message-wrapper:hover {
  background-color: rgba(124, 58, 237, 0.02);
}

/* Transition animations */
.load-indicator-enter-active,
.load-indicator-leave-active {
  transition: all 0.3s ease;
}

.load-indicator-enter-from,
.load-indicator-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-10px);
}

/* Scrollbar styling */
.simple-message-list::-webkit-scrollbar {
  width: 6px;
}

.simple-message-list::-webkit-scrollbar-track {
  background: transparent;
}

.simple-message-list::-webkit-scrollbar-thumb {
  background: rgba(124, 58, 237, 0.2);
  border-radius: 3px;
}

.simple-message-list::-webkit-scrollbar-thumb:hover {
  background: rgba(124, 58, 237, 0.35);
}

/* Responsive */
@media (max-width: 768px) {
  .messages-wrapper {
    margin: 0 auto;
    padding: 0 1rem;
    max-width: 100%;
  }
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  .message-wrapper {
    transition: none;
  }
  
  .load-indicator-enter-active,
  .load-indicator-leave-active {
    transition: none;
  }
}
</style>