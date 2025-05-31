<template>
  <div class="flex h-screen bg-gray-100 overflow-hidden">
    <!-- Left Sidebar - Chat List -->
    <div class="w-80 lg:w-80 md:w-64 sm:w-full sm:absolute sm:inset-y-0 sm:left-0 sm:z-50 bg-purple-900 text-white flex flex-col flex-shrink-0 transform transition-transform duration-300"
         :class="{ 'sm:-translate-x-full': !showMobileSidebar, 'sm:translate-x-0': showMobileSidebar }">
      
      <!-- Mobile Overlay -->
      <div v-if="showMobileSidebar" 
           class="fixed inset-0 bg-black/50 z-40 sm:block lg:hidden"
           @click="showMobileSidebar = false"></div>
      
      <!-- Workspace Header -->
      <div class="p-4 border-b border-purple-800 relative z-50">
        <div class="flex items-center justify-between">
          <div class="flex-1 min-w-0 mr-2">
            <h1 class="text-lg font-bold text-white truncate">{{ workspaceStore.workspaceName }}</h1>
            <p class="text-sm text-purple-200 truncate">{{ authStore.user?.fullname || 'User' }}</p>
          </div>
          <div class="flex items-center space-x-2 flex-shrink-0">
            <!-- Mobile Close Button -->
            <button v-if="showMobileSidebar" 
                    @click="showMobileSidebar = false"
                    class="p-1 text-purple-200 hover:text-white hover:bg-purple-800 rounded sm:block lg:hidden">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
              </svg>
            </button>
            
            <router-link to="/" 
              class="p-1 text-purple-200 hover:text-white hover:bg-purple-800 rounded">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"></path>
              </svg>
            </router-link>
            <div class="relative">
              <button @click="showUserMenu = !showUserMenu" 
                class="p-1 text-purple-200 hover:text-white hover:bg-purple-800 rounded">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"></path>
                </svg>
              </button>
              <!-- User Menu Dropdown -->
              <div v-if="showUserMenu" class="absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg py-1 z-50">
                <button @click="showWorkspaceSettings = true; showUserMenu = false" 
                  class="block w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100">
                  Workspace Settings
                </button>
                <div class="border-t border-gray-100 my-1"></div>
                <button @click="authStore.logoutAll" 
                  class="block w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100">
                  Logout All Sessions
                </button>
                <button @click="authStore.logout" 
                  class="block w-full text-left px-4 py-2 text-sm text-red-600 hover:bg-gray-100">
                  Logout
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Channels Section -->
      <div class="flex-1 overflow-y-auto">
        <div class="p-4">
          <!-- Channels Header -->
          <div class="flex items-center justify-between mb-3">
            <h2 class="text-sm font-semibold text-purple-200 uppercase tracking-wide">Channels</h2>
            <router-link to="/" 
              class="p-1 text-purple-200 hover:text-white hover:bg-purple-800 rounded">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"></path>
              </svg>
            </router-link>
          </div>

          <!-- Chat List -->
          <div v-if="chatStore.chats.length > 0" class="space-y-1">
            <div v-for="chat in chatStore.chats" :key="chat.id">
              <router-link :to="'/chat/' + chat.id" 
                @click="showMobileSidebar = false"
                class="flex items-center px-3 py-2 text-sm rounded hover:bg-purple-800 group"
                :class="{ 'bg-purple-700': chat.id == currentChatId }">
                <span class="mr-2 text-purple-300">#</span>
                <span class="flex-1 truncate">{{ chat.name }}</span>
                <div v-if="chat.id == currentChatId" class="w-2 h-2 bg-white rounded-full"></div>
              </router-link>
            </div>
          </div>

          <!-- Empty State -->
          <div v-else class="text-center py-8">
            <svg class="mx-auto h-8 w-8 text-purple-400 mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"></path>
            </svg>
            <p class="text-sm text-purple-300 mb-3">No channels yet</p>
            <router-link to="/" 
              class="px-3 py-1 bg-purple-700 text-white text-xs rounded hover:bg-purple-600">
              Create Channel
            </router-link>
          </div>
        </div>
      </div>
    </div>

    <!-- Main Chat Area -->
    <div class="flex-1 flex flex-col min-w-0 bg-white lg:ml-0 sm:ml-0">
      <!-- Mobile Header -->
      <div class="lg:hidden bg-white border-b border-gray-200 px-4 py-3 flex items-center justify-between">
        <button @click="showMobileSidebar = true" 
          class="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-md">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
          </svg>
        </button>
        <h1 class="text-lg font-semibold text-gray-900 truncate">
          {{ currentChat?.name || workspaceStore.workspaceName }}
        </h1>
        <div class="w-9"></div> <!-- Spacer for centering -->
      </div>
      
      <!-- Chat Header -->
      <ChatInfo 
        v-if="currentChat"
        :chat="currentChat"
        :member-count="chatMembers.length"
        :message-count="messages.length"
        :show-stats="false"
        @toggle-search="showSearchModal = true"
        @toggle-members="showMemberManagement = true"
        @edit-chat="editCurrentChat"
        @chat-settings="openChatSettings"
        @leave-chat="leaveCurrentChat" />

      <!-- Loading State -->
      <div v-if="chatStore.loading && !currentChat" class="flex-1 flex items-center justify-center bg-white">
        <div class="text-center">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-600 mx-auto"></div>
          <p class="mt-2 text-gray-500">Loading chat...</p>
        </div>
      </div>

      <!-- Chat Content -->
      <div v-else class="flex-1 flex flex-col min-h-0">
        <!-- Messages Container -->
        <div class="flex-1 min-h-0">
          <MessageList
            :messages="chatStore.messages"
            :current-user-id="authStore.user?.id"
            :loading="chatStore.loading"
            :has-more="chatStore.hasMoreMessages"
            @load-more="loadMoreMessages"
            ref="messageListRef"
          />
        </div>
        
        <!-- Message Input -->
        <div class="flex-shrink-0">
          <MessageInput @send="handleSendMessage" />
        </div>
      </div>
    </div>

    <!-- Members Management Modal -->
    <div v-if="showMemberManagement" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <MemberManagement 
        :chat="currentChat" 
        @close="showMemberManagement = false"
        @updated="handleMembersUpdated" />
    </div>

    <!-- Search Messages Modal -->
    <div v-if="showSearchModal" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[80vh]">
        <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">Search Messages</h3>
          <button @click="closeSearchModal" class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
            </svg>
          </button>
        </div>
        <div class="p-4">
          <CompactSearch 
            @search="handleCompactSearch"
            @select="jumpToMessage" />
        </div>
      </div>
    </div>

    <!-- Chat Settings Modal -->
    <div v-if="showChatSettings" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-lg p-6 w-96 max-w-full">
        <h3 class="text-lg font-semibold mb-4">Channel Settings</h3>
        
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Channel Name</label>
            <input v-model="editChatName" 
                   class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500">
          </div>
          
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Description</label>
            <textarea v-model="editChatDescription" 
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
                      rows="3"></textarea>
          </div>
          
          <div class="border-t pt-4">
            <h4 class="text-sm font-medium text-gray-700 mb-2">Channel Info</h4>
            <div class="space-y-2 text-sm text-gray-600">
              <p><span class="font-medium">Type:</span> {{ currentChat?.chat_type }}</p>
              <p><span class="font-medium">Created:</span> {{ formatDate(currentChat?.created_at) }}</p>
              <p><span class="font-medium">Members:</span> {{ chatMembers.length }}</p>
              <p><span class="font-medium">Owner:</span> {{ chatOwner?.fullname || 'Unknown' }}</p>
            </div>
          </div>
          
          <!-- Danger Zone -->
          <div v-if="canDeleteChat" class="border-t pt-4">
            <h4 class="text-sm font-medium text-red-700 mb-2">Danger Zone</h4>
            <button @click="confirmDeleteChat" 
                    class="px-3 py-2 bg-red-600 text-white text-sm rounded hover:bg-red-700">
              Delete Channel
            </button>
          </div>
        </div>
        
        <div class="flex justify-end space-x-3 mt-6">
          <button @click="showChatSettings = false" 
                  class="px-4 py-2 text-gray-600 hover:text-gray-800">
            Cancel
          </button>
          <button @click="updateChat" 
                  :disabled="chatStore.loading"
                  class="px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700 disabled:opacity-50">
            {{ chatStore.loading ? 'Saving...' : 'Save' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Workspace Settings Modal -->
    <div v-if="showWorkspaceSettings" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <WorkspaceSettings @close="showWorkspaceSettings = false" />
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
import WorkspaceSettings from '../components/workspace/WorkspaceSettings.vue';
import AdvancedSearch from '../components/chat/AdvancedSearch.vue';
import ChatInfo from '../components/chat/ChatInfo.vue';
import CompactSearch from '../components/search/CompactSearch.vue';

const route = useRoute();
const router = useRouter();
const chatStore = useChatStore();
const authStore = useAuthStore();
const workspaceStore = useWorkspaceStore();

const showMemberManagement = ref(false);
const showSearchModal = ref(false);
const showChatSettings = ref(false);
const showWorkspaceSettings = ref(false);
const searchQuery = ref('');
const searchResults = ref([]);
const searchLoading = ref(false);
const showUserMenu = ref(false);
const editChatName = ref('');
const editChatDescription = ref('');
const showMembersModal = ref(false);
const messageListRef = ref(null);
const showMobileSidebar = ref(false);

// 响应式状态管理
const isMobile = ref(false);

const currentChatId = computed(() => parseInt(route.params.id));
const currentChat = computed(() => chatStore.getCurrentChat);
const messages = computed(() => chatStore.messages);
const chatMembers = computed(() => chatStore.getChatMembers(currentChatId.value));

const chatOwner = computed(() => {
  if (!currentChat.value) return null;
  return chatMembers.value.find(member => member.id === currentChat.value.owner_id);
});

const canDeleteChat = computed(() => {
  return currentChat.value && 
         authStore.user?.id === currentChat.value.owner_id;
});

// 检查屏幕尺寸
function checkScreenSize() {
  isMobile.value = window.innerWidth < 1024; // lg断点
  if (!isMobile.value) {
    showMobileSidebar.value = false;
  }
}

// 监听窗口大小变化
function handleResize() {
  checkScreenSize();
}

onMounted(async () => {
  if (!authStore.token) {
    router.push('/login');
    return;
  }
  
  // 初始化响应式检查
  checkScreenSize();
  window.addEventListener('resize', handleResize);
  
  // Initialize workspace and load chats
  await Promise.all([
    workspaceStore.initializeWorkspace(),
    chatStore.fetchChats()
  ]);
  
  // Set current chat from route
  if (route.params.id) {
    await loadChatData();
  }
});

watch(() => route.params.id, async (newId) => {
  if (newId) {
    await loadChatData();
  }
});

watch(currentChat, (newChat) => {
  if (newChat) {
    editChatName.value = newChat.name;
    editChatDescription.value = newChat.description || '';
  }
});

async function loadChatData() {
  if (!currentChatId.value) return;
  
  try {
    // 清空之前的消息状态，显示加载状态
    chatStore.clearMessages();
    
    // Load messages and members in parallel
    await Promise.all([
      loadChatMessages(),
      loadChatMembers()
    ]);
    
    // 确保消息加载完成后滚动到底部
    await nextTick();
    if (messageListRef.value) {
      setTimeout(() => {
        messageListRef.value.scrollToBottom(false);
      }, 200);
    }
  } catch (error) {
    console.error('Failed to load chat data:', error);
  }
}

async function loadChatMessages() {
  if (!currentChatId.value) return;
  
  try {
    // 加载初始消息（最新的50条）
    await chatStore.fetchMessages(currentChatId.value, 50);
  } catch (error) {
    console.error('Failed to load chat messages:', error);
  }
}

async function loadChatMembers() {
  if (currentChatId.value) {
    try {
      await chatStore.fetchChatMembers(currentChatId.value);
    } catch (error) {
      console.error('Failed to load chat members:', error);
    }
  }
}

function selectChat(chatId) {
  router.push(`/chat/${chatId}`);
}

async function handleSendMessage({ content, files }) {
  if (!currentChatId.value) return;
  
  try {
    await chatStore.sendMessage(currentChatId.value, content, files);
    
    // 发送消息后自动滚动到底部
    await nextTick();
    if (messageListRef.value) {
      setTimeout(() => {
        messageListRef.value.scrollToBottom(true); // 使用平滑滚动
      }, 100);
    }
  } catch (error) {
    console.error('Failed to send message:', error);
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
    router.push('/');
  } catch (error) {
    console.error('Failed to delete chat:', error);
  }
}

async function handleMembersUpdated() {
  // Reload members when they are updated
  await loadChatMembers();
}

async function handleAdvancedSearch(query) {
  if (!currentChatId.value) return;
  
  try {
    searchLoading.value = true;
    searchResults.value = await chatStore.searchMessages(currentChatId.value, query);
  } catch (error) {
    console.error('Failed to search messages:', error);
    // Error is already handled by the store
  } finally {
    searchLoading.value = false;
  }
}

function closeSearchModal() {
  showSearchModal.value = false;
  searchQuery.value = '';
  searchResults.value = [];
}

async function jumpToMessage(message) {
  closeSearchModal();
  // 尝试滚动到消息
  if (messageListRef.value) {
    messageListRef.value.scrollToMessage(message.id);
  }
}

async function handleLogout() {
  await authStore.logout();
  router.push('/login');
}

async function loadMoreMessages() {
  if (currentChatId.value) {
    await chatStore.fetchMoreMessages(currentChatId.value);
  }
}

function formatDate(dateString) {
  const date = new Date(dateString);
  const now = new Date();
  const diffInHours = (now - date) / (1000 * 60 * 60);
  
  if (diffInHours < 1) {
    return 'Just now';
  } else if (diffInHours < 24) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  } else if (diffInHours < 24 * 7) {
    return date.toLocaleDateString([], { weekday: 'short', hour: '2-digit', minute: '2-digit' });
  } else {
    return date.toLocaleDateString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
  }
}

function editCurrentChat() {
  // Implementation for editing current chat
  console.log('Edit current chat');
  // You could emit an event or navigate to an edit page
}

function openChatSettings() {
  // Implementation for showing chat settings
  showChatSettings.value = true;
}

function leaveCurrentChat() {
  // Implementation for leaving current chat
  if (confirm('Are you sure you want to leave this chat?')) {
    console.log('Leave current chat');
    // You could call an API to leave the chat
    router.push('/');
  }
}

async function handleCompactSearch(searchParams) {
  try {
    searchLoading.value = true;
    const results = await chatStore.searchMessages(currentChatId.value, searchParams);
    searchResults.value = results.messages || [];
  } catch (error) {
    console.error('Search failed:', error);
    searchResults.value = [];
  } finally {
    searchLoading.value = false;
  }
}

// 清理事件监听器
onUnmounted(() => {
  window.removeEventListener('resize', handleResize);
});
</script>