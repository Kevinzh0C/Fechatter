<template>
  <!-- 🧹 CLEARED: MessageItemOptimized display completely removed -->
  <div class="optimized-message-placeholder" :data-message-id="message.id">
    <div class="cleared-notice">
      MessageItemOptimized display cleared - awaiting new Discord MessageItem implementation
    </div>
  </div>
</template>

<script setup>
// 🧹 MINIMAL: Only essential props preserved
const props = defineProps({
  message: {
    type: Object,
    required: true
  }
});

// 🧹 CLEARED: All display logic removed
const emit = defineEmits(['reply', 'edit', 'delete', 'reaction']);

// Console log for verification
console.log(`🧹 [MessageItemOptimized] CLEARED - Message ${props.message.id} display removed`);
</script>

<style scoped>
.optimized-message-placeholder {
  padding: 16px 20px;
  margin: 12px 0;
  background: var(--bg-secondary, #2b2d31);
  border: 2px dashed var(--text-warning, #fbbf24);
  border-radius: 12px;
  text-align: center;
  opacity: 0.8;
}

.cleared-notice {
  color: var(--text-warning, #fbbf24);
  font-style: italic;
  font-size: 14px;
  font-weight: 500;
}
</style>