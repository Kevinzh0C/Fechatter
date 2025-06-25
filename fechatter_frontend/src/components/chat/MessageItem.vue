<template>
  <DiscordMessageItem
    :message="message"
    :current-user-id="currentUserId"
    :chat-id="chatId"
    :is-highlighted="isHighlighted"
    :is-compact="isCompact"
    :show-user-status="showUserStatus"
    @user-profile-opened="handleUserProfileOpened"
    @dm-created="handleDMCreated"
    @reply="handleReply"
    @scroll-to-message="handleScrollToMessage"
    @context-menu="handleContextMenu"
  />
</template>

<script setup>
import DiscordMessageItem from '../discord/DiscordMessageItem.vue';

const props = defineProps({
  message: {
    type: Object,
    required: true
  },
  currentUserId: {
    type: Number,
    required: false,
    default: 0
  },
  chatId: {
    type: [Number, String],
    default: null
  },
  isHighlighted: {
    type: Boolean,
    default: false
  },
  isCompact: {
    type: Boolean,
    default: false
  },
  showUserStatus: {
    type: Boolean,
    default: true
  }
});

const emit = defineEmits([
  'user-profile-opened', 
  'dm-created', 
  'reply',
  'scroll-to-message',
  'context-menu'
]);

// 事件转发
const handleUserProfileOpened = (user) => {
  emit('user-profile-opened', user);
};

const handleDMCreated = (dm) => {
  emit('dm-created', dm);
};

const handleReply = (message) => {
  emit('reply', message);
};

const handleScrollToMessage = (messageId) => {
  emit('scroll-to-message', messageId);
};

const handleContextMenu = (data) => {
  emit('context-menu', data);
};
</script>

<style scoped>
/* 这个组件现在只是一个包装器，所有样式都在DiscordMessageItem中 */
</style> 