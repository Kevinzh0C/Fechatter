<template>
  <div class="enhanced-chat-view">
    <!-- 主聊天区域 -->
    <div class="chat-messages-area" ref="messagesContainer">
      <!-- 虚拟消息列表 -->
      <VirtualMessageList
        ref="messageList"
        :messages="messages"
        :current-user-id="currentUserId || 0"
        :chat-id="chatId"
        :loading="loading"
        @load-more-messages="handleLoadMore"
        @user-profile-opened="handleUserProfileOpened"
        @dm-created="handleDMCreated"
      />
      
      <!-- Jump to Latest 按钮 -->
      <JumpToLatestButton
        :scroll-container="messagesContainer"
        :unread-count="unreadCount"
        @jump-to-latest="handleJumpToLatest"
      />
    </div>
    
    <!-- 搜索模态框 -->
    <ChatSearchModal
      :is-open="searchModalOpen"
      :chat-id="chatId"
      @close="searchModalOpen = false"
      @message-selected="handleSearchResultSelected"
    />
    
    <!-- 快捷搜索栏（可选） -->
    <div v-if="showQuickSearch" class="quick-search-bar">
      <input
        v-model="quickSearchQuery"
        @input="handleQuickSearch"
        @focus="openFullSearch"
        placeholder="Search messages... (Ctrl+K)"
        class="quick-search-input"
      />
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useChatStore } from '@/stores/chat';
import { useChatSearch } from '@/composables/useChatSearch';
import { useFocusAnchoredScroll } from '@/composables/useFocusAnchoredScroll';
import { highlightKeyword } from '@/utils/highlightText';
import VirtualMessageList from './VirtualMessageList.vue';
import JumpToLatestButton from './JumpToLatestButton.vue';
import ChatSearchModal from './ChatSearchModal.vue';

const props = defineProps({
  chatId: {
    type: [String, Number],
    required: true
  },
  currentUserId: {
    type: Number,
    required: true
  },
  showQuickSearch: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['user-profile-opened', 'dm-created']);

// Store
const chatStore = useChatStore();

// Refs
const messagesContainer = ref(null);
const messageList = ref(null);
const searchModalOpen = ref(false);
const quickSearchQuery = ref('');

// Composables
const { 
  debouncedSearch,
  searchResults,
  isSearching
} = useChatSearch(props.chatId);

const {
  scrollToMessage,
  setScrollContainer
} = useFocusAnchoredScroll();

// Computed
const messages = computed(() => chatStore.messages);
const loading = computed(() => chatStore.loading);
const unreadCount = computed(() => {
  const chat = chatStore.getChatById(props.chatId);
  return chat?.unread_count || 0;
});

// 处理搜索结果选择
const handleSearchResultSelected = async (result) => {
  const messageId = result.messageId || result.id;
  
  // 1. 确保消息在当前视图中
  if (!messages.value.find(m => m.id === messageId)) {
    // 需要加载包含该消息的页面
    await chatStore.loadMessageContext(props.chatId, messageId);
  }
  
  // 2. 滚动到消息并高亮
  await scrollToMessage(messageId, {
    highlight: true,
    smooth: true,
    block: 'center',
    onComplete: (element) => {
      // 添加搜索高亮
      if (quickSearchQuery.value || result.query) {
        applySearchHighlight(element, result.query || quickSearchQuery.value);
      }
    }
  });
  
  // 3. 关闭搜索模态框
  searchModalOpen.value = false;
};

// 应用搜索高亮
const applySearchHighlight = (element, query) => {
  const contentElement = element.querySelector('.message-content, .message-text');
  if (!contentElement) return;
  
  const originalText = contentElement.textContent;
  const highlightedHTML = highlightKeyword(originalText, query, {
    className: 'search-highlight bg-yellow-300',
    maxLength: null
  });
  
  contentElement.innerHTML = highlightedHTML;
  
  // 3秒后移除高亮
  setTimeout(() => {
    contentElement.textContent = originalText;
  }, 3000);
};

// 快捷搜索处理
const handleQuickSearch = (event) => {
  const query = event.target.value;
  if (query.length >= 2) {
    debouncedSearch(query);
  }
};

// 打开完整搜索
const openFullSearch = () => {
  searchModalOpen.value = true;
  quickSearchQuery.value = '';
};

// 处理加载更多
const handleLoadMore = () => {
  chatStore.fetchMoreMessages(props.chatId);
};

// 处理跳转到最新
const handleJumpToLatest = () => {
  messageList.value?.scrollToBottom(true);
  // 清除未读计数
  chatStore.markChatAsRead(props.chatId);
};

// 处理用户资料打开
const handleUserProfileOpened = (user) => {
  emit('user-profile-opened', user);
};

// 处理DM创建
const handleDMCreated = (dm) => {
  emit('dm-created', dm);
};

// 键盘快捷键
const handleKeyboard = (event) => {
  // Ctrl/Cmd + K 打开搜索
  if ((event.ctrlKey || event.metaKey) && event.key === 'k') {
    event.preventDefault();
    searchModalOpen.value = true;
  }
  
  // Ctrl/Cmd + F 也可以打开搜索
  if ((event.ctrlKey || event.metaKey) && event.key === 'f') {
    event.preventDefault();
    searchModalOpen.value = true;
  }
};

// 生命周期
onMounted(() => {
  // 设置滚动容器
  setScrollContainer(messagesContainer.value);
  
  // 监听键盘事件
  document.addEventListener('keydown', handleKeyboard);
  
  // 监听搜索模态框打开事件
  const handleOpenSearch = () => {
    searchModalOpen.value = true;
  };
  window.addEventListener('open-search-modal', handleOpenSearch);
  
  // 清理
  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeyboard);
    window.removeEventListener('open-search-modal', handleOpenSearch);
  });
});
</script>

