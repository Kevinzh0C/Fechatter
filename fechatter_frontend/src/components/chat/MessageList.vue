<template>
  <!-- 🧹 CLEARED: MessageList display completely removed -->
  <div class="message-list" ref="scrollContainer">
    <!-- Loading History -->
    <div v-if="loadingHistory" class="loading-history">
      <div class="loading-text">Loading earlier messages...</div>
    </div>

    <!-- Messages Container -->
    <div class="messages-container" ref="messagesContainer">
      <!-- Loading State -->
      <div v-if="loading" class="loading-state">
        <div class="message-skeleton" v-for="n in 3" :key="n">
          <div class="skeleton-avatar"></div>
          <div class="skeleton-content">
            <div class="skeleton-line short"></div>
            <div class="skeleton-line long"></div>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else-if="!messages.length" class="empty-state">
        <h3>No messages yet</h3>
        <p>Start the conversation by sending a message!</p>
      </div>

      <!-- 🧹 CLEARED: MessageItem rendering removed -->
      <div v-else class="cleared-messages">
        <div class="cleared-notice">
          <h3>🧹 MessageList Display Cleared</h3>
          <p>All MessageItem components have been removed from MessageList.vue</p>
          <p>Messages count: {{ messages.length }}</p>
          <p>Chat ID: {{ chatId }}</p>
          <p>Awaiting new Discord MessageItem implementation...</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, nextTick, onMounted, onUnmounted, watch } from 'vue';

// 🧹 MINIMAL: Only essential props preserved
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
    required: true
  }
});

// 🧹 CLEARED: All message display logic removed
const emit = defineEmits(['user-profile-opened', 'dm-created', 'load-more-messages']);

// 🧹 MINIMAL: Basic refs only
const scrollContainer = ref(null);
const messagesContainer = ref(null);
const loadingHistory = ref(false);

// Console log for verification
console.log(`🧹 [MessageList] CLEARED - Chat ${props.chatId} message display removed`);
</script>

<style scoped>
.message-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow-y: auto;
  background: var(--bg-primary, #313338);
  padding: 16px;
}

.messages-container {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.cleared-messages {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
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

.loading-history {
  display: flex;
  justify-content: center;
  padding: 20px;
  color: var(--text-muted, #949ba4);
}

.loading-state {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 20px;
}

.message-skeleton {
  display: flex;
  gap: 12px;
  animation: pulse 1.5s ease-in-out infinite;
}

.skeleton-avatar {
  width: 40px;
  height: 40px;
  background: var(--bg-tertiary, #1e1f22);
  border-radius: 50%;
}

.skeleton-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-line {
  height: 16px;
  background: var(--bg-tertiary, #1e1f22);
  border-radius: 4px;
}

.skeleton-line.short {
  width: 30%;
}

.skeleton-line.long {
  width: 80%;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: var(--text-muted, #949ba4);
}

.empty-state h3 {
  margin-bottom: 8px;
  color: var(--text-secondary, #b5bac1);
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
</style>