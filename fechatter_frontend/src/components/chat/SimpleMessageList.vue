<template>
  <!-- 🧹 CLEARED: SimpleMessageList message display completely removed -->
  <div class="simple-message-list" ref="scrollContainer">

    <!-- Loading Indicator -->
    <div v-if="loading && messages.length === 0" class="loading-indicator">
      <div class="loading-spinner"></div>
      <span>Loading messages...</span>
    </div>

    <!-- Load More Messages Indicator -->
    <div v-if="loading && messages.length > 0" class="load-more-indicator">
      <div class="loading-spinner"></div>
      <span>Loading earlier messages...</span>
    </div>

    <!-- 🧹 CLEARED: All message rendering removed -->
    <div class="cleared-message-display">
      <div class="cleared-notice">
        <h3>🧹 Message Display Cleared</h3>
        <p>All MessageItem components have been removed.</p>
        <p>Chat ID: {{ chatId }} | Messages Count: {{ messages?.length || 0 }}</p>
        <p>Awaiting new Discord MessageItem implementation...</p>
      </div>
    </div>

    <!-- Scroll to Bottom Button -->
    <button v-if="showScrollToBottomButton" @click="scrollToBottom(true)" class="scroll-to-bottom-button">
      ↓ Jump to latest
    </button>

  </div>
</template>

<script setup>
import { ref, nextTick, watch, computed } from 'vue';

// 🧹 MINIMAL: Only essential props preserved
const props = defineProps({
  messages: { type: Array, default: () => [] },
  currentUserId: { type: Number, required: true },
  loading: { type: Boolean, default: false },
  chatId: { type: [Number, String], default: null },
  hasMoreMessages: { type: Boolean, default: false }
});

// 🧹 CLEARED: All message display logic removed
const emit = defineEmits(['user-profile-opened', 'dm-created', 'load-more-messages']);

// 🧹 MINIMAL: Basic scroll state only
const scrollContainer = ref(null);
const showScrollToBottomButton = ref(false);

// 🧹 Simple scroll to bottom function
const scrollToBottom = (smooth = false) => {
  const container = scrollContainer.value;
  if (!container) return;

  container.scrollTo({
    top: container.scrollHeight,
    behavior: smooth ? 'smooth' : 'instant'
  });
  showScrollToBottomButton.value = false;
};

// Console log for verification
console.log(`🧹 [SimpleMessageList] CLEARED - Chat ${props.chatId} message display removed`);

defineExpose({
  scrollToBottom,
  scrollContainer
});
</script>

<style scoped>
.simple-message-list {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  background: var(--bg-primary, #313338);
  display: flex;
  flex-direction: column;
  align-items: center;
  max-width: 860px;
  margin: 0 auto;
  padding: 16px 20px;
  gap: 8px;
  min-height: 100%;
}

.cleared-message-display {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
}

.cleared-notice {
  text-align: center;
  padding: 40px 20px;
  background: var(--bg-secondary, #2b2d31);
  border: 2px dashed var(--border-primary, #3e4146);
  border-radius: 12px;
  max-width: 500px;
}

.cleared-notice h3 {
  color: var(--text-primary, #dbdee1);
  margin-bottom: 12px;
  font-size: 18px;
}

.cleared-notice p {
  color: var(--text-muted, #949ba4);
  margin: 8px 0;
  font-size: 14px;
}

.loading-indicator,
.load-more-indicator {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 12px;
  padding: 20px;
  color: var(--text-muted, #b5bac1);
  font-size: 14px;
  width: 100%;
}

.loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-primary, #3f4147);
  border-top: 2px solid var(--discord-primary, #5865f2);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

.scroll-to-bottom-button {
  position: fixed;
  bottom: 100px;
  right: 20px;
  background: var(--discord-primary, #5865f2);
  color: white;
  border: none;
  border-radius: 16px;
  padding: 8px 16px;
  box-shadow: var(--shadow-medium, 0 4px 4px rgba(0, 0, 0, 0.16));
  cursor: pointer;
  z-index: 1000;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.15s ease;
}

.scroll-to-bottom-button:hover {
  background: var(--discord-primary-hover, #4752c4);
}
</style>
