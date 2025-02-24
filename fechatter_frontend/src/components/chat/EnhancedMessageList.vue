<template>
  <div class="flex-1 overflow-hidden bg-gray-50">
    <!-- Message container -->
    <div ref="messageContainer" class="h-full overflow-y-auto scroll-smooth" @scroll="handleScroll">
      <!-- Loading indicator -->
      <div v-if="loading" class="flex justify-center py-4">
        <div class="flex items-center space-x-2 text-gray-500">
          <div class="w-4 h-4 border-2 border-gray-300 border-t-violet-600 rounded-full animate-spin"></div>
          <span class="text-sm">Loading messages...</span>
        </div>
      </div>

      <!-- Load more button -->
      <div v-else-if="hasMore" class="flex justify-center py-4">
        <button @click="loadMoreMessages"
          class="px-4 py-2 text-sm text-violet-600 hover:text-violet-700 hover:bg-violet-50 rounded-lg transition-colors">
          Load earlier messages
        </button>
      </div>

      <!-- Message list -->
      <div class="max-w-3xl mx-auto px-6">
        <div v-for="(message, index) in messages" :key="message.id" class="mb-1">
          <!-- Date separator -->
          <div v-if="shouldShowDateSeparator(message, index)" class="flex items-center justify-center my-6">
            <div class="bg-white rounded-full px-4 py-1 text-xs text-gray-500 border border-gray-200 shadow-sm">
              {{ formatDate(message.created_at) }}
            </div>
          </div>

          <!-- Message item -->
          <EnhancedMessageItem :message="message" :show-avatar="shouldShowAvatar(message, index)"
            :show-timestamp="shouldShowTimestamp(message, index)" :is-editing="editingMessageId === message.id"
            :search-query="searchQuery" @edit="handleEditMessage" @delete="handleDeleteMessage"
            @reply="handleReplyMessage" @react="handleReactMessage" @mention-click="handleMentionClick" />
        </div>

        <!-- New messages indicator -->
        <div v-if="unreadCount > 0" class="flex justify-center my-4">
          <div class="bg-red-500 text-white text-xs px-3 py-1 rounded-full">
            {{ unreadCount }} new message(s)
          </div>
        </div>

        <!-- Typing indicator -->
        <div v-if="typingText" class="py-2">
          <div class="flex items-center space-x-2 text-sm text-gray-500">
            <div class="flex space-x-1">
              <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 0ms;"></div>
              <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 150ms;"></div>
              <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 300ms;"></div>
            </div>
            <span>{{ typingText }}</span>
          </div>
        </div>

        <!-- Bottom spacing -->
        <div class="h-4"></div>
      </div>
    </div>

    <!-- Scroll to bottom button -->
    <Transition enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0 transform translate-y-1" enter-to-class="opacity-100 transform translate-y-0"
      leave-active-class="transition ease-in duration-150" leave-from-class="opacity-100 transform translate-y-0"
      leave-to-class="opacity-0 transform translate-y-1">
      <div v-if="showScrollToBottom" class="absolute bottom-6 right-6">
        <button @click="scrollToBottom"
          class="bg-violet-600 text-white p-3 rounded-full shadow-lg hover:bg-violet-700 hover:shadow-xl transition-all duration-200 transform hover:scale-105">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3"></path>
          </svg>
          <div v-if="unreadCount > 0"
            class="absolute -top-2 -right-2 bg-red-500 text-white text-xs rounded-full min-w-[20px] h-5 flex items-center justify-center">
            {{ unreadCount > 99 ? '99+' : unreadCount }}
          </div>
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import EnhancedMessageItem from './EnhancedMessageItem.vue';

// Props
const props = defineProps({
  chatId: {
    type: Number,
    required: true
  },
  searchQuery: {
    type: String,
    default: ''
  }
});

// Emits
const emit = defineEmits(['edit-message', 'reply-message', 'mention-click']);

// Reactive state
const messageContainer = ref(null);
const showScrollToBottom = ref(false);
const isAutoScrolling = ref(false);
const lastScrollTop = ref(0);
const messages = ref([]);
const loading = ref(false);
const hasMore = ref(true);
const unreadCount = ref(0);
const typingText = ref('');
const editingMessageId = ref(null);

// Message grouping logic
const shouldShowDateSeparator = (message, index) => {
  if (index === 0) return true;

  const prevMessage = messages.value[index - 1];
  if (!prevMessage) return true;

  const currentDate = new Date(message.created_at).toDateString();
  const prevDate = new Date(prevMessage.created_at).toDateString();

  return currentDate !== prevDate;
};

