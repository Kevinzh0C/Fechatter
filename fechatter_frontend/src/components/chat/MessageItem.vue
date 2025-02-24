<template>
  <!-- 🧹 CLEARED: MessageItem display completely removed -->
  <div class="message-item-placeholder" :data-message-id="message.id">
    <div class="cleared-notice">
      MessageItem display cleared - awaiting new Discord MessageItem implementation
    </div>
  </div>
</template>

<script setup>
// 🧹 MINIMAL: Only essential props preserved
const props = defineProps({
  message: {
    type: Object,
    required: true
  },
  currentUserId: {
    type: Number,
    default: null
  },
  chatId: {
    type: [Number, String],
    default: null
  }
});

// 🧹 CLEARED: All display logic removed
const emit = defineEmits(['user-profile-opened', 'dm-created']);

// Console log for verification
console.log(`🧹 [MessageItem] CLEARED - Message ${props.message.id} display removed`);
</script>

<style scoped>
.message-item-placeholder {
  padding: 12px 16px;
  margin: 8px 0;
  background: var(--bg-tertiary, #2b2d31);
  border: 2px dashed var(--border-primary, #3e4146);
  border-radius: 8px;
  text-align: center;
  opacity: 0.7;
}

.cleared-notice {
  color: var(--text-muted, #949ba4);
  font-style: italic;
  font-size: 14px;
}
</style>