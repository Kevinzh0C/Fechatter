<template>
  <!-- 🧹 CLEARED: EnhancedChatView completely removed -->
  <div class="enhanced-chat-view-cleared">
    <div class="cleared-notice">
      <h3>🧹 EnhancedChatView Display Cleared</h3>
      <p>VirtualMessageList component usage has been completely removed</p>
      <p>Chat ID: {{ chatId }}</p>
      <p>Messages Count: {{ messages?.length || 0 }}</p>
      <p>Awaiting new Discord MessageItem implementation...</p>
    </div>
  </div>
</template>

<script setup>
// 🧹 CLEARED: All VirtualMessageList imports and functionality removed
// import VirtualMessageList from './VirtualMessageList.vue';

const props = defineProps({
  chatId: {
    type: [Number, String],
    required: true
  },
  messages: {
    type: Array,
    default: () => []
  },
  currentUserId: {
    type: Number,
    default: null
  }
});

// 🧹 CLEARED: All message list refs and logic removed
// const messageList = ref(null);

// Console log for verification
console.log(`🧹 [EnhancedChatView] CLEARED - Chat ${props.chatId} virtual message display removed`);

// 🧹 CLEARED: All exposed methods removed
defineExpose({
  // scrollToBottom: () => console.log('🧹 Scroll functionality cleared'),
  // scrollToMessage: () => console.log('🧹 Scroll functionality cleared')
});
</script>

<style scoped>
.enhanced-chat-view-cleared {
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