const shouldShowAvatar = (message, index) => {
  if (index === 0) return true;

  const prevMessage = messages.value[index - 1];
  if (!prevMessage) return true;

  // Show avatar if different sender
  if (prevMessage.sender_id !== message.sender_id) return true;

  // Show avatar if more than 5 minutes apart
  const timeDiff = new Date(message.created_at) - new Date(prevMessage.created_at);
  return timeDiff > 5 * 60 * 1000;
};

const shouldShowTimestamp = (message, index) => {
  return shouldShowAvatar(message, index);
};

// Format functions
const formatDate = (dateString) => {
  const date = new Date(dateString);
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);

  if (date.toDateString() === today.toDateString()) {
    return 'Today';
  } else if (date.toDateString() === yesterday.toDateString()) {
    return 'Yesterday';
  } else {
    return date.toLocaleDateString('en-US', {
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  }
};

// Scroll management
const handleScroll = () => {
  const container = messageContainer.value;
  if (!container) return;

  const { scrollTop, scrollHeight, clientHeight } = container;
  const isNearBottom = scrollHeight - scrollTop - clientHeight < 100;

  showScrollToBottom.value = !isNearBottom;

  if (scrollTop < 100 && !loading.value && hasMore.value) {
    loadMoreMessages();
  }

  if (isNearBottom && !isAutoScrolling.value) {
    markVisibleMessagesAsRead();
  }

  lastScrollTop.value = scrollTop;
};

const scrollToBottom = async (smooth = true) => {
  await nextTick();
  const container = messageContainer.value;
  if (!container) return;

  isAutoScrolling.value = true;

  container.scrollTo({
    top: container.scrollHeight,
    behavior: smooth ? 'smooth' : 'instant'
  });

  setTimeout(() => {
    isAutoScrolling.value = false;
    markVisibleMessagesAsRead();
  }, 500);
};

// Message operations
const loadMoreMessages = async () => {
  if (loading.value) return;

  loading.value = true;
  try {
    // TODO: Implement message loading
    console.log('Loading more messages for chat:', props.chatId);
  } catch (error) {
    console.error('Failed to load more messages:', error);
  } finally {
    loading.value = false;
  }
};

const handleEditMessage = (message) => {
  editingMessageId.value = message.id;
  emit('edit-message', message);
};

const handleDeleteMessage = async (message) => {
  if (!confirm('Are you sure you want to delete this message?')) return;

  try {
    // TODO: Implement message deletion
    console.log('Deleting message:', message.id);
  } catch (error) {
    console.error('Failed to delete message:', error);
  }
};

const handleReplyMessage = (message) => {
  emit('reply-message', message);
};

const handleReactMessage = async (message, emoji) => {
  console.log('React to message:', message.id, 'with emoji:', emoji);
};

const handleMentionClick = (userId) => {
  emit('mention-click', userId);
};

const markVisibleMessagesAsRead = async () => {
  const container = messageContainer.value;
  if (!container) return;

  const visibleMessages = [];
  const messageElements = container.querySelectorAll('[data-message-id]');

  messageElements.forEach(element => {
    const rect = element.getBoundingClientRect();
    const containerRect = container.getBoundingClientRect();

    if (rect.top >= containerRect.top && rect.bottom <= containerRect.bottom) {
      const messageId = parseInt(element.dataset.messageId);
      if (messageId) {
        visibleMessages.push(messageId);
      }
    }
  });

  if (visibleMessages.length > 0) {
    try {
      // TODO: Mark messages as read
      console.log('Marking messages as read:', visibleMessages);
      unreadCount.value = Math.max(0, unreadCount.value - visibleMessages.length);
    } catch (error) {
      console.warn('Failed to mark messages as read:', error);
    }
  }
};

// Lifecycle
watch(() => messages.value.length, async (newLength, oldLength) => {
  if (newLength > oldLength) {
    await nextTick();

    const container = messageContainer.value;
    if (!container) return;

    const isNearBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 200;

    if (isNearBottom || !oldLength) {
      scrollToBottom(true);
    }
  }
});

watch(() => props.chatId, async () => {
  showScrollToBottom.value = false;
  messages.value = [];
  await nextTick();
  scrollToBottom(false);
}, { immediate: true });

onMounted(async () => {
  try {
    await loadMoreMessages();
    await nextTick();
    scrollToBottom(false);
  } catch (error) {
    console.error('Initial load failed:', error);
  }
});

// Expose methods
defineExpose({
  scrollToBottom,
  loadMoreMessages,
  markVisibleMessagesAsRead
});
</script>

<style scoped>
/* Custom scrollbar */
.overflow-y-auto::-webkit-scrollbar {
  width: 8px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: transparent;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: rgba(156, 163, 175, 0.3);
  border-radius: 4px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: rgba(156, 163, 175, 0.5);
}

/* Smooth scrolling */
.scroll-smooth {
  scroll-behavior: smooth;
}
</style>