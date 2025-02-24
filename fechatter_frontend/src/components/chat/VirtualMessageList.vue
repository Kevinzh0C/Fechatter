<template>
  <!-- 🧹 CLEARED: VirtualMessageList display completely removed -->
  <div class="virtual-message-list" ref="scrollContainer" role="log" aria-label="Chat messages">
    
    <!-- 🧹 CLEARED: All virtual scrolling and message rendering removed -->
    <div class="cleared-message-display">
      <div class="cleared-notice">
        <h3>🧹 VirtualMessageList Display Cleared</h3>
        <p>All MessageItem components and virtual scrolling have been removed</p>
        <p>Messages count: {{ messages?.length || 0 }}</p>
        <p>Chat ID: {{ chatId }}</p>
        <p>Awaiting new Discord MessageItem implementation...</p>
      </div>
    </div>

    <!-- Loading indicator -->
    <div v-if="loadingHistory" class="loading-history-container">
      <div class="loading-history-fallback">
        <div class="loading-dots">
          <div></div>
          <div></div>
          <div></div>
        </div>
        <span>Loading older messages...</span>
      </div>
    </div>

  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, toRefs, nextTick, onUnmounted } from 'vue';

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
  chatId: {
    type: [Number, String],
    required: true
  },
  workspaceId: {
    type: Number,
    required: false
  },
  loadingHistory: {
    type: Boolean,
    default: false
  }
});

// 🧹 CLEARED: All message display logic removed
const emit = defineEmits(['user-profile-opened', 'dm-created', 'load-more-messages']);

// 🧹 MINIMAL: Basic refs only
const scrollContainer = ref(null);

// Console log for verification
console.log(`🧹 [VirtualMessageList] CLEARED - Chat ${props.chatId} virtual message display removed`);
</script>

<style scoped>
.virtual-message-list {
  height: 100%;
  overflow-y: auto;
  position: relative;
  background: var(--bg-primary, #313338);
  display: flex;
  flex-direction: column;
}

.cleared-message-display {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
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

.loading-history-container {
  padding: 20px;
  display: flex;
  justify-content: center;
}

.loading-history-fallback {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--text-muted, #949ba4);
  font-size: 14px;
}

.loading-dots {
  display: flex;
  gap: 4px;
}

.loading-dots div {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-muted, #949ba4);
  animation: pulse 1.5s ease-in-out infinite;
}

.loading-dots div:nth-child(1) { animation-delay: 0s; }
.loading-dots div:nth-child(2) { animation-delay: 0.5s; }
.loading-dots div:nth-child(3) { animation-delay: 1s; }

@keyframes pulse {
  0%, 100% { opacity: 0.3; }
  50% { opacity: 1; }
}
</style>