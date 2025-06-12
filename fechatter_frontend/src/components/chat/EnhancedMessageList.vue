<template>
  <div class="flex-1 overflow-hidden bg-gray-50">
    <!-- 消息列表容器 -->
    <div 
      ref="messageContainer"
      class="h-full overflow-y-auto scroll-smooth"
      @scroll="handleScroll"
    >
      <!-- 加载更多指示器 -->
      <div v-if="messageSystem.state.loading" class="flex justify-center py-4">
        <div class="flex items-center space-x-2 text-gray-500">
          <div class="w-4 h-4 border-2 border-gray-300 border-t-violet-600 rounded-full animate-spin"></div>
          <span class="text-sm">Loading messages...</span>
        </div>
      </div>

      <!-- 历史消息加载按钮 -->
      <div v-else-if="messageSystem.state.hasMore" class="flex justify-center py-4">
        <button 
          @click="loadMoreMessages"
          class="px-4 py-2 text-sm text-violet-600 hover:text-violet-700 hover:bg-violet-50 rounded-lg transition-colors"
        >
          Load earlier messages
        </button>
      </div>

      <!-- 消息列表 -->
      <div class="max-w-3xl mx-auto px-6">
        <div v-for="(message, index) in groupedMessages" :key="message.id" class="mb-1">
          
          <!-- 日期分隔符 -->
          <div v-if="shouldShowDateSeparator(message, index)" class="flex items-center justify-center my-6">
            <div class="bg-white rounded-full px-4 py-1 text-xs text-gray-500 border border-gray-200 shadow-sm">
              {{ formatDate(message.created_at) }}
            </div>
          </div>

          <!-- 消息项 -->
          <EnhancedMessageItem 
            :message="message"
            :show-avatar="shouldShowAvatar(message, index)"
            :show-timestamp="shouldShowTimestamp(message, index)"
            :is-editing="messageSystem.state.editing.messageId === message.id"
            :search-query="searchQuery"
            @edit="handleEditMessage"
            @delete="handleDeleteMessage"
            @reply="handleReplyMessage"
            @react="handleReactMessage"
            @mention-click="handleMentionClick"
          />

        </div>

        <!-- 新消息指示器 -->
        <div v-if="messageSystem.hasUnreadMessages" class="flex justify-center my-4">
          <div class="bg-red-500 text-white text-xs px-3 py-1 rounded-full">
            {{ messageSystem.state.unreadCount }} new message(s)
          </div>
        </div>

        <!-- 正在输入指示器 -->
        <div v-if="messageSystem.isTypingDisplayText" class="py-2">
          <div class="flex items-center space-x-2 text-sm text-gray-500">
            <div class="flex space-x-1">
              <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 0ms;"></div>
              <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 150ms;"></div>
              <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 300ms;"></div>
            </div>
            <span>{{ messageSystem.isTypingDisplayText }}</span>
          </div>
        </div>

        <!-- 底部间距 -->
        <div class="h-4"></div>
      </div>
    </div>

    <!-- 滚动到底部按钮 -->
    <Transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0 transform translate-y-1"
      enter-to-class="opacity-100 transform translate-y-0"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100 transform translate-y-0"
      leave-to-class="opacity-0 transform translate-y-1"
    >
      <div v-if="showScrollToBottom" class="absolute bottom-6 right-6">
        <button 
          @click="scrollToBottom"
          class="bg-violet-600 text-white p-3 rounded-full shadow-lg hover:bg-violet-700 hover:shadow-xl transition-all duration-200 transform hover:scale-105"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3"></path>
          </svg>
          <div v-if="messageSystem.state.unreadCount > 0" class="absolute -top-2 -right-2 bg-red-500 text-white text-xs rounded-full min-w-[20px] h-5 flex items-center justify-center">
            {{ messageSystem.state.unreadCount > 99 ? '99+' : messageSystem.state.unreadCount }}
          </div>
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import { useMessageSystem } from '@/composables/useMessageSystem';
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

// ================================
// 消息系统集成
// ================================

const messageSystem = useMessageSystem(props.chatId);

// ================================
// 响应式状态
// ================================

const messageContainer = ref(null);
const showScrollToBottom = ref(false);
const isAutoScrolling = ref(false);
const lastScrollTop = ref(0);

// ================================
// 计算属性
// ================================

const groupedMessages = computed(() => {
  return messageSystem.sortedMessages;
});

// ================================
// 消息分组和显示逻辑
// ================================

const shouldShowDateSeparator = (message, index) => {
  if (index === 0) return true;
  
  const prevMessage = groupedMessages.value[index - 1];
  if (!prevMessage) return true;
  
  const currentDate = new Date(message.created_at).toDateString();
  const prevDate = new Date(prevMessage.created_at).toDateString();
  
  return currentDate !== prevDate;
};

const shouldShowAvatar = (message, index) => {
  if (index === 0) return true;
  
  const prevMessage = groupedMessages.value[index - 1];
  if (!prevMessage) return true;
  
  // 如果发送者不同，显示头像
  if (prevMessage.sender_id !== message.sender_id) return true;
  
  // 如果时间间隔超过5分钟，显示头像
  const timeDiff = new Date(message.created_at) - new Date(prevMessage.created_at);
  return timeDiff > 5 * 60 * 1000; // 5分钟
};