<style scoped>
.enhanced-chat-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  position: relative;
}

.chat-messages-area {
  flex: 1;
  overflow: hidden;
  position: relative;
  background: #fdfcfa;
}

/* 快捷搜索栏 */
.quick-search-bar {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  padding: 0.5rem 1rem;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  z-index: 10;
}

.quick-search-input {
  width: 100%;
  max-width: 400px;
  height: 36px;
  padding: 0 2.5rem 0 1rem;
  border: 1px solid #e5e5e5;
  border-radius: 18px;
  font-size: 0.875rem;
  background: white;
  transition: all 0.2s ease;
}

.quick-search-input:focus {
  outline: none;
  border-color: #7c3aed;
  box-shadow: 0 0 0 3px rgba(124, 58, 237, 0.1);
}

/* 搜索高亮样式 */
:deep(.search-highlight) {
  background: #fef3c7;
  padding: 0 2px;
  border-radius: 2px;
  font-weight: 500;
  animation: highlight-pulse 0.5s ease;
}

@keyframes highlight-pulse {
  0% {
    background: #fde047;
  }
  100% {
    background: #fef3c7;
  }
}

/* 消息高亮样式 */
:deep(.message-highlight) {
  background: rgba(124, 58, 237, 0.08);
  border-left: 3px solid #7c3aed;
  margin-left: -3px;
  transition: all 0.3s ease;
}

/* 响应式 */
@media (max-width: 768px) {
  .quick-search-bar {
    padding: 0.5rem;
  }
  
  .quick-search-input {
    max-width: 100%;
  }
}

/* 暗色模式 */
@media (prefers-color-scheme: dark) {
  .chat-messages-area {
    background: #1a1a1a;
  }
  
  .quick-search-bar {
    background: rgba(26, 26, 26, 0.95);
    border-bottom-color: rgba(255, 255, 255, 0.1);
  }
  
  .quick-search-input {
    background: #2a2a2a;
    border-color: #3a3a3a;
    color: #e0e0e0;
  }
  
  :deep(.search-highlight) {
    background: #854d0e;
    color: #fef3c7;
  }
  
  :deep(.message-highlight) {
    background: rgba(124, 58, 237, 0.2);
  }
}
</style>