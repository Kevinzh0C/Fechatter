<template>
  <div class="chat-content-container">
    <!-- Chat Header -->
    <header class="chat-header">
      <div class="chat-header-info">
        <div class="chat-title">
          <span class="channel-prefix">
            <svg v-if="currentChat?.chat_type === 'PrivateChannel'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm-6 9c-1.1 0-2-.9-2-2s.9-2 2-2 2 .9 2 2-.9 2-2 2zm3.1-9H8.9V6c0-1.71 1.39-3.1 3.1-3.1 1.71 0 3.1 1.39 3.1 3.1v2z"/>
            </svg>
            <span v-else class="font-bold text-lg">#</span>
          </span>
          <h1>{{ currentChat?.name || 'Loading...' }}</h1>
        </div>
        <div v-if="currentChat?.description" class="chat-description">
          {{ currentChat.description }}
        </div>
      </div>
      
      <div class="chat-header-actions">
        <!-- Online Users -->
        <OnlineUsers 
          :chat-id="currentChatId"
          :workspace-id="workspaceStore.activeWorkspaceId"
          @user-clicked="handleUserProfileOpened"
        />
        
        <!-- Member Count -->
        <div class="header-action" :title="`${chatMembers.length} members`">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/>
          </svg>
          <span>{{ chatMembers.length }}</span>
        </div>
        
        <!-- Search -->
        <button class="header-action" @click="handleSearchClick" title="Search in channel (Ctrl+F)">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
          </svg>
        </button>
        
        <!-- More Actions -->
        <div class="chat-dropdown">
          <button class="header-action" @click="showMoreMenu = !showMoreMenu" title="More actions">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"/>
            </svg>
          </button>
          
          <!-- More Menu -->
          <div v-if="showMoreMenu" class="chat-dropdown-menu">
            <div class="chat-menu-item" @click="showMemberManagement = true; showMoreMenu = false">
              <span>View members</span>
            </div>
            <div class="chat-menu-item" @click="showChatSettings = true; showMoreMenu = false">
              <span>Channel settings</span>
            </div>
            <div class="chat-menu-divider"></div>
            <div class="chat-menu-item chat-menu-danger" @click="handleLeaveChat">
              <span>Leave channel</span>
            </div>
          </div>
        </div>
      </div>
    </header>

    <!-- Messages Area -->
    <div class="messages-container">
      <!-- Loading State -->
      <div v-if="chatStore.loading && !currentChat" class="loading-state">
        <div class="loading-spinner"></div>
        <p>Loading messages...</p>
      </div>

      <!-- Messages -->
      <MessageList
        v-else
        :messages="chatStore.messages"
        :current-user-id="authStore.user?.id || 0"
        :chat-id="currentChatId"
        :loading="chatStore.loading"
        @load-more-messages="handleLoadMoreMessages"
        @user-profile-opened="handleUserProfileOpened"
        @dm-created="handleDMCreated"
        ref="messageListRef"
        class="message-list"
      />
    </div>

    <!-- Typing Indicator (Only for DM/Single chats) -->
    <TypingIndicator 
      v-if="currentChatId && currentChat?.chat_type === 'Single'"
      :chat-id="currentChatId"
      :current-user-id="authStore.user?.id || 0"
    />

    <!-- Message Input -->
    <div class="input-container">
      <MessageInput 
        @send="handleSendMessage" 
        :placeholder="`Message #${currentChat?.name || 'channel'}`"
        :chat-id="currentChatId"
        :chat-type="currentChat?.chat_type"
        class="message-input"
      />
    </div>

    <!-- User Profile Modal -->
    <UserProfile 
      v-if="selectedUserProfile"
      :user="selectedUserProfile"
      @close="selectedUserProfile = null"
      @dm-created="handleDMCreated"
    />

    <!-- Modals -->

    <!-- Member Management Modal -->
    <div v-if="showMemberManagement" class="chat-modal-overlay" @click="showMemberManagement = false">
      <div class="chat-modal" @click.stop>
        <MemberManagement 
          :chat="currentChat" 
          @close="showMemberManagement = false"
          @updated="handleMembersUpdated"
          @dm-created="handleDMCreated"
        />
      </div>
    </div>

    <!-- Search Modal -->
    <ChatSearchModal
      :model-value="showSearchModal"
      @update:model-value="showSearchModal = $event"
      :chat-id="currentChatId"
      :chat-name="currentChat?.name"
      :chat-members="chatMembers"
      @select-message="handleSearchMessageSelect"
    />

    <!-- Chat Settings Modal -->
    <div v-if="showChatSettings" class="chat-modal-overlay" @click="showChatSettings = false">
      <div class="chat-modal chat-modal-large" @click.stop>
        <div class="chat-modal-header">
          <h3>Channel Settings</h3>
          <button @click="showChatSettings = false" class="chat-modal-close">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
            </svg>
          </button>
        </div>
        <div class="chat-modal-content">
          <div class="chat-settings-form">
            <div class="chat-form-group">
              <label>Channel Name</label>
              <input v-model="editChatName" class="chat-input">
            </div>
            
            <div class="chat-form-group">
              <label>Description</label>
              <textarea v-model="editChatDescription" class="chat-input" rows="3"></textarea>
            </div>
            
            <div class="chat-settings-info">
              <h4>Channel Information</h4>
              <div class="chat-info-grid">
                <div class="chat-info-item">
                  <span class="chat-info-label">Type:</span>
                  <span>{{ currentChat?.chat_type }}</span>
                </div>
                <div class="chat-info-item">
                  <span class="chat-info-label">Created:</span>
                  <span>{{ formatDate(currentChat?.created_at) }}</span>
                </div>
                <div class="chat-info-item">
                  <span class="chat-info-label">Members:</span>
                  <span>{{ chatMembers.length }}</span>
                </div>
                <div class="chat-info-item">
                  <span class="chat-info-label">Owner:</span>
                  <span>{{ chatOwner?.fullname || 'Unknown' }}</span>
                </div>
              </div>
            </div>
            
            <div v-if="canDeleteChat" class="chat-danger-zone">
              <h4>Danger Zone</h4>
              <button @click="confirmDeleteChat" class="chat-btn-danger">
                Delete Channel
              </button>
            </div>
          </div>
        </div>
        <div class="chat-modal-footer">
          <button @click="showChatSettings = false" class="chat-btn-secondary">
            Cancel
          </button>
          <button @click="updateChat" :disabled="chatStore.loading" class="chat-btn-primary">
            {{ chatStore.loading ? 'Saving...' : 'Save Changes' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch, computed, nextTick, onUnmounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useChatStore } from '../stores/chat';
import { useAuthStore } from '../stores/auth';
import { useWorkspaceStore } from '../stores/workspace';
import MessageList from '../components/chat/MessageList.vue';
import MessageInput from '../components/chat/MessageInput.vue';
import MemberManagement from '../components/chat/MemberManagement.vue';
import ChatSearchModal from '../components/chat/ChatSearchModal.vue';
import TypingIndicator from '../components/chat/TypingIndicator.vue';
import OnlineUsers from '../components/chat/OnlineUsers.vue';
import UserProfile from '../components/modals/UserProfile.vue';
import api from '../services/api';

const route = useRoute();
const router = useRouter();
const chatStore = useChatStore();
const authStore = useAuthStore();
const workspaceStore = useWorkspaceStore();

// Reactive state
const showMemberManagement = ref(false);
const showSearchModal = ref(false);
const showChatSettings = ref(false);
const showMoreMenu = ref(false);
const editChatName = ref('');
const editChatDescription = ref('');
const messageListRef = ref(null);
const selectedUserProfile = ref(null);

// Computed properties
const currentChatId = computed(() => {
  const id = route.params.id;
  if (!id) return null;
  // Don't parseInt preview IDs, which are strings.
  if (String(id).startsWith('preview-dm-')) {
    return id;
  }
  return parseInt(id, 10);
});
const currentChat = computed(() => chatStore.getChatById(currentChatId.value));
const messages = computed(() => {
  const msgs = chatStore.messages;
  return msgs;
});
const chatMembers = computed(() => chatStore.getChatMembers(currentChatId.value));

const chatOwner = computed(() => {
  if (!currentChat.value) return null;
  return chatMembers.value.find(member => member.id === currentChat.value.owner_id);
});

const canDeleteChat = computed(() => {
  return currentChat.value && 
         authStore.user?.id === currentChat.value.owner_id;
});

// Lifecycle hooks
onMounted(async () => {
  if (!authStore.token) {
    router.push('/login');
    return;
  }
  
  if (route.params.id) {
    await loadChatData();
  }
  
  // Add keyboard shortcut for search
  document.addEventListener('keydown', handleKeyDown);
  
  // Listen for global keyboard shortcut events
  window.addEventListener('fechatter:open-search', () => {
    handleSearchClick();
  });
  
  window.addEventListener('fechatter:show-chat-info', () => {
    showChatSettings.value = true;
  });
  
  window.addEventListener('fechatter:cancel-action', () => {
    // Close any open modals or reset states
    showSearchModal.value = false;
    showMemberManagement.value = false;
    showChatSettings.value = false;
    showMoreMenu.value = false;
  });
  
  window.addEventListener('fechatter:toggle-members-list', () => {
    showMemberManagement.value = !showMemberManagement.value;
  });
});

// Enhanced keyboard shortcuts handler
const handleKeyDown = (event) => {
  // Don't interfere with the global shortcut system for most keys
  // Only handle very specific chat-context shortcuts here
  
  // Ctrl/Cmd + F or Ctrl/Cmd + K for search (legacy support)
  if ((event.ctrlKey || event.metaKey) && (event.key === 'f' || event.key === 'k')) {
    event.preventDefault();
    handleSearchClick();
  }
  
  // Escape to close modals or cancel actions
  if (event.key === 'Escape') {
    if (showSearchModal.value) {
      showSearchModal.value = false;
      event.preventDefault();
    } else if (showMemberManagement.value) {
      showMemberManagement.value = false;
      event.preventDefault();
    } else if (showChatSettings.value) {
      showChatSettings.value = false;
      event.preventDefault();
    } else if (showMoreMenu.value) {
      showMoreMenu.value = false;
      event.preventDefault();
    }
  }
};

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown);
});

