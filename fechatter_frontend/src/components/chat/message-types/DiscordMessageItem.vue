<template>
  <div class="discord-message-item" :class="{ 'is-reply': message.reply_to }">
    <!-- Avatar -->
    <div class="message-avatar" @click="openUserProfile">
      <Avatar :key="sender?.id" :src="sender?.avatar_url" :alt="sender?.fullname || 'Unknown User'"
        :user-id="sender?.id || 0" :status="sender?.status" :size="40" class="clickable-avatar" />
    </div>
    <!-- Message Content -->
    <div class="message-main">
      <div class="message-header">
        <span class="sender-name" @click="openUserProfile">{{ sender?.fullname || 'Unknown User' }}</span>
        <span class="message-timestamp">{{ formattedTimestamp }}</span>
      </div>
      <EnhancedMessageContent v-if="shouldRenderContent" :content="message.content" :enable-markdown="true"
        class="message-content" />
      <div v-if="message.files && message.files.length > 0" class="message-files">
        <FilePreview v-for="file in message.files" :key="file.id || file.url" :file="file"
          :workspace-id="workspaceId" />
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { marked } from 'marked';
import DOMPurify from 'dompurify';
import FilePreview from '@/components/chat/FilePreview.vue';
import Avatar from '@/components/common/Avatar.vue';
import EnhancedMessageContent from '@/components/chat/EnhancedMessageContent.vue';

const props = defineProps({
  message: {
    type: Object,
    required: true,
  },
  currentUserId: {
    type: Number,
    required: true,
  },
  chatId: {
    type: [Number, String],
    required: true,
  },
  workspaceId: {
    type: Number,
    required: false,
  },
});

const emit = defineEmits(['user-profile-opened']);

const sender = computed(() => props.message.sender);

const formattedTimestamp = computed(() => {
  if (!props.message.created_at) return '';
  const date = new Date(props.message.created_at);
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
});

const shouldRenderContent = computed(() => {
  // Don't render content if the message only contains the file placeholder
  if (props.message.content === '📎' && props.message.files?.length > 0) {
    return false;
  }
  // Render content if there's any meaningful content
  return props.message.content && props.message.content.trim();
});

const renderedContent = computed(() => {
  // If the message only contains the file placeholder, don't render it
  if (props.message.content === '📎' && props.message.files?.length > 0) {
    return '';
  }
  const dirty = marked.parse(props.message.content || '');
  return DOMPurify.sanitize(dirty);
});

const openUserProfile = () => {
  if (sender.value) {
    emit('user-profile-opened', sender.value);
  }
};
</script>

<style scoped>
/* Scoped styles for DiscordMessageItem */
.discord-message-item {
  display: flex;
  padding: 13px 16px;
  transition: all 0.2s ease;
  position: relative;
  border-bottom: 1px solid transparent;
}

/* Subtle message separation */
.discord-message-item::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 60px;
  /* Start after avatar area */
  right: 16px;
  height: 1px;
  background: linear-gradient(90deg, rgba(0, 0, 0, 0.04) 0%, rgba(0, 0, 0, 0.08) 50%, rgba(0, 0, 0, 0.04) 100%);
  opacity: 0.6;
}

.discord-message-item:hover {
  /* Subtle background for better reading focus */
  background-color: rgba(124, 58, 237, 0.015);
  border-radius: 8px;
  /* Remove transform that causes jitter */
  transform: none;
  /* Subtle shadow without movement */
  box-shadow: 0 0 0 1px rgba(124, 58, 237, 0.08);
}

.discord-message-item:hover::after {
  opacity: 0;
}

/* Remove separator from last message */
.discord-message-item:last-child::after {
  display: none;
}

.message-avatar {
  margin-right: 13px;
}

.clickable-avatar {
  cursor: pointer;
  transition: transform 0.2s ease;
}

.clickable-avatar:hover {
  transform: scale(1.05);
}

.message-main {
  display: flex;
  flex-direction: column;
  flex-grow: 1;
}

.message-header {
  display: flex;
  align-items: baseline;
  margin-bottom: 5px;
}

.sender-name {
  font-weight: 500;
  color: #060607;
  margin-right: 8px;
  cursor: pointer;
  transition: color 0.2s ease;
}

.sender-name:hover {
  text-decoration: underline;
  color: #0969da;
}

.message-timestamp {
  font-size: 0.75rem;
  color: #72767d;
  transition: color 0.2s ease;
}

.discord-message-item:hover .message-timestamp {
  color: #8c959f;
}

.message-content {
  color: #2e3338;
  word-wrap: break-word;
  white-space: pre-wrap;
}

.message-files {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* Dark mode message separation */
@media (prefers-color-scheme: dark) {
  .discord-message-item::after {
    background: linear-gradient(90deg, rgba(255, 255, 255, 0.06) 0%, rgba(255, 255, 255, 0.12) 50%, rgba(255, 255, 255, 0.06) 100%);
    opacity: 0.4;
  }

  .discord-message-item:hover {
    background-color: rgba(255, 255, 255, 0.03);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }

  .sender-name {
    color: #e6edf3;
  }

  .sender-name:hover {
    color: #79c0ff;
  }

  .message-timestamp {
    color: #8b949e;
  }

  .discord-message-item:hover .message-timestamp {
    color: #a8b1bb;
  }

  .message-content {
    color: #e6edf3;
  }
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .discord-message-item {
    padding: 10px 12px;
  }

  .discord-message-item::after {
    left: 50px;
    right: 12px;
  }

  .discord-message-item:hover {
    transform: none;
    /* Disable slide effect on mobile */
  }
}
</style>