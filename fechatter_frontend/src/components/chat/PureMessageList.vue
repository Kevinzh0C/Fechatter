<template>
  <!-- 🧹 CLEARED: PureMessageList display completely removed -->
  <div ref="scrollContainer" class="pure-message-list">
    <!-- Loading Indicator for initial load -->
    <div v-if="loading && !hasMessages" class="loading-indicator">
      <div class="loading-text">Loading messages...</div>
    </div>

    <!-- 🧹 CLEARED: All message rendering removed -->
    <div class="cleared-message-display">
      <div class="cleared-notice">
        <h3>🧹 PureMessageList Display Cleared</h3>
        <p>All MessageItem components have been removed from PureMessageList.vue</p>
        <p>Messages count: {{ messages?.length || 0 }}</p>
        <p>Chat ID: {{ chatId }}</p>
        <p>Awaiting new Discord MessageItem implementation...</p>
      </div>
    </div>

    <!-- Empty State -->
    <div v-if="!loading && messages.length === 0" class="empty-state">
      <p>No messages yet. Start a conversation!</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue'
import type { Message, MessageListProps, MessageListEmits, ScrollPosition } from '@/types/message'

// 🧹 MINIMAL: Only essential props preserved
interface Props {
  messages: Message[]
  chatId: number
  currentUserId: number
  loading?: boolean
  hasMore?: boolean
  showDebugInfo?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  hasMore: false,
  showDebugInfo: false
})

// 🧹 CLEARED: All message display logic removed
const emit = defineEmits<{
  'user-profile-opened': [userId: number]
  'dm-created': [chatId: number]
  'load-more': []
  'auto-load-more': []
  'scroll-changed': [position: ScrollPosition]
  'message-displayed': [messageId: number]
}>()

// 🧹 MINIMAL: Basic refs only
const scrollContainer = ref<HTMLElement | null>(null)

const hasMessages = computed(() => props.messages.length > 0)

// Console log for verification
console.log(`🧹 [PureMessageList] CLEARED - Chat ${props.chatId} message display removed`);
</script>

<style scoped>
.pure-message-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow-y: auto;
  background: var(--bg-primary, #313338);
  padding: 16px;
}

.cleared-message-display {
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

.loading-indicator {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 40px 20px;
  color: var(--text-muted, #b5bac1);
  font-size: 14px;
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: var(--text-muted, #949ba4);
}
</style>