<template>
  <!-- 🚨 REFACTORED: Minimal container, direct DiscordMessage rendering -->
  <div class="simple-message-list" ref="scrollContainer" @scroll="debouncedHandleScroll">

    <!-- Loading Indicator -->
    <div v-if="loading && messages.length === 0" class="loading-indicator">
      <div class="loading-spinner"></div>
      <span>Loading messages...</span>
    </div>

    <!-- 🔥 DIRECT RENDERING: No wrapper divs, no transitions, no excessive containers -->
    <template v-for="item in enhancedMessages"
      :key="item.id || item._stableKey || `divider_${item.type}_${item.id}`">
      
      <!-- Time Session Divider -->
      <TimeSessionDivider 
        v-if="item.type === 'date-divider' || item.type === 'session-divider'" 
        :divider="item"
        :compact="item.subType === 'short-break'" />

      <!-- 📝 DIRECT DiscordMessageItem - Zero wrapper containers -->
      <DiscordMessageItem 
        v-else
        :message="item" 
        :current-user-id="currentUserId" 
        :chat-id="chatId"
        :data-message-id="item.id"
        :ref="el => registerMessageElement(item.id, el)"
        @user-profile-opened="$emit('user-profile-opened', $event)" 
        @dm-created="$emit('dm-created', $event)" />
      
    </template>

    <!-- Scroll to Bottom Button -->
    <button v-if="showScrollToBottomButton" @click="scrollToBottom(true)" class="scroll-to-bottom-button">
      ↓ 最新
    </button>

  </div>
</template>

<script setup>
import { ref, nextTick, watch, computed } from 'vue';
import DiscordMessageItem from '@/components/discord/DiscordMessageItem.vue';
import TimeSessionDivider from './TimeSessionDivider.vue';
import { messageSessionGrouper } from '@/services/MessageSessionGrouper.js';

const props = defineProps({
  messages: { type: Array, default: () => [] },
  currentUserId: { type: Number, required: true },
  loading: { type: Boolean, default: false },
  chatId: { type: [Number, String], default: null },
  hasMoreMessages: { type: Boolean, default: false }
});

const emit = defineEmits(['user-profile-opened', 'dm-created', 'load-more-messages']);

// 🔥 MINIMAL State
const scrollContainer = ref(null);
const showScrollToBottomButton = ref(false);
const messageElements = ref(new Map());
const messageGroupingState = ref({ groupedMessages: [], lastProcessedCount: 0 });

// 🎯 Core: Enhanced message rendering (ultra-simplified)
const enhancedMessages = computed(() => {
  if (!props.messages?.length) return [];

  if (messageGroupingState.value.lastProcessedCount !== props.messages.length) {
    const result = messageSessionGrouper.analyzeAndGroupMessages(props.messages, props.chatId);
    messageGroupingState.value = {
      groupedMessages: result.groupedMessages,
      lastProcessedCount: props.messages.length
    };
  }

  return messageGroupingState.value.groupedMessages;
});

// 🔥 MINIMAL: Element registration
const registerMessageElement = (messageId, el) => {
  if (el) {
    messageElements.value.set(messageId, el);
    window.messageDisplayGuarantee?.markMessageDisplayed?.(parseInt(messageId), el, parseInt(props.chatId));
  } else {
    messageElements.value.delete(messageId);
  }
};

// 🔥 MINIMAL: Scroll functions
const scrollToBottom = (smooth = false) => {
  const container = scrollContainer.value;
  if (!container) return;
  
  container.scrollTo({
    top: container.scrollHeight,
    behavior: smooth ? 'smooth' : 'instant'
  });
  showScrollToBottomButton.value = false;
};

// 🔥 MINIMAL: Scroll handler
const debouncedHandleScroll = (() => {
  let timeoutId = null;
  return () => {
    if (timeoutId) clearTimeout(timeoutId);
    timeoutId = setTimeout(() => {
      const container = scrollContainer.value;
      if (!container) return;
      
      const { scrollTop, scrollHeight, clientHeight } = container;
      showScrollToBottomButton.value = scrollTop + clientHeight < scrollHeight - 100;
    }, 50);
  };
})();

// 🔥 Watch for new messages - auto scroll if near bottom
watch(() => props.messages?.length, (newLength, oldLength) => {
  if (newLength > (oldLength || 0)) {
    nextTick(() => {
      const container = scrollContainer.value;
      if (!container) return;
      
      const { scrollTop, scrollHeight, clientHeight } = container;
      if (scrollTop + clientHeight >= scrollHeight - 150) {
        scrollToBottom(true);
      }
    });
  }
});

defineExpose({ scrollToBottom, scrollContainer });
</script>

<style scoped>
/* 🔥 ULTRA-SIMPLIFIED: Core styles only */
.simple-message-list {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  background: #fdfcfa;
  scroll-behavior: smooth;
  contain: layout style paint;
}

.loading-indicator {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 12px;
  padding: 40px 20px;
  color: #6b7280;
  font-size: 14px;
}

.loading-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid #e5e7eb;
  border-top: 2px solid #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.scroll-to-bottom-button {
  position: fixed;
  bottom: 100px;
  right: 20px;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 20px;
  padding: 8px 16px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  cursor: pointer;
  z-index: 1000;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.2s ease;
}

.scroll-to-bottom-button:hover {
  background: #2563eb;
  transform: translateY(-2px);
}

@media (prefers-color-scheme: dark) {
  .simple-message-list { background: #1f2937; }
  .loading-indicator { color: #9ca3af; }
  .loading-spinner { border-color: #374151; border-top-color: #60a5fa; }
}
</style> 