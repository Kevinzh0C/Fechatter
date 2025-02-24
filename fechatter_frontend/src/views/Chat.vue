<template>
  <div class="chat-view-container">
    <!-- 🔥 FULL WIDTH LAYOUT - NO SIDEBAR -->
    <div class="main-content-container full-width">
      <!-- Chat Content -->
      <div class="chat-content-container">
        <!-- Chat Header -->
        <div class="chat-header">
          <div class="chat-header-info">
            <div class="chat-title">
              <span class="channel-prefix">#</span>
              <h1>{{ currentChat?.name || 'Loading...' }}</h1>
            </div>
            <div class="chat-description">
              {{ currentChat?.description || 'Chat channel' }}
            </div>
          </div>

          <div class="chat-header-actions elegant">
            <button class="header-action elegant-search" @click="openPerfectSearch">
              <!-- 🔍 黄金分割比例搜索图标 -->
              <svg class="golden-search-icon" width="20" height="20" viewBox="0 0 100 100" fill="none"
                xmlns="http://www.w3.org/2000/svg">
                <!-- 主搜索圆环 - 黄金分割比例设计 -->
                <circle cx="38.2" cy="38.2" r="23.6" stroke="currentColor" stroke-width="6.18" fill="none"
                  stroke-linecap="round" />

                <!-- 搜索手柄 - 黄金分割角度 -->
                <line x1="55.9" y1="55.9" x2="76.4" y2="76.4" stroke="currentColor" stroke-width="6.18"
                  stroke-linecap="round" />

                <!-- 精致内部高光效果 -->
                <circle cx="32.4" cy="32.4" r="4.8" fill="currentColor" opacity="0.3" />
              </svg>
              <span class="action-text">Search</span>
            </button>
            <button class="header-action elegant-btn" @click="openChatSettings">
              <Icon name="settings" class="action-icon" />
            </button>
          </div>
        </div>

        <!-- Messages Container -->
        <div class="messages-container" :class="{ 'has-input-preview': hasInputPreview }">
          <!-- 🔥 DISCORD MESSAGE SYSTEM -->
          <DiscordMessageList :chat-id="currentChatId" :current-user-id="authStore.user?.id || 0"
            :messages="chatStore.messages" :loading="chatStore.loading" :has-more-messages="chatStore.hasMoreMessages"
            :typing-users="[]" @load-more-messages="handleLoadMoreMessages"
            @user-profile-opened="handleUserProfileOpened" @dm-created="handleUserProfileOpened"
            @reply-to="handleReplyTo" @edit-message="handleEditMessage" @delete-message="handleDeleteMessage"
            @scroll-position-changed="handleScrollPositionChanged"
            @reading-position-updated="handleReadingPositionUpdated" />
        </div>

        <!-- Message Input -->
        <div class="input-container">
          <MessageInput :chat-id="currentChatId" :current-user-id="authStore.user?.id || 0" class="message-input" />
        </div>
      </div>

      <!-- Perfect Search Modal -->
      <PerfectSearchModal v-if="showPerfectSearch" :is-open="showPerfectSearch" :chat-id="currentChatId"
        @close="handlePerfectSearchClose" @navigate="handlePerfectSearchNavigate" />

      <!-- Member Management Modal -->
      <div v-if="showMemberManagement" class="chat-modal-overlay" @click="showMemberManagement = false">
        <div class="chat-modal chat-modal-large" @click.stop>
          <div class="chat-modal-header">
            <h3>Manage Members</h3>
            <button class="chat-modal-close" @click="showMemberManagement = false">×</button>
          </div>
          <div class="chat-modal-content">
            <MemberManagement :chat-id="currentChatId" :chat="currentChat" @member-updated="handleMemberUpdated" />
          </div>
        </div>
      </div>

      <!-- ChatSettings Modal -->
      <div v-if="showChatSettings" class="chat-modal-overlay" @click="showChatSettings = false">
        <div class="chat-modal" @click.stop>
          <div class="chat-modal-header">
            <h3>Chat Settings</h3>
            <button class="chat-modal-close" @click="showChatSettings = false">×</button>
          </div>
          <div class="chat-modal-content">
            <ChatSettings :chat="currentChat" @close="showChatSettings = false" @save="handleChatSettingsUpdate" />
          </div>
        </div>
      </div>

      <!-- UserProfileModal -->
      <UserProfileModal v-if="selectedUserProfile" :user="selectedUserProfile" @close="selectedUserProfile = null" />

      <!-- Translation Panel -->
      <TranslationPanel v-if="activeTranslationPanel && translationPanelMessage" :visible="!!activeTranslationPanel"
        :message="translationPanelMessage" :position="getOptimalTranslationPanelPosition()"
        @close="handleTranslationPanelClose" @translated="handleTranslationCompleted"
        @applied="handleTranslationApplied" />
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { useChatStore } from '@/stores/chat';
import { useMessageUIStore } from '@/stores/messageUI';

// Components
import Icon from '@/components/icons/BaseIcon.vue';
import MessageInput from '@/components/chat/MessageInput.vue';
import DiscordMessageList from '@/components/discord/DiscordMessageList.vue';
import MemberManagement from '@/components/chat/MemberManagement.vue';
import ChatSettings from '@/components/chat/ChatSettings.vue';
import UserProfileModal from '@/components/user/UserProfileModal.vue';
import PerfectSearchModal from '@/components/search/PerfectSearchModal.vue';
import TranslationPanel from '@/components/chat/TranslationPanel.vue';

// Router and stores
const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();
const chatStore = useChatStore();
const messageUIStore = useMessageUIStore();

// State - 🔧 CRITICAL FIX: Initialize with route param or default
const currentChatId = ref(parseInt(route.params.id) || null);
const selectedUserProfile = ref(null);
const replyToMessage = ref(null);
const showPerfectSearch = ref(false);
const showMemberManagement = ref(false);
const showChatSettings = ref(false);
const hasInputPreview = ref(false);
const selectedTranslationPanel = ref(null);

// Computed
const currentChat = computed(() => chatStore.getCurrentChat);

// Translation panel integration
const activeTranslationPanel = computed(() => messageUIStore.activeTranslationPanel);
const translationPanelMessage = computed(() => {
  if (!activeTranslationPanel.value) return null;
  const messageId = activeTranslationPanel.value.messageId;
  return chatStore.messages.find(msg => msg.id === messageId);
});

// 🔧 OPTIMIZED: Prevent infinite loading loops
let loadingInProgress = false;

// 🔥 DISCORD MESSAGE LIST EVENT HANDLERS
const handleLoadMoreMessages = async () => {
  // 🔧 CRITICAL FIX: Prevent infinite loops
  if (!currentChatId.value || chatStore.loading || !chatStore.hasMoreMessages || loadingInProgress) {
    console.log('⏸️ Load more blocked:', {
      chatId: currentChatId.value,
      loading: chatStore.loading,
      hasMore: chatStore.hasMoreMessages,
      inProgress: loadingInProgress
    });
    return;
  }

  loadingInProgress = true;

  try {
    console.log('📥 Loading more messages for chat:', currentChatId.value);
    await chatStore.fetchMoreMessages(currentChatId.value);
  } catch (error) {
    console.error('Failed to load more messages:', error);
  } finally {
    // 🔧 Reset loading flag after delay to prevent rapid calls
    setTimeout(() => {
      loadingInProgress = false;
    }, 500);
  }
};

const handleUserProfileOpened = (user) => {
  selectedUserProfile.value = user;
};

const handleReplyTo = (message) => {
  replyToMessage.value = message;
};

const handleEditMessage = (message) => {
  console.log('Edit message:', message.id);
};

const handleDeleteMessage = (message) => {
  console.log('Delete message:', message.id);
};

const handleScrollPositionChanged = (position) => {
  console.log('Scroll position changed:', position);
};

const handleReadingPositionUpdated = (position) => {
  console.log('Reading position updated:', position);
};

// Chat management
const handleChannelSelected = async (chatId) => {
  if (!chatId) return;

  // 🔧 CRITICAL FIX: Ensure chatId is properly set
  currentChatId.value = parseInt(chatId);

  try {
    await chatStore.setCurrentChat(parseInt(chatId));
  } catch (error) {
    console.error('Failed to switch chat:', error);
  }
};

const openPerfectSearch = () => {
  showPerfectSearch.value = true;
};

const openChatSettings = () => {
  showChatSettings.value = true;
};

const handleMemberUpdated = () => {
  console.log('Member updated');
};

const handleChatSettingsUpdate = () => {
  console.log('Chat settings updated');
  showChatSettings.value = false;
};

// Translation Panel Event Handlers
const handleTranslationPanelClose = () => {
  console.log('🌐 Translation panel closed');
  messageUIStore.closeTranslationPanel();
};

const handleTranslationCompleted = (translationResult) => {
  console.log('🌐 Translation completed:', translationResult);
  // Additional logic if needed
};

const handleTranslationApplied = (applicationResult) => {
  console.log('🌐 Translation applied:', applicationResult);
  // Additional logic if needed - e.g., update message content
};

// Date formatting utility
const formatMessageDate = (dateString) => {
  if (!dateString) return '';
  const date = new Date(dateString);
  return date.toLocaleString();
};

// Lifecycle
onMounted(async () => {
  console.log('🔥 Chat.vue mounted - Discord system');

  // 🔧 CRITICAL FIX: Always ensure currentChatId is set
  const chatId = route.params.id;
  if (chatId && !currentChatId.value) {
    currentChatId.value = parseInt(chatId);
  }

  if (currentChatId.value) {
    await handleChannelSelected(currentChatId.value);
  }

  // 🔍 Add Perfect Search keyboard shortcut
  document.addEventListener('keydown', handleGlobalKeydown);
});

onUnmounted(() => {
  console.log('🔥 Chat.vue unmounted');
  loadingInProgress = false; // Reset on unmount

  // Remove keyboard event listener
  document.removeEventListener('keydown', handleGlobalKeydown);
});

// 🔍 Global keyboard shortcuts
const handleGlobalKeydown = (event) => {
  // Cmd+K (Mac) or Ctrl+K (Windows/Linux) to open Perfect Search
  if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
    event.preventDefault();
    openPerfectSearch();
  }
};

// Watch route changes
watch(
  () => route.params.id,
  async (newChatId) => {
    if (newChatId && parseInt(newChatId) !== currentChatId.value) {
      currentChatId.value = parseInt(newChatId);
      loadingInProgress = false; // Reset loading state
      await handleChannelSelected(parseInt(newChatId));
    }
  }
);

// 🔧 Watch currentChatId to ensure it's never null when needed
watch(currentChatId, (newId) => {
  if (newId) {
    console.log('✅ Chat ID set to:', newId);
  }
});

// Translation Panel position logic
const getOptimalTranslationPanelPosition = () => {
  // 🎯 优化翻译面板位置：右边最高位置，避免遮挡消息区域
  if (typeof window === 'undefined') {
    return { x: 400, y: 80 }; // SSR fallback
  }

  const viewportWidth = window.innerWidth;
  const viewportHeight = window.innerHeight;
  const panelWidth = 420; // 翻译面板宽度 + margin
  const panelHeight = 500; // 估算最大高度
  const topMargin = 80; // 距离顶部的安全距离
  const rightMargin = 20; // 距离右边的安全距离
  const messageAreaWidth = Math.min(860, viewportWidth * 0.7); // 消息区域宽度

  // 计算消息区域的右边界（居中布局）
  const messageAreaLeft = (viewportWidth - messageAreaWidth) / 2;
  const messageAreaRight = messageAreaLeft + messageAreaWidth;

  // 计算翻译面板的最佳位置
  let left, top;

  // 优先放在消息区域右侧
  if (viewportWidth - messageAreaRight >= panelWidth + rightMargin) {
    // 消息区域右侧有足够空间
    left = messageAreaRight + 16; // 距离消息区域16px
  } else if (messageAreaLeft >= panelWidth + rightMargin) {
    // 消息区域左侧有足够空间
    left = messageAreaLeft - panelWidth - 16;
  } else {
    // 屏幕右边显示，即使可能遮挡一部分消息
    left = viewportWidth - panelWidth - rightMargin;
  }

  // 垂直位置：尽可能高，但要考虑header高度
  top = topMargin;

  // 确保不超出屏幕边界
  left = Math.max(rightMargin, Math.min(left, viewportWidth - panelWidth - rightMargin));
  top = Math.max(topMargin, Math.min(top, viewportHeight - panelHeight - 20));

  return { x: left, y: top };
};

const handlePerfectSearchClose = () => {
  showPerfectSearch.value = false;
};

const handlePerfectSearchNavigate = (navigationResult) => {
  console.log('🎯 Perfect Search navigation:', navigationResult);
  // 关闭搜索模态框
  showPerfectSearch.value = false;

  // 如果需要切换到不同的聊天
  if (navigationResult.chatId && navigationResult.chatId !== currentChatId.value) {
    router.push(`/chat/${navigationResult.chatId}`);
  }

  // Perfect Search 已经处理了消息导航，这里只需要处理界面状态
};
</script>

<style scoped>
.chat-view-container {
  display: flex;
  height: 100vh;
  overflow: hidden;
  background-color: #f8f9fa;
}

.main-content-container {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  background-color: #313338;
}

.main-content-container.full-width {
  width: 100%;
  flex-grow: 1;
}

.chat-content-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  max-height: 100vh;
  overflow: hidden;
}

.chat-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid #e1e5e9;
  background: white;
  position: relative;
  z-index: 10;
  flex-shrink: 0;
}

.chat-header-info {
  flex: 1;
  min-width: 0;
}

.chat-title {
  display: flex;
  align-items: center;
  margin-bottom: 2px;
}

.channel-prefix {
  font-size: 16px;
  margin-right: 6px;
  color: #616061;
}

.chat-title h1 {
  font-size: 18px;
  font-weight: 900;
  color: #1d1c1d;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chat-description {
  font-size: 13px;
  color: #616061;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chat-header-actions.elegant {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-action {
  background: transparent;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.2s ease, color 0.2s ease;
  padding: 8px;
  color: #616061;
}

.header-action:hover {
  background-color: #f0f0f0;
  color: #1d1c1d;
}

.elegant-search {
  background-color: #f6f6f6;
  padding: 8px 12px;
  gap: 8px;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.elegant-search:hover {
  background-color: #e8f4fd;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 122, 255, 0.15);
}

.elegant-search .action-text {
  font-weight: 500;
  font-size: 14px;
}

.elegant-btn {
  width: 36px;
  height: 36px;
}

.action-icon {
  width: 20px;
  height: 20px;
}

.messages-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.messages-container.has-input-preview {
  transform: translateY(-150px);
}

.input-container {
  position: relative;
  z-index: 10;
  background: #ffffff;
  border-top: 1px solid #e5e7eb;
  padding: 16px;
}

.message-input {
  width: 100%;
}

/* Modals */
.chat-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.chat-modal {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border-radius: 24px;
  max-width: 600px;
  width: 100%;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border: none;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.12);
}

.chat-modal-large {
  max-width: 800px;
}

.chat-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: none;
  background: linear-gradient(to bottom, rgba(255, 255, 255, 0.8), rgba(248, 250, 252, 0.6));
  border-radius: 24px 24px 0 0;
}

.chat-modal-header h3 {
  font-size: 18px;
  font-weight: 700;
  color: #1d1c1d;
  margin: 0;
}

.chat-modal-close {
  background: rgba(255, 255, 255, 0.6);
  border: none;
  cursor: pointer;
  color: #616061;
  padding: 8px;
  border-radius: 16px;
  transition: all 0.3s ease;
  backdrop-filter: blur(10px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.chat-modal-close:hover {
  background: rgba(255, 255, 255, 0.9);
  color: #1d1c1d;
  transform: scale(1.05);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
}

.chat-modal-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}

/* 🔍 黄金分割搜索图标样式 */
.golden-search-icon {
  transition: all 0.2s ease;
  color: inherit;
}

.golden-search-icon:hover {
  transform: scale(1.1);
}

.elegant-search:hover .golden-search-icon {
  color: #007AFF;
}
</style>