<template>
  <!-- 🧹 CLEARED: MessageListContainer completely removed -->
  <div class="message-list-container-cleared">
    <div class="cleared-notice">
      <h3>🧹 MessageListContainer Display Cleared</h3>
      <p>PureMessageList component usage has been completely removed</p>
      <p>Chat ID: {{ chatId }}</p>
      <p>Messages Count: {{ messages?.length || 0 }}</p>
      <p>Awaiting new Discord MessageItem implementation...</p>
    </div>
  </div>
</template>

<script setup lang="ts">
// 🧹 CLEARED: All MessageList imports and functionality removed
// import PureMessageList from './PureMessageList.vue'

const props = defineProps<{
  chatId: string | number
  messages?: any[]
  currentUserId?: number
  loading?: boolean
}>()

// 🧹 CLEARED: All MessageList refs and logic removed
// const messageListRef = ref<InstanceType<typeof PureMessageList>>()

// Console log for verification
console.log(`🧹 [MessageListContainer] CLEARED - Chat ${props.chatId} display removed`);

// 🧹 CLEARED: All exposed methods removed
defineExpose({
  // scrollToBottom: () => console.log('🧹 Scroll functionality cleared'),
  // scrollToMessage: () => console.log('🧹 Scroll functionality cleared'),
  // getScrollContainer: () => null
});
</script>

<style scoped>
.message-list-container-cleared {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  width: 100%;
  background: var(--bg-primary, #313338);
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
</style> 