const shouldShowTimestamp = (message, index) => {
  return shouldShowAvatar(message, index);
};

// ================================
// 格式化函数
// ================================

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

// ================================
// 滚动管理
// ================================

const handleScroll = () => {
  const container = messageContainer.value;
  if (!container) return;
  
  const { scrollTop, scrollHeight, clientHeight } = container;
  const isNearBottom = scrollHeight - scrollTop - clientHeight < 100;
  
  // 显示/隐藏滚动到底部按钮
  showScrollToBottom.value = !isNearBottom;
  
  // 如果用户滚动到顶部附近，加载更多消息
  if (scrollTop < 100 && !messageSystem.state.loading && messageSystem.state.hasMore) {
    loadMoreMessages();
  }
  
  // 自动标记消息为已读
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

// ================================
// 消息加载
// ================================

const loadMoreMessages = async () => {
  if (messageSystem.state.loading) return;
  
  const container = messageContainer.value;
  const oldScrollHeight = container?.scrollHeight || 0;
  
  try {
    await messageSystem.loadMoreMessages();
    
    // 保持滚动位置
    await nextTick();
    if (container) {
      const newScrollHeight = container.scrollHeight;
      container.scrollTop = newScrollHeight - oldScrollHeight + lastScrollTop.value;
    }
  } catch (error) {
    console.error('❌ [EnhancedMessageList] Load more messages failed:', error);
  }
};

// ================================
// 消息操作
// ================================

const handleEditMessage = (message) => {
  emit('edit-message', message);
};

const handleDeleteMessage = async (message) => {
  if (!confirm('Are you sure you want to delete this message?')) return;
  
  try {
    await messageSystem.deleteMessage(message.id);
  } catch (error) {
    console.error('❌ [EnhancedMessageList] Delete message failed:', error);
    // TODO: 显示错误提示
  }
};

const handleReplyMessage = (message) => {
  emit('reply-message', message);
};

const handleReactMessage = async (message, emoji) => {
  // TODO: 实现消息反应功能
  console.log('React to message:', message.id, 'with emoji:', emoji);
};

const handleMentionClick = (userId) => {
  emit('mention-click', userId);
};

// ================================
// 已读状态管理
// ================================

const markVisibleMessagesAsRead = async () => {
  const container = messageContainer.value;
  if (!container) return;
  
  const visibleMessages = [];
  const messageElements = container.querySelectorAll('[data-message-id]');
  
  messageElements.forEach(element => {
    const rect = element.getBoundingClientRect();
    const containerRect = container.getBoundingClientRect();
    
    // 检查元素是否在可视区域内
    if (rect.top >= containerRect.top && rect.bottom <= containerRect.bottom) {
      const messageId = parseInt(element.dataset.messageId);
      if (messageId) {
        visibleMessages.push(messageId);
      }
    }
  });
  
  if (visibleMessages.length > 0) {
    try {
      await messageSystem.markMessagesAsRead(visibleMessages);
    } catch (error) {
      console.warn('⚠️ [EnhancedMessageList] Mark as read failed:', error);
    }
  }
};

// ================================
// 生命周期和监听
// ================================

// 监听新消息，自动滚动到底部
watch(() => messageSystem.state.messages.length, async (newLength, oldLength) => {
  if (newLength > oldLength) {
    await nextTick();
    
    const container = messageContainer.value;
    if (!container) return;
    
    const isNearBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 200;
    
    if (isNearBottom || !oldLength) {
      // 如果用户在底部附近或是初始加载，自动滚动到底部
      scrollToBottom(true);
    }
  }
});

// 监听聊天ID变化
watch(() => props.chatId, async () => {
  showScrollToBottom.value = false;
  await nextTick();
  scrollToBottom(false);
}, { immediate: true });

onMounted(async () => {
  // 初始加载消息
  try {
    await messageSystem.loadMessages();
    await nextTick();
    scrollToBottom(false);
  } catch (error) {
    console.error('❌ [EnhancedMessageList] Initial load failed:', error);
  }
  
  // 开始监听实时更新
  startRealtimeUpdates();
});

onUnmounted(() => {
  stopRealtimeUpdates();
});

// ================================
// 实时更新
// ================================

let realtimeInterval = null;

const startRealtimeUpdates = () => {
  // 定期更新未读计数和正在输入的用户
  realtimeInterval = setInterval(async () => {
    try {
      await messageSystem.updateUnreadCount();
      await messageSystem.updateTypingUsers();
    } catch (error) {
      console.warn('⚠️ [EnhancedMessageList] Realtime update failed:', error);
    }
  }, 3000);
};

const stopRealtimeUpdates = () => {
  if (realtimeInterval) {
    clearInterval(realtimeInterval);
    realtimeInterval = null;
  }
};

// ================================
// 公开方法
// ================================

defineExpose({
  scrollToBottom,
  loadMoreMessages,
  markVisibleMessagesAsRead
});
</script>

<style scoped>
/* 自定义滚动条 */
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

/* 平滑滚动 */
.scroll-smooth {
  scroll-behavior: smooth;
}
</style>