// Handle search button click - verify auth state
async function handleSearchClick() {
  console.log('🔍 Search clicked - checking auth state:', {
    isAuthenticated: authStore.isAuthenticated,
    hasToken: !!authStore.token,
    tokenPreview: authStore.token ? authStore.token.substring(0, 20) + '...' : 'none',
    user: authStore.user,
    localStorage: {
      auth_token: !!localStorage.getItem('auth_token'),
      token: !!localStorage.getItem('token')
    },
    sessionStorage: {
      auth_token: !!sessionStorage.getItem('auth_token'),
      token: !!sessionStorage.getItem('token')
    }
  });
  
  if (!authStore.isAuthenticated || !authStore.token) {
    console.error('🔐 User not authenticated - cannot open search');
    router.push('/login');
    return;
  }
  
  // Test auth by making a simple API call
  try {
    const response = await api.get('/workspaces');
    } catch (error) {
    console.error('❌ Auth test failed:', {
      status: error.response?.status,
      data: error.response?.data,
      headers: error.config?.headers
    });
    
    if (error.response?.status === 401) {
      console.error('🔐 Token is invalid or expired');
      authStore.clearAuthState();
      router.push('/login');
      return;
    }
  }
  
  showSearchModal.value = true;
}

// 🚀 预加载机制：当鼠标悬停在聊天链接上时预加载
const preloadChatData = async (chatId) => {
  if (!chatId || chatStore.messageCache[chatId]) {
    return; // 已有缓存，无需预加载
  }
  
  try {
    // 在后台预加载数据
    await chatStore.fetchMessages(chatId, 10); // 预加载少量消息
  } catch (error) {
    console.warn('预加载失败:', error);
  }
};

// 🚀 性能优化：防抖路由变化，避免快速切换时的重复加载
let routeChangeTimeout = null;
watch(() => route.params.id, async (newId, oldId) => {
  if (newId && newId !== oldId) {
    // 清除之前的定时器
    if (routeChangeTimeout) {
      clearTimeout(routeChangeTimeout);
    }
    
    // Handle cleanup of temporary DM previews
    if (oldId && String(oldId).startsWith('preview-dm-')) {
      const previousChat = chatStore.getChatById(oldId);
      if (previousChat && previousChat.is_preview) {
        chatStore.removeTemporaryDM(oldId);
      }
    }
    
    // Debounce the new chat loading
    routeChangeTimeout = setTimeout(async () => {
      await loadChatData();
    }, 50); // 50ms防抖
  }
});

watch(currentChat, (newChat) => {
  if (newChat) {
    editChatName.value = newChat.name;
    editChatDescription.value = newChat.description || '';
  }
});

// 🚀 暴露预加载方法供父组件使用
defineExpose({
  preloadChatData
});

// Close dropdowns when clicking outside
const handleClickOutside = (event) => {
  if (showMoreMenu.value && !event.target.closest('.chat-dropdown')) {
    showMoreMenu.value = false;
  }
};

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
  document.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
  document.removeEventListener('keydown', handleKeyDown);
  
  // Remove global keyboard shortcut event listeners
  window.removeEventListener('fechatter:open-search', handleSearchClick);
  window.removeEventListener('fechatter:show-chat-info', () => {});
  window.removeEventListener('fechatter:cancel-action', () => {});
  window.removeEventListener('fechatter:toggle-members-list', () => {});
  
  // 🚀 清理定时器
  if (routeChangeTimeout) {
    clearTimeout(routeChangeTimeout);
  }
});

// Methods
async function loadChatData() {
  if (!currentChatId.value) return;

  const chatId = currentChatId.value;
  const chat = chatStore.getChatById(chatId);

  // Handle preview chats: they have no server messages.
  if (chat && chat.is_preview) {
    chatStore.messages = [];
    chatStore.hasMoreMessages = false;
    chatStore.currentChatId = chatId;
    return;
  }

  // Handle real chats
  try {
    // 1. Check for a valid cache first.
    const cachedData = chatStore.messageCache[chatId];
    const hasValidCache = cachedData && (Date.now() - cachedData.timestamp) < chatStore.cacheTimeout;

    if (hasValidCache) {
      chatStore.messages = cachedData.messages;
      chatStore.hasMoreMessages = cachedData.hasMoreMessages;
    } else {
      // 2. If no valid cache, clear previous messages and fetch from server.
      chatStore.clearMessages();
      await chatStore.fetchMessages(chatId, 15);
    }

    // 3. Set the current chat and load members in the background.
    chatStore.setCurrentChat(chatId);
    loadChatMembersIfNeeded(chatId);

    // 4. Scroll to bottom after messages are loaded.
    await nextTick();
    if (messageListRef.value) {
      messageListRef.value.scrollToBottom(false);
    }
  } catch (error) {
    console.error('Failed to load chat data:', error);
  } finally {
    }
}

async function loadChatMessages() {
  if (!currentChatId.value) return;
  
  try {
    await chatStore.fetchMessages(currentChatId.value, 15);
    } catch (error) {
    console.error('Failed to load chat messages:', error);
    }
}

// 🚀 新增：智能成员加载
async function loadChatMembersIfNeeded(chatId) {
  try {
    // 检查是否已有成员缓存
    const existingMembers = chatStore.getChatMembers(chatId);
    
    if (existingMembers && existingMembers.length > 0) {
      return;
    }
    
    await chatStore.fetchChatMembers(chatId);
    } catch (error) {
    console.error('Failed to load chat members:', error);
  }
}

// 保留原方法作为备用
async function loadChatMembers() {
  if (currentChatId.value) {
    try {
      await chatStore.fetchChatMembers(currentChatId.value);
    } catch (error) {
      console.error('Failed to load chat members:', error);
    }
  }
}

async function handleSendMessage({ content, files }) {
  console.log('📮 [Chat.vue] handleSendMessage called:', { 
    currentChatId: currentChatId.value,
    content,
    filesCount: files?.length || 0,
    chatStore_currentChatId: chatStore.currentChatId
  });
  
  if (!currentChatId.value || (!content.trim() && files.length === 0)) return;

  const chat = currentChat.value;
  if (!chat) return;

  // If this is a preview DM, create the chat first
  if (chat.is_preview) {
    const otherUser = chat.chat_members.find(m => m.id !== authStore.user.id);
    if (!otherUser) {
      console.error("Could not find the other user in the preview chat.");
      return;
    }
    
    try {
      // Create the actual chat
      const newChat = await chatStore.createChat(
        otherUser.fullname,
        [otherUser.id],
        '',
        'Single'
      );
      
      // Replace the temporary route with the real one
      router.replace(`/chat/${newChat.id}`);
      
      // Send the message to the new chat
      await chatStore.sendMessage(newChat.id, { content, files });

      // Optional: Clean up the temporary chat from the store
      chatStore.removeTemporaryDM(chat.id);

    } catch (error) {
      console.error("Failed to create DM and send message:", error);
      // TODO: show error notification
    }
  } else {
    // Standard message sending for existing chats
    await chatStore.sendMessage(currentChatId.value, { content, files });
  }
}

async function updateChat() {
  if (!currentChatId.value) return;
  
  try {
    await chatStore.updateChat(currentChatId.value, editChatName.value, editChatDescription.value);
    showChatSettings.value = false;
  } catch (error) {
    console.error('Failed to update chat:', error);
  }
}

async function confirmDeleteChat() {
  if (!currentChat.value) return;
  
  const confirmed = confirm(`Are you sure you want to delete "${currentChat.value.name}"? This action cannot be undone.`);
  if (!confirmed) return;
  
  try {
    await chatStore.deleteChat(currentChatId.value);
    router.push('/home');
  } catch (error) {
    console.error('Failed to delete chat:', error);
  }
}

const handleLeaveChat = () => {
  confirmLeaveChat();
  showMoreMenu.value = false;
};

async function confirmLeaveChat() {
  if (!currentChat.value) return;
  
  const confirmed = confirm(`Are you sure you want to leave "${currentChat.value.name}"?`);
  if (!confirmed) return;
  
  try {
    await chatStore.leaveChat(currentChatId.value);
    router.push('/home');
  } catch (error) {
    console.error('Failed to leave chat:', error);
  }
}

async function handleMembersUpdated() {
  await loadChatMembers();
}

async function handleDMCreated(chat) {
  selectedUserProfile.value = null; // Close the profile modal
  if (chat && chat.id) {
    // If it's a temporary chat, it's already in the store and list.
    // The watcher will handle cleanup if the user navigates away.
    router.push(`/chat/${chat.id}`);
  }
}

function handleUserProfileOpened(user) {
  selectedUserProfile.value = user;
}

// Search related methods

// Handle search message selection from modal
async function handleSearchMessageSelect(event) {
  // Close the search modal first
  showSearchModal.value = false;
  
  // If the message is in a different chat, navigate to it
  if (event.chatId && event.chatId !== currentChatId.value) {
    await router.push(`/chat/${event.chatId}`);
    
    // Wait for the new chat to load
    await nextTick();
    setTimeout(() => {
      // Scroll to the message after navigation
      if (event.messageId && messageListRef.value) {
        messageListRef.value.scrollToMessage(event.messageId);
      }
    }, 500);
  } else if (event.messageId && messageListRef.value) {
    // Same chat, just scroll to the message
    messageListRef.value.scrollToMessage(event.messageId);
  }
}

// Legacy method - kept for compatibility
async function handleSearchMessageSelected(event) {
  return handleSearchMessageSelect(event);
}

// Format date helper
function formatDate(timestamp) {
  if (!timestamp) return 'Unknown';
  const date = new Date(timestamp);
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  });
}

async function handleLoadMoreMessages() {
  if (currentChatId.value) {
    await chatStore.fetchMoreMessages(currentChatId.value);
  }
}
</script>

<style scoped>
/* Chat Content Container */
.chat-content-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: white;
  overflow: hidden;
}

/* Chat Header */
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

/* 📱 Mobile Chat Header Adjustments */
@media (max-width: 767px) {
  .chat-header {
    padding: 8px 12px;
    flex-wrap: wrap;
    gap: 8px;
  }

  .chat-header-info {
    flex: 1;
    min-width: 0;
    margin-right: 8px;
  }

  .chat-header-actions {
    gap: 4px;
    flex-wrap: wrap;
  }

  .chat-title h1 {
    font-size: 16px;
  }

  .chat-description {
    font-size: 12px;
  }

  .header-action {
    padding: 8px;
    min-width: 36px;
    min-height: 36px;
  }

  .header-action span {
    font-size: 12px;
  }
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

.chat-header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-action {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 8px;
  border: none;
  background: none;
  color: #616061;
  cursor: pointer;
  border-radius: 6px;
  font-size: 13px;
  transition: all 0.2s;
}

.header-action:hover {
  background: #f8f9fa;
  color: #1d1c1d;
}

/* Dropdown */
.chat-dropdown {
  position: relative;
}

.chat-dropdown-menu {
  position: absolute;
  top: 100%;
  right: 0;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border-radius: 20px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
  border: none;
  overflow: hidden;
  z-index: 1000;
  min-width: 180px;
  margin-top: 8px;
}

/* Messages Container */
.messages-container {
  flex: 1;
  min-height: 0;
  position: relative;
  overflow: hidden;
}

.message-list {
  height: 100%;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #616061;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #f1f2f6;
  border-top: 3px solid #4a154b;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 16px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* Message Input */
.input-container {
  padding: 16px;
  flex-shrink: 0;
  background: white;
  border-top: 1px solid #e1e5e9;
}

/* 📱 Mobile Message Input */
@media (max-width: 767px) {
  .input-container {
    padding: 12px;
  }
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

/* 📱 Mobile Modals */
@media (max-width: 767px) {
  .chat-modal-overlay {
    padding: 8px;
    align-items: flex-start;
    padding-top: 20px;
  }

  .chat-modal {
    max-width: 100%;
    max-height: 90vh;
    margin: 0;
  }

  .chat-modal-large {
    max-width: 100%;
  }

  .chat-modal-header {
    padding: 16px 20px;
  }

  .chat-modal-header h3 {
    font-size: 16px;
  }

  .chat-modal-content {
    padding: 16px 20px;
  }

  .chat-modal-footer {
    padding: 16px 20px;
    flex-direction: column;
    gap: 8px;
  }

  .chat-modal-footer button {
    width: 100%;
  }
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

.chat-modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 20px 24px;
  border-top: none;
  background: linear-gradient(to top, rgba(255, 255, 255, 0.8), rgba(248, 250, 252, 0.6));
  border-radius: 0 0 24px 24px;
}

/* Settings Form */
.chat-settings-form {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.chat-form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.chat-form-group label {
  font-size: 15px;
  font-weight: 600;
  color: #1d1c1d;
}

.chat-input {
  padding: 12px 16px;
  border: none;
  border-radius: 16px;
  font-size: 15px;
  transition: all 0.3s ease;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.chat-input:focus {
  outline: none;
  background: rgba(255, 255, 255, 0.95);
  box-shadow: 0 4px 16px rgba(116, 20, 121, 0.15);
  transform: translateY(-1px);
}

.chat-settings-info {
  background: linear-gradient(135deg, rgba(248, 250, 252, 0.8), rgba(241, 245, 249, 0.6));
  backdrop-filter: blur(10px);
  padding: 20px;
  border-radius: 20px;
  border: none;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.05);
}

.chat-settings-info h4 {
  font-size: 15px;
  font-weight: 600;
  color: #1d1c1d;
  margin: 0 0 12px 0;
}

.chat-info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
}

.chat-info-item {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
}

.chat-info-label {
  font-weight: 600;
  color: #616061;
}

.chat-danger-zone {
  border-top: 1px solid #e1e5e9;
  padding-top: 24px;
}

.chat-danger-zone h4 {
  font-size: 15px;
  font-weight: 600;
  color: #e01e5a;
  margin: 0 0 12px 0;
}

/* Buttons */
.chat-btn-primary {
  background: linear-gradient(135deg, #4a154b, #741479);
  color: white;
  border: none;
  padding: 12px 20px;
  border-radius: 16px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 4px 16px rgba(116, 20, 121, 0.25);
}

.chat-btn-primary:hover {
  background: linear-gradient(135deg, #5d1a5e, #8a1a8a);
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(116, 20, 121, 0.35);
}

.chat-btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
  box-shadow: 0 2px 8px rgba(116, 20, 121, 0.15);
}

.chat-btn-secondary {
  background: rgba(255, 255, 255, 0.8);
  color: #1d1c1d;
  border: none;
  padding: 12px 20px;
  border-radius: 16px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  backdrop-filter: blur(10px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.chat-btn-secondary:hover {
  background: rgba(255, 255, 255, 0.95);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
}

.chat-btn-danger {
  background: linear-gradient(135deg, #e01e5a, #cc1a4d);
  color: white;
  border: none;
  padding: 12px 20px;
  border-radius: 16px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 4px 16px rgba(224, 30, 90, 0.25);
}

.chat-btn-danger:hover {
  background: linear-gradient(135deg, #cc1a4d, #b81742);
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(224, 30, 90, 0.35);
}

/* Menu Items */
.chat-menu-item {
  padding: 12px 20px;
  color: #1d1c1d;
  cursor: pointer;
  font-size: 15px;
  transition: all 0.3s ease;
  border-radius: 12px;
  margin: 4px 8px;
}

.chat-menu-item:hover {
  background: rgba(248, 250, 252, 0.8);
  backdrop-filter: blur(10px);
  transform: translateX(4px);
}

.chat-menu-danger {
  color: #e01e5a;
}

.chat-menu-danger:hover {
  background: linear-gradient(135deg, rgba(255, 229, 229, 0.8), rgba(254, 226, 226, 0.6));
  backdrop-filter: blur(10px);
}

.chat-menu-divider {
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(225, 229, 233, 0.5), transparent);
  margin: 8px 16px;
}

.chat-danger-zone button:hover {
  background: #dc2626;
}

/* Online status indicator */
@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

/* Loading shimmer effect */
@keyframes shimmer {
  0% {
    background-position: -200px 0;
  }
  100% {
    background-position: calc(200px + 100%) 0;
  }
}

/* Smooth scrollbar */
.chat-messages-container::-webkit-scrollbar {
  width: 8px;
}

.chat-messages-container::-webkit-scrollbar-track {
  background: #f1f2f6;
}

.chat-messages-container::-webkit-scrollbar-thumb {
  background: #e1e5e9;
  border-radius: 4px;
}

.chat-messages-container::-webkit-scrollbar-thumb:hover {
  background: #e1e5e9;
}

</style>