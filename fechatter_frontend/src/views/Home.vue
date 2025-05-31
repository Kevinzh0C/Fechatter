<template>
  <div class="h-screen flex bg-gray-100">
    <!-- Mobile Menu Overlay -->
    <div v-if="showMobileSidebar" 
         class="fixed inset-0 z-40 lg:hidden"
         @click="showMobileSidebar = false">
      <div class="fixed inset-0 bg-black opacity-50"></div>
    </div>

    <!-- Left Sidebar - Chat List -->
    <div 
      :class="[
        'w-80 bg-purple-900 text-white flex flex-col transform transition-transform duration-300 ease-in-out lg:translate-x-0 lg:static lg:inset-0',
        showMobileSidebar ? 'translate-x-0' : '-translate-x-full',
        'fixed inset-y-0 left-0 z-50 lg:relative lg:z-auto'
      ]">
      <!-- Workspace Header -->
      <div class="p-4 border-b border-purple-800">
        <div class="flex items-center justify-between">
          <div class="flex-1 min-w-0 mr-2">
            <!-- Workspace Selector -->
            <WorkspaceSelector />
            <p class="text-sm text-purple-200 truncate mt-1">{{ authStore.user?.fullname || 'User' }}</p>
          </div>
          <div class="flex items-center space-x-2 ml-2">
            <!-- Mobile Close Button -->
            <button @click="showMobileSidebar = false" 
              class="p-1 text-purple-200 hover:text-white hover:bg-purple-800 rounded lg:hidden">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
              </svg>
            </button>
            <button @click="refreshChats" 
              class="p-1 text-purple-200 hover:text-white hover:bg-purple-800 rounded">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
              </svg>
            </button>
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
                <button @click="toggleTheme" 
                  class="block w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100">
                  {{ isDarkMode ? 'Light Mode' : 'Dark Mode' }}
                </button>
                <div class="border-t border-gray-100 my-1"></div>
                <button @click="logout" 
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
            <button @click="openNewChatModal"
              class="p-1 text-purple-200 hover:text-white hover:bg-purple-800 rounded">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"></path>
              </svg>
            </button>
          </div>

          <!-- Error State -->
          <div v-if="chatStore.error" class="bg-red-600 text-white p-3 rounded mb-4">
            <div class="flex items-center justify-between">
              <p class="text-sm">{{ chatStore.error }}</p>
              <button @click="chatStore.clearError" class="text-red-200 hover:text-white">
                <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                </svg>
              </button>
            </div>
          </div>

          <!-- Loading State -->
          <div v-if="chatStore.loading" class="text-center py-4">
            <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-purple-300 mx-auto"></div>
            <p class="mt-2 text-xs text-purple-200">Loading...</p>
          </div>

          <!-- Chat List -->
          <div v-else-if="chatStore.chats.length > 0" class="space-y-1">
            <div v-for="chat in chatStore.chats" :key="chat.id">
              <router-link :to="'/chat/' + chat.id" 
                @click="showMobileSidebar = false"
                class="flex items-center px-3 py-2 text-sm rounded hover:bg-purple-800 group"
                :class="{ 'bg-purple-700': $route.params.id == chat.id }">
                <span class="mr-2 text-purple-300">#</span>
                <span class="flex-1 truncate">{{ chat.name }}</span>
                <div class="opacity-0 group-hover:opacity-100 flex items-center space-x-1">
                  <button @click.prevent="editChat(chat)" 
                    class="p-1 text-purple-300 hover:text-white rounded">
                    <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
                    </svg>
                  </button>
                  <button @click.prevent="deleteChat(chat)" 
                    class="p-1 text-purple-300 hover:text-red-400 rounded">
                    <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                    </svg>
                  </button>
                </div>
              </router-link>
            </div>
          </div>

          <!-- Empty State -->
          <div v-else class="text-center py-6">
            <svg class="mx-auto h-6 w-6 text-purple-400 mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"></path>
            </svg>
            <p class="text-xs text-purple-300 mb-2">No channels yet</p>
            <button @click="openNewChatModal" 
              class="px-2 py-1 bg-purple-700 text-white text-xs rounded hover:bg-purple-600">
              Create Channel
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col min-w-0">
      <!-- Mobile Header -->
      <div class="lg:hidden bg-white border-b border-gray-200 px-4 py-3 flex items-center justify-between">
        <button @click="showMobileSidebar = true" 
          class="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-md">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
          </svg>
        </button>
        <h1 class="text-lg font-semibold text-gray-900 truncate">{{ workspaceStore.workspaceName }}</h1>
        <div class="w-9"></div> <!-- Spacer for centering -->
      </div>

      <!-- Welcome Message -->
      <div class="flex-1 flex items-center justify-center bg-white">
        <div class="text-center max-w-sm px-4">
          <div class="mb-4">
            <div class="mx-auto h-12 w-12 bg-purple-100 rounded-full flex items-center justify-center">
              <svg class="h-6 w-6 text-purple-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"></path>
              </svg>
            </div>
          </div>
          <h2 class="text-xl font-bold text-gray-900 mb-1">Welcome to Fechatter!</h2>
          <p class="text-gray-600 mb-4 text-sm">
            Select a channel to start chatting or create a new one.
          </p>
          <div class="space-y-2">
            <button @click="openNewChatModal"
              class="w-full px-3 py-2 bg-purple-600 text-white text-sm rounded-md hover:bg-purple-700">
              Create Channel
            </button>
            <div class="grid grid-cols-2 gap-2">
              <router-link to="/demo"
                class="block px-3 py-2 border border-purple-600 text-purple-600 text-xs rounded-md hover:bg-purple-50 text-center">
                Features
              </router-link>
              <button @click="showMobileSidebar = true"
                class="px-3 py-2 border border-purple-600 text-purple-600 text-xs rounded-md hover:bg-purple-50 lg:hidden">
                Channels
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- New Chat Modal -->
    <div v-if="showNewChatModal" class="fixed z-50 inset-0 overflow-y-auto">
      <div class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
        <div class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" @click="closeNewChatModal"></div>
        <div class="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
          <form @submit.prevent="createNewChat">
            <div class="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
              <div class="sm:flex sm:items-start">
                <div class="mt-3 text-center sm:mt-0 sm:text-left w-full">
                  <h3 class="text-lg leading-6 font-medium text-gray-900">
                    Create New Channel
                  </h3>
                  <div class="mt-4 space-y-4">
                    <div>
                      <label for="chatName" class="block text-sm font-medium text-gray-700">Channel Name</label>
                      <input type="text" id="chatName" v-model="newChatName"
                        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-purple-500 focus:ring-purple-500 sm:text-sm"
                        placeholder="e.g. general, random, project-alpha" required />
                    </div>
                    
                    <div>
                      <label for="chatType" class="block text-sm font-medium text-gray-700">Channel Type</label>
                      <select id="chatType" v-model="newChatType"
                        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-purple-500 focus:ring-purple-500 sm:text-sm">
                        <option value="PrivateChannel">Private Channel</option>
                        <option value="PublicChannel">Public Channel</option>
                        <option value="Group">Group Chat (3+ members)</option>
                        <option value="Single">Direct Message (1-on-1)</option>
                      </select>
                      <p class="mt-1 text-xs text-gray-500">
                        <span v-if="newChatType === 'Single'">Direct message between exactly 2 people</span>
                        <span v-else-if="newChatType === 'Group'">Group chat requires at least 3 members</span>
                        <span v-else-if="newChatType === 'PrivateChannel'">Private channel with invited members only</span>
                        <span v-else-if="newChatType === 'PublicChannel'">Public channel, anyone in workspace can join</span>
                      </p>
                    </div>
                    
                    <div>
                      <label for="chatDescription" class="block text-sm font-medium text-gray-700">Description</label>
                      <textarea id="chatDescription" v-model="newChatDescription"
                        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-purple-500 focus:ring-purple-500 sm:text-sm"
                        rows="3" placeholder="What's this channel about?"></textarea>
                    </div>
                    
                    <!-- Member selection for different chat types -->
                    <div v-if="newChatType === 'Single'">
                      <label class="block text-sm font-medium text-gray-700 mb-2">Select Contact</label>
                      <div class="max-h-40 overflow-y-auto border border-gray-300 rounded-md p-2">
                        <div v-for="user in userStore.workspaceUsers" :key="user.id" 
                             v-if="user.id !== authStore.user?.id"
                             class="flex items-center py-1">
                          <input type="radio" 
                                 :id="`user-${user.id}`"
                                 :value="user.id"
                                 v-model="selectedSingleUser"
                                 class="h-4 w-4 text-purple-600 focus:ring-purple-500 border-gray-300">
                          <label :for="`user-${user.id}`" class="ml-2 text-sm text-gray-700">
                            {{ user.fullname }} ({{ user.email }})
                          </label>
                        </div>
                      </div>
                      <p class="mt-1 text-xs text-gray-500">Select exactly one person for direct message</p>
                    </div>
                    
                    <div v-else-if="newChatType === 'Group'">
                      <label class="block text-sm font-medium text-gray-700 mb-2">Add Members (minimum 2 more)</label>
                      <div class="max-h-40 overflow-y-auto border border-gray-300 rounded-md p-2">
                        <div v-for="user in userStore.workspaceUsers" :key="user.id" 
                             v-if="user.id !== authStore.user?.id"
                             class="flex items-center py-1">
                          <input type="checkbox" 
                                 :id="`user-${user.id}`"
                                 :value="user.id"
                                 v-model="selectedMembers"
                                 class="h-4 w-4 text-purple-600 focus:ring-purple-500 border-gray-300 rounded">
                          <label :for="`user-${user.id}`" class="ml-2 text-sm text-gray-700">
                            {{ user.fullname }} ({{ user.email }})
                          </label>
                        </div>
                      </div>
                      <p class="mt-1 text-xs text-gray-500">
                        Selected: {{ selectedMembers.length }} members 
                        <span v-if="selectedMembers.length < 2" class="text-red-500">(need at least 2 more)</span>
                        <span v-else class="text-green-500">({{ selectedMembers.length + 1 }} total including you)</span>
                      </p>
                    </div>
                    
                    <div v-else-if="newChatType === 'PrivateChannel'">
                      <label class="block text-sm font-medium text-gray-700 mb-2">Add Members (optional)</label>
                      <div class="max-h-40 overflow-y-auto border border-gray-300 rounded-md p-2">
                        <div v-for="user in userStore.workspaceUsers" :key="user.id" 
                             v-if="user.id !== authStore.user?.id"
                             class="flex items-center py-1">
                          <input type="checkbox" 
                                 :id="`user-${user.id}`"
                                 :value="user.id"
                                 v-model="selectedMembers"
                                 class="h-4 w-4 text-purple-600 focus:ring-purple-500 border-gray-300 rounded">
                          <label :for="`user-${user.id}`" class="ml-2 text-sm text-gray-700">
                            {{ user.fullname }} ({{ user.email }})
                          </label>
                        </div>
                      </div>
                      <p class="mt-1 text-xs text-gray-500">You can invite specific members to this private channel</p>
                    </div>
                    
                    <div v-else-if="newChatType === 'PublicChannel'">
                      <div class="bg-blue-50 border border-blue-200 rounded-md p-3">
                        <div class="flex">
                          <div class="flex-shrink-0">
                            <svg class="h-5 w-5 text-blue-400" fill="currentColor" viewBox="0 0 20 20">
                              <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"></path>
                            </svg>
                          </div>
                          <div class="ml-3">
                            <p class="text-sm text-blue-700">
                              Public channels are visible to all workspace members. Anyone can join without invitation.
                            </p>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <div class="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
              <button type="submit" :disabled="isCreateDisabled"
                class="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-purple-600 text-base font-medium text-white hover:bg-purple-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500 sm:ml-3 sm:w-auto sm:text-sm disabled:opacity-50">
                <span v-if="chatStore.loading">Creating...</span>
                <span v-else>Create Channel</span>
              </button>
              <button type="button" @click="closeNewChatModal"
                class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm">
                Cancel
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>

    <!-- Edit Chat Modal -->
    <div v-if="showEditChatModal" class="fixed z-50 inset-0 overflow-y-auto">
      <div class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
        <div class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" @click="closeEditChatModal"></div>
        <div class="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
          <form @submit.prevent="updateChat">
            <div class="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
              <div class="sm:flex sm:items-start">
                <div class="mt-3 text-center sm:mt-0 sm:text-left w-full">
                  <h3 class="text-lg leading-6 font-medium text-gray-900">
                    Edit Channel
                  </h3>
                  <div class="mt-4 space-y-4">
                    <div>
                      <label for="editChatName" class="block text-sm font-medium text-gray-700">Channel Name</label>
                      <input type="text" id="editChatName" v-model="editingChat.name"
                        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-purple-500 focus:ring-purple-500 sm:text-sm"
                        required />
                    </div>
                    
                    <div>
                      <label for="editChatDescription" class="block text-sm font-medium text-gray-700">Description</label>
                      <textarea id="editChatDescription" v-model="editingChat.description"
                        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-purple-500 focus:ring-purple-500 sm:text-sm"
                        rows="3"></textarea>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <div class="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
              <button type="submit" :disabled="chatStore.loading"
                class="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-purple-600 text-base font-medium text-white hover:bg-purple-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500 sm:ml-3 sm:w-auto sm:text-sm disabled:opacity-50">
                <span v-if="chatStore.loading">Updating...</span>
                <span v-else>Update Channel</span>
              </button>
              <button type="button" @click="closeEditChatModal"
                class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm">
                Cancel
              </button>
            </div>
          </form>
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
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useChatStore } from '../stores/chat';
import { useAuthStore } from '../stores/auth';
import { useUserStore } from '../stores/user';
import { useWorkspaceStore } from '../stores/workspace';
import WorkspaceSettings from '../components/workspace/WorkspaceSettings.vue';
import WorkspaceSelector from '../components/workspace/WorkspaceSelector.vue';

const router = useRouter();
const chatStore = useChatStore();
const authStore = useAuthStore();
const userStore = useUserStore();
const workspaceStore = useWorkspaceStore();

const showNewChatModal = ref(false);
const showEditChatModal = ref(false);
const showWorkspaceSettings = ref(false);
const newChatName = ref('');
const newChatDescription = ref('');
const newChatType = ref('PrivateChannel');
const selectedMembers = ref([]);
const selectedSingleUser = ref(null);
const creating = ref(false);
const showUserMenu = ref(false);
const editingChat = ref(null);
const showMobileSidebar = ref(false);
const isDarkMode = ref(localStorage.getItem('theme') === 'dark');

onMounted(async () => {
  if (!authStore.token) {
    router.push('/login');
    return;
  }
  
  try {
    await Promise.all([
      workspaceStore.initializeWorkspace(),
      chatStore.fetchChats(),
      userStore.fetchWorkspaceUsers()
    ]);
  } catch (error) {
    console.error('Failed to load data:', error);
    // If authentication fails, redirect to login
    if (error.response?.status === 401) {
      authStore.logout();
      router.push('/login');
    }
  }
  
  // Add click outside handler
  document.addEventListener('click', handleClickOutside);
  
  // Initialize theme on mount
  initializeTheme();
});

function openNewChatModal() {
  showNewChatModal.value = true;
  selectedMembers.value = [];
  selectedSingleUser.value = null;
}

function closeNewChatModal() {
  showNewChatModal.value = false;
  newChatName.value = '';
  newChatDescription.value = '';
  newChatType.value = 'PrivateChannel';
  selectedMembers.value = [];
  selectedSingleUser.value = null;
}

// Validation function
function validateChatCreation() {
  if (!newChatName.value.trim()) {
    return 'Chat name is required';
  }
  
  if (newChatName.value.length > 128) {
    return 'Chat name cannot be longer than 128 characters';
  }
  
  switch (newChatType.value) {
    case 'Single':
      if (!selectedSingleUser.value) {
        return 'Please select exactly one person for direct message';
      }
      break;
    case 'Group':
      if (selectedMembers.value.length < 2) {
        return 'Group chat requires at least 2 additional members (3 total including you)';
      }
      break;
    case 'PrivateChannel':
      // No validation needed - can have any number of members
      break;
    case 'PublicChannel':
      // No validation needed - only creator initially
      break;
    default:
      return 'Invalid chat type selected';
  }
  
  return null;
}

// Computed property for button state
const isCreateDisabled = computed(() => {
  if (chatStore.loading) return true;
  
  const validationError = validateChatCreation();
  return validationError !== null;
});

async function createNewChat() {
  // Validate before creating
  const validationError = validateChatCreation();
  if (validationError) {
    alert(validationError);
    return;
  }
  
  creating.value = true;
  
  try {
    let members = [];
    
    // Prepare members based on chat type
    switch (newChatType.value) {
      case 'Single':
        // For Single chat, only pass the target user ID
        members = [selectedSingleUser.value];
        break;
      case 'Group':
        // For Group chat, pass selected members (backend will add creator)
        members = [...selectedMembers.value];
        break;
      case 'PrivateChannel':
        // For Private channel, pass selected members (backend will add creator)
        members = [...selectedMembers.value];
        break;
      case 'PublicChannel':
        // For Public channel, no members needed (backend will add creator)
        members = [];
        break;
    }
    
    const chat = await chatStore.createChat(
      newChatName.value,
      members,
      newChatDescription.value,
      newChatType.value
    );
    
    if (chat) {
      closeNewChatModal();
      router.push(`/chat/${chat.id}`);
    }
  } catch (error) {
    console.error('Failed to create chat:', error);
    // Error is already handled by the store
  } finally {
    creating.value = false;
  }
}

async function refreshChats() {
  await chatStore.fetchChats();
}

function editChat(chat) {
  editingChat.value = { ...chat };
  showEditChatModal.value = true;
}

function closeEditChatModal() {
  showEditChatModal.value = false;
  editingChat.value = null;
}

async function updateChat() {
  if (!editingChat.value) return;
  
  const success = await chatStore.updateChat(
    editingChat.value.id,
    editingChat.value.name,
    editingChat.value.description
  );
  
  if (success) {
    closeEditChatModal();
  }
}

async function deleteChat(chat) {
  if (confirm(`Are you sure you want to delete "${chat.name}"?`)) {
    await chatStore.deleteChat(chat.id);
  }
}

function getChatTypeClass(chatType) {
  const classes = {
    'PrivateChannel': 'bg-gray-100 text-gray-800',
    'PublicChannel': 'bg-green-100 text-green-800',
    'Group': 'bg-blue-100 text-blue-800',
    'Single': 'bg-purple-100 text-purple-800'
  };
  return classes[chatType] || 'bg-gray-100 text-gray-800';
}

function formatDate(dateString) {
  const date = new Date(dateString);
  const now = new Date();
  const diffInHours = (now - date) / (1000 * 60 * 60);
  
  if (diffInHours < 24) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  } else if (diffInHours < 24 * 7) {
    return date.toLocaleDateString([], { weekday: 'short' });
  } else {
    return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
  }
}

// Close user menu when clicking outside
function handleClickOutside(event) {
  if (!event.target.closest('.relative')) {
    showUserMenu.value = false;
  }
}

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});

// Theme management
function toggleTheme() {
  isDarkMode.value = !isDarkMode.value;
  const theme = isDarkMode.value ? 'dark' : 'light';
  localStorage.setItem('theme', theme);
  document.documentElement.classList.toggle('dark', isDarkMode.value);
  showUserMenu.value = false;
}

// Initialize theme on mount
function initializeTheme() {
  const savedTheme = localStorage.getItem('theme');
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  
  if (savedTheme) {
    isDarkMode.value = savedTheme === 'dark';
  } else {
    isDarkMode.value = prefersDark;
    localStorage.setItem('theme', prefersDark ? 'dark' : 'light');
  }
  
  document.documentElement.classList.toggle('dark', isDarkMode.value);
}

// Handle mobile sidebar
function handleResize() {
  if (window.innerWidth >= 1024) { // lg breakpoint
    showMobileSidebar.value = false;
  }
}

// Logout function
async function logout() {
  await authStore.logout();
  router.push('/login');
}
</script>