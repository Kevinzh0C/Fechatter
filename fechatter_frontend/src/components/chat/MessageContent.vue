<template>
  <div class="message-content">
    <DiscordMarkdown :content="content" />
  </div>
</template>

<script setup>
import DiscordMarkdown from '../discord/DiscordMarkdown.vue';

defineProps({
  content: {
    type: String,
    required: true,
  },
});
</script>

<style scoped>
.message-content {
  /* Add any specific styling for message content here */
  line-height: 1.5;
}
</style>