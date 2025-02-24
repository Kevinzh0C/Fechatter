<template>
  <div ref="scrollContainer" class="simple-message-list" @scroll="handleScroll">
    <!-- 🔧 SIMPLIFIED: Loading indicator without complex transitions -->
    <div v-if="isLoading" class="loading-indicator">
      <div class="spinner"></div>
      <span>Loading messages...</span>
    </div>

    <!-- Messages wrapper -->
    <div class="messages-wrapper">
      <div v-for="message in props.messages" :key="message.id" class="message-wrapper">
        <DiscordMessageItem :message="message" :current-user-id="currentUserId" />
      </div>
    </div>

    <!-- 🔧 FIXED: Simple scroll to bottom button -->
    <Transition name="fade">
      <button v-if="showScrollButton" @click="scrollToBottom" class="scroll-to-bottom-btn">
        ↓
      </button>
    </Transition>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { useAuthStore } from '@/stores/auth';
import DiscordMessageItem from '@/components/discord/DiscordMessageItem.vue';

// Props
const props = defineProps({
  messages: {
    type: Array,
    default: () => []
  },
  chatId: {
    type: [Number, String],
    required: true
  },
  loading: {
    type: Boolean,
    default: false
  },
  hasMoreMessages: {
    type: Boolean,
    default: true
  }
});

// Emits
const emit = defineEmits(['load-more']);

// Refs
const scrollContainer = ref(null);
const authStore = useAuthStore();

// State
const isLoading = ref(false);
const showScrollButton = ref(false);
const isAutoScrollEnabled = ref(true);
const lastScrollTop = ref(0);

// 🔧 FIX: Debounce mechanism to prevent excessive calls
const scrollDebounceTimer = ref(null);
const loadMoreDebounceTimer = ref(null);
const scrollButtonDebounceTimer = ref(null);

// Computed
const currentUserId = computed(() => authStore.user?.id);

// 🔧 FIXED: Simplified scroll handler with debouncing
const handleScroll = () => {
  if (!scrollContainer.value) return;

  // Clear existing timer
  if (scrollDebounceTimer.value) {
    clearTimeout(scrollDebounceTimer.value);
  }

  // Debounce scroll handling
  scrollDebounceTimer.value = setTimeout(() => {
    const container = scrollContainer.value;
    if (!container) return;

    const { scrollTop, scrollHeight, clientHeight } = container;

    // 🔧 Check if user scrolled up (show scroll button)
    if (scrollButtonDebounceTimer.value) {
      clearTimeout(scrollButtonDebounceTimer.value);
    }

    scrollButtonDebounceTimer.value = setTimeout(() => {
      const isAtBottom = scrollTop + clientHeight >= scrollHeight - 100;
      showScrollButton.value = !isAtBottom && props.messages.length > 5;
    }, 100);

    // 🔧 Load more messages when scrolled to top
    if (scrollTop < 100 && props.hasMoreMessages && !isLoading.value) {
      handleLoadMore();
    }

    lastScrollTop.value = scrollTop;
  }, 50); // 50ms debounce
};

// 🔧 FIXED: Promise-based load more (no event listening)
const handleLoadMore = () => {
  if (isLoading.value || !props.hasMoreMessages) return;

  // Clear existing timer
  if (loadMoreDebounceTimer.value) {
    clearTimeout(loadMoreDebounceTimer.value);
  }

  // Debounce load more calls
  loadMoreDebounceTimer.value = setTimeout(() => {
    isLoading.value = true;

    // Store current scroll position for restoration
    const container = scrollContainer.value;
    const currentScrollHeight = container ? container.scrollHeight : 0;

    // Emit load more event and handle Promise completion
    const loadPromise = emit('load-more');

    // 🔧 FIXED: Handle Promise completion without events
    if (loadPromise && typeof loadPromise.then === 'function') {
      loadPromise
        .then(() => {
          // Restore scroll position after new messages loaded
          nextTick(() => {
            if (container) {
              const newScrollHeight = container.scrollHeight;
              const scrollDiff = newScrollHeight - currentScrollHeight;
              container.scrollTop = lastScrollTop.value + scrollDiff;
            }
          });
        })
        .catch(error => {
          console.error('Load more failed:', error);
        })
        .finally(() => {
          isLoading.value = false;
        });
    } else {
      // Fallback if no Promise returned
      setTimeout(() => {
        isLoading.value = false;
      }, 1000);
    }
  }, 300); // 300ms debounce for load more
};

// 🔧 Simple scroll to bottom
const scrollToBottom = (smooth = true) => {
  if (!scrollContainer.value) return;

  const container = scrollContainer.value;
  const scrollOptions = {
    top: container.scrollHeight,
    behavior: smooth ? 'smooth' : 'instant'
  };

  container.scrollTo(scrollOptions);
  showScrollButton.value = false;
};

// 🔧 Watch for new messages and auto-scroll if at bottom
watch(() => props.messages?.length, (newLength, oldLength) => {
  if (!newLength || newLength <= (oldLength || 0)) return;

  nextTick(() => {
    if (!scrollContainer.value || !isAutoScrollEnabled.value) return;

    const container = scrollContainer.value;
    const { scrollTop, scrollHeight, clientHeight } = container;
    const isNearBottom = scrollTop + clientHeight >= scrollHeight - 150;

    // Auto-scroll only if user was near bottom
    if (isNearBottom) {
      scrollToBottom(true);
    }
  });
});

// 🔧 Watch chat changes and reset state
watch(() => props.chatId, (newChatId, oldChatId) => {
  if (newChatId !== oldChatId) {
    // Reset state for new chat
    isLoading.value = false;
    showScrollButton.value = false;
    isAutoScrollEnabled.value = true;

    // Clear all timers
    [scrollDebounceTimer, loadMoreDebounceTimer, scrollButtonDebounceTimer].forEach(timer => {
      if (timer.value) {
        clearTimeout(timer.value);
        timer.value = null;
      }
    });

    // Scroll to bottom for new chat
    nextTick(() => {
      scrollToBottom(false);
    });
  }
});

// 🔧 Component lifecycle
onMounted(() => {
  // Initial scroll to bottom if messages exist
  if (props.messages && props.messages.length > 0) {
    nextTick(() => {
      scrollToBottom(false);
    });
  }
});

onUnmounted(() => {
  // 🔧 CRITICAL: Clear all timers to prevent memory leaks
  [scrollDebounceTimer, loadMoreDebounceTimer, scrollButtonDebounceTimer].forEach(timer => {
    if (timer.value) {
      clearTimeout(timer.value);
      timer.value = null;
    }
  });
});

// Expose methods
defineExpose({
  scrollToBottom,
  isLoading: () => isLoading.value
});
</script>

<style scoped>
.simple-message-list {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  background: #fdfcfa;
  position: relative;
  scroll-behavior: smooth;
}

.loading-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 16px;
  color: #6b7280;
  font-size: 14px;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid #e5e7eb;
  border-top-color: #6366f1;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.messages-wrapper {
  width: 100%;
  max-width: 900px;
  margin: 0 auto;
  padding: 1rem 2rem;
  min-height: 100vh;
}

.message-wrapper {
  margin-bottom: 8px;
  transition: background-color 0.15s ease;
  border-radius: 8px;
  padding: 4px 0;
}

.message-wrapper:hover {
  background-color: rgba(124, 58, 237, 0.02);
}

.scroll-to-bottom-btn {
  position: fixed;
  bottom: 24px;
  right: 24px;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid #e5e7eb;
  border-radius: 50%;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  transition: all 0.2s ease;
  z-index: 100;
  color: #6b7280;
  font-size: 18px;
  font-weight: bold;
}

.scroll-to-bottom-btn:hover {
  background: rgba(255, 255, 255, 1);
  border-color: #6366f1;
  color: #6366f1;
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.15);
}

/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: all 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(20px) scale(0.8);
}

/* Scrollbar styling */
.simple-message-list::-webkit-scrollbar {
  width: 6px;
}

.simple-message-list::-webkit-scrollbar-track {
  background: transparent;
}

.simple-message-list::-webkit-scrollbar-thumb {
  background: rgba(124, 58, 237, 0.2);
  border-radius: 3px;
}

.simple-message-list::-webkit-scrollbar-thumb:hover {
  background: rgba(124, 58, 237, 0.35);
}

/* Responsive */
@media (max-width: 768px) {
  .messages-wrapper {
    padding: 1rem;
    max-width: 100%;
  }

  .scroll-to-bottom-btn {
    bottom: 16px;
    right: 16px;
    width: 40px;
    height: 40px;
    font-size: 16px;
  }
}

/* Accessibility */
@media (prefers-reduced-motion: reduce) {
  .simple-message-list {
    scroll-behavior: auto;
  }

  .message-wrapper {
    transition: none;
  }

  .fade-enter-active,
  .fade-leave-active {
    transition: none;
  }
}
</style>