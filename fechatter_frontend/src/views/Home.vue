<template>
  <div class="slack-container">
    <!-- Mobile Menu Button -->
    <button v-if="isMobile" @click="toggleMobileSidebar" class="mobile-menu-button"
      :class="{ 'mobile-menu-open': isMobileSidebarOpen }">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path v-if="!isMobileSidebarOpen" d="M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z" />
        <path v-else
          d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
      </svg>
    </button>

    <!-- Mobile Overlay -->
    <div v-if="isMobile && isMobileSidebarOpen" class="mobile-overlay" @click="closeMobileSidebar"></div>

    <!-- Left Sidebar -->
    <aside class="slack-sidebar" :class="{
      'mobile-sidebar-open': isMobileSidebarOpen,
      'mobile-sidebar-closed': isMobile && !isMobileSidebarOpen
    }">
      <!-- Workspace Header -->
      <div class="slack-workspace-header">
        <div class="slack-workspace-info">
          <div class="slack-workspace-name-container">
            <AppIcon :size="20" :preserve-gradient="true" start-color="#ffffff" end-color="#dddddd"
              class="slack-workspace-icon" />
            <h1 class="slack-workspace-name">{{ currentWorkspace?.name || 'Fechatter' }}</h1>
          </div>
          <div class="slack-user-status">
            <div class="slack-connection-status" :class="{
              'connected': wsConnectionState.isConnected,
              'disconnected': !wsConnectionState.isConnected,
              'reconnecting': wsConnectionState.reconnectAttempts > 0
            }">
              <div class="slack-status-indicator"></div>
              <span v-if="wsConnectionState.isConnected">
                {{ currentUser?.fullname || 'User' }}
                <span v-if="wsConnectionState.latency" class="slack-latency">
                  {{ wsConnectionState.latency }}ms
                </span>
              </span>
              <span v-else-if="wsConnectionState.reconnectAttempts > 0" class="slack-reconnecting">
                Reconnecting... ({{ wsConnectionState.reconnectAttempts }})
              </span>
              <span v-else class="slack-offline">
                {{ currentUser?.fullname || 'User' }} (Offline)
              </span>
            </div>
          </div>
        </div>
        <div class="slack-header-actions">
          <button class="slack-compose-btn" @click="showCreateChannelModal = true" title="Create Channel">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M20 14h-6v6h-2v-6H6v-2h6V6h2v6h6v2z" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Navigation Sections -->
      <div class="slack-nav-content">
        <!-- Admin Status Bar (only for admins) -->
        <AdminStatusBar v-if="isCurrentUserAdmin" />

        <!-- Quick Actions -->
        <div class="slack-nav-section">
          <nav class="slack-nav-list">
            <router-link to="/home" class="slack-nav-item" :class="{ active: $route.path === '/home' }">
              <span class="slack-nav-icon">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z" />
                </svg>
              </span>
              <span class="slack-nav-text">Home</span>
            </router-link>
          </nav>
        </div>

        <!-- Unified Channel List - handles both channels and direct messages -->
        <div class="slack-unified-channels">
          <ChannelList :channels="channelsList" :directMessages="directMessagesList" :groupMessages="groupMessagesList"
            :isLoading="isLoading" :currentChatId="currentChatId" @channel-selected="handleChannelSelected"
            @create-channel="showCreateChannelModal = true" @create-dm="showCreateDMModal = true" @refresh="refresh" />
        </div>
      </div>

      <!-- User Bottom Bar -->
      <UserBottomBar @profile-opened="handleProfileOpened" @settings-opened="handleSettingsOpened"
        @role-switched="handleRoleSwitched" />
    </aside>

    <!-- Main Content Area -->
    <main class="slack-main">
      <!-- router-view will render nested routes here -->
      <router-view @create-channel="showCreateChannelModal = true" @create-dm="showCreateDMModal = true" />
    </main>

    <!-- Modals -->
    <CreateChannelModal v-if="showCreateChannelModal" @close="showCreateChannelModal = false"
      @created="onChannelCreated" />

    <!-- Enhanced Channel Creation Modals -->
    <CreateChannelModal v-if="showCreatePublicChannelModal" :channel-type="'public'" :available-users="availableUsers"
      @close="showCreatePublicChannelModal = false" @create-channel="handleCreateChannel" />

    <CreateChannelModal v-if="showCreatePrivateChannelModal" :channel-type="'private'" :available-users="availableUsers"
      @close="showCreatePrivateChannelModal = false" @create-channel="handleCreateChannel" />

    <!-- Group Chat Creation Modal -->
    <CreateGroupChatModal v-if="showCreateGroupChatModal" :available-users="availableUsers"
      @close="showCreateGroupChatModal = false" @create-group="handleCreateGroupChat" />

    <CreateDMModal v-if="showCreateDMModal" @close="showCreateDMModal = false" @created="onDMCreated" />

    <WorkspaceSettings v-if="showWorkspaceSettings" @close="showWorkspaceSettings = false" />

    <!-- Keyboard Shortcuts Modal -->
    <KeyboardShortcutsModal v-model="showKeyboardShortcutsModal" :shortcuts="keyboardShortcuts"
      @close="showKeyboardShortcutsModal = false" />

  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useWindowSize } from '@vueuse/core';
import { useHomeLayout } from '@/composables/useHomeLayout';
import { useChatStore } from '@/stores/chat';
import { useAuthStore } from '@/stores/auth';
import { useUserStore } from '@/stores/user';
import CreateChannelModal from '@/components/modals/CreateChannelModal.vue';
import CreateDMModal from '@/components/modals/CreateDMModal.vue';
import CreateGroupChatModal from '@/components/modals/CreateGroupChatModal.vue';
import WorkspaceSettings from '@/components/workspace/WorkspaceSettings.vue';
import KeyboardShortcutsModal from '@/components/modals/KeyboardShortcutsModal.vue';
import ChannelList from '@/components/layout/ChannelList.vue';
import AdminStatusBar from '@/components/layout/AdminStatusBar.vue';
import UserBottomBar from '@/components/layout/UserBottomBar.vue';
import { AppIcon } from '@/components/icons';
import { useRouterGuard } from '@/composables/useRouterGuard';
import { useNotifications } from '@/composables/useNotifications';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { errorHandler } from '@/utils/errorHandler';
import { debugDuplicateChannels, debugChannelsList } from '@/utils/debugDuplicateChannels';
import { debugMessageFlow } from '@/utils/messageDebugger';

const route = useRoute();
const router = useRouter();
const chatStore = useChatStore();
const authStore = useAuthStore();
const userStore = useUserStore();

// Add router guard for safe navigation
const { navigateToChat, getNavigationState, isNavigating } = useRouterGuard();

// Initialize keyboard shortcuts
const { shortcuts: keyboardShortcuts, emitGlobalEvent } = useKeyboardShortcuts({
  enableGlobalShortcuts: true,
  enableNavigationShortcuts: true,
  enableChatShortcuts: true,
  enableSearchShortcuts: true
});

// Mobile responsiveness
const { width: windowWidth } = useWindowSize();
const isMobile = computed(() => windowWidth.value < 768);
const isMobileSidebarOpen = ref(false);

// Use the composable for business logic
const {
  currentUser,
  currentWorkspace,
  channels,
  isLoading,
  wsConnectionState,
  showCreateChannelModal,
  showCreateDMModal,
  logout,
  onChannelCreated,
  onDMCreated,
  refresh,
  getLoadingStatus,
} = useHomeLayout();

// Local state for UI
const showUserMenu = ref(false);
const showWorkspaceSettings = ref(false);
const showKeyboardShortcutsModal = ref(false);
const showPerformanceStatus = ref(false);

// Platform detection for keyboard shortcuts display
const isMac = computed(() => navigator.platform.toUpperCase().indexOf('MAC') >= 0);
const shortcutHint = computed(() => isMac.value ? '⌘⇧/' : 'Ctrl+Shift+/');

// Check if current user is admin
const isCurrentUserAdmin = computed(() => {
  return currentUser.value?.role === 'admin' || currentUser.value?.role === 'super_admin'
})

// Navigation method for admin dashboard
const navigateToAdmin = () => {
  router.push('/admin');
};

// New handler functions for enhanced sidebar
const handleAddPublicChannel = () => {
  currentChannelType.value = 'public';
  showCreatePublicChannelModal.value = true;
};

const handleAddPrivateChannel = () => {
  currentChannelType.value = 'private';
  showCreatePrivateChannelModal.value = true;
};

const handleAddGroupChat = () => {
  showCreateGroupChatModal.value = true;
};

// Handler methods for UserBottomBar events
const handleProfileOpened = () => {
  // Open user profile modal or navigate to profile page
  console.log('Profile opened')
  // TODO: Implement profile modal
}

const handleSettingsOpened = () => {
  showWorkspaceSettings.value = true
}

const handleRoleSwitched = () => {
  // Handle role switching functionality
  console.log('Role switch requested')
  // TODO: Implement role switching
}

// New modals for enhanced functionality
const showCreateGroupChatModal = ref(false);
const showCreatePublicChannelModal = ref(false);
const showCreatePrivateChannelModal = ref(false);
const currentChannelType = ref('public');

// Computed properties for Sidebar
const allChannels = computed(() => {
  return channels.value || [];
});

const channelsList = computed(() => {
  // Ensure chatStore is available and properly initialized
  if (!chatStore || !chatStore.chats) {
    return [];
  }

  const allChats = Array.isArray(chatStore.chats) ? chatStore.chats : [];
  const filtered = allChats.filter(chat =>
    chat?.chat_type === 'PublicChannel' ||
    chat?.chat_type === 'PrivateChannel'
  ).map(channel => ({
    id: channel.id,
    name: channel.name || 'Unnamed Channel',
    chat_type: channel.chat_type,
    description: channel.description || '',
    unread_count: channel.unread_count || 0,
    mention_count: channel.mention_count || 0,
    last_message: channel.last_message || null,
    member_count: channel.member_count || 0,
    is_muted: channel.is_muted || false,
    updated_at: channel.updated_at
  }));

  // Debug: Check for duplicates
  if (process.env.NODE_ENV === 'development') {
    debugChannelsList(filtered);
  }

  return filtered;
});

const directMessagesList = computed(() => {
  // Ensure chatStore is available and properly initialized
  if (!chatStore || !chatStore.chats) {
    return [];
  }

  const allChats = Array.isArray(chatStore.chats) ? chatStore.chats : [];
  return allChats.filter(chat =>
    chat?.chat_type === 'Single'
  ).map(dm => {
    // Get the other user's ID from chat members
    const otherUserId = dm.chat_members?.find(id => id !== authStore.user?.id);

    // Try to get user info from userStore
    let userInfo = null;
    let avatarUrl = null;
    let userStatus = 'offline';

    if (otherUserId && userStore.getUserById) {
      userInfo = userStore.getUserById(otherUserId);
      if (userInfo) {
        avatarUrl = userInfo.avatar_url;
        userStatus = userInfo.status || 'offline';
      }
    }

    // Fallback to display_avatar if available
    if (!avatarUrl && dm.display_avatar) {
      avatarUrl = dm.display_avatar;
    }

    return {
      id: dm.id,
      name: dm.display_name || dm.name || 'Direct Message',
      chat_type: dm.chat_type,
      unread_count: dm.unread_count || 0,
      mention_count: dm.mention_count || 0,
      last_message: dm.last_message || null,
      user_status: userStatus,
      avatar_url: avatarUrl,
      updated_at: dm.updated_at,
      other_user_id: otherUserId // Store for future reference
    };
  });
});

const groupMessagesList = computed(() => {
  // Ensure chatStore is available and properly initialized
  if (!chatStore || !chatStore.chats) {
    return [];
  }

  const allChats = Array.isArray(chatStore.chats) ? chatStore.chats : [];
  return allChats.filter(chat =>
    chat?.chat_type === 'Group'
  ).map(group => ({
    id: group.id,
    name: group.display_name || group.name || 'Group Chat',
    chat_type: group.chat_type,
    unread_count: group.unread_count || 0,
    mention_count: group.mention_count || 0,
    last_message: group.last_message || null,
    user_status: 'group', // Groups don't have user status
    avatar_url: null, // Groups should use icon instead of avatar
    member_count: group.member_count || group.chat_members?.length || 0,
    updated_at: group.updated_at
  }));
});

const availableUsers = computed(() => {
  // Get all users from workspace or return mock data for now
  return [
    { id: 1, name: 'Alice Johnson', email: 'alice@example.com' },
    { id: 2, name: 'Bob Smith', email: 'bob@example.com' },
    { id: 3, name: 'Carol Davis', email: 'carol@example.com' },
  ];
});

// Current chat ID from route
const currentChatId = computed(() => {
  return route.params.id ? parseInt(route.params.id) : null;
});

// Handle channel selection with protection
const handleChannelSelected = async (channel) => {
  try {
    // Use protected navigation instead of direct router.push
    await navigateToChat(channel.id);

    // Close mobile sidebar after navigation
    if (isMobile.value) {
      closeMobileSidebar();
    }
  } catch (error) {
    console.error('❌ [HOME] Failed to navigate to chat:', error);
    // Don't show error notification for navigation cancellations
    if (!error.message?.includes('Navigation cancelled')) {
      const { notifyError } = useNotifications();
      notifyError('Failed to open chat. Please try again.');
    }
  }
};

// Handle message selection from search
const onMessageSelected = (event) => {
  // The search modal already handles navigation
  // We might need to emit an event to the Chat view to scroll to the message
  // This could be done via event bus or store
};

// Mobile functions
const toggleMobileSidebar = () => {
  isMobileSidebarOpen.value = !isMobileSidebarOpen.value;
};

const closeMobileSidebar = () => {
  isMobileSidebarOpen.value = false;
};

// Auto-close mobile sidebar when navigating to a chat
const onChannelSelectedMobile = (channel) => {
  onChannelSelected(channel);
  if (isMobile.value) {
    closeMobileSidebar();
  }
};

// Update channel selection handler to use mobile-aware version
const handleChannelSelectedMobile = (channel) => {
  onChannelSelectedMobile(channel);
};

// Show keyboard shortcuts help
const showKeyboardShortcuts = () => {
  showKeyboardShortcutsModal.value = true;
};

// New handler functions for enhanced sidebar
const handleCreateChannel = async (channelData) => {
  try {
    const channel = await chatStore.createChat(
      channelData.name,
      channelData.members,
      channelData.description,
      channelData.type === 'private' ? 'PrivateChannel' : 'PublicChannel'
    );

    // Navigate to the new channel
    router.push(`/chat/${channel.id}`);

    // Close modals
    showCreatePublicChannelModal.value = false;
    showCreatePrivateChannelModal.value = false;

    // Refresh the channel list
    await refresh();
  } catch (error) {
    console.error('Failed to create channel:', error);
  }
};

const handleCreateGroupChat = async (groupData) => {
  try {
    const group = await chatStore.createChat(
      groupData.name,
      groupData.members,
      groupData.description,
      'Group'
    );

    // Navigate to the new group
    router.push(`/chat/${group.id}`);

    // Close modal
    showCreateGroupChatModal.value = false;

    // Refresh the chat list
    await refresh();
  } catch (error) {
    console.error('Failed to create group chat:', error);
  }
};

// Close user menu when clicking outside
const handleClickOutside = (event) => {
  if (showUserMenu.value && !event.target.closest('.slack-user-section')) {
    showUserMenu.value = false;
  }
  if (showPerformanceStatus.value && !event.target.closest('.performance-status-panel')) {
    showPerformanceStatus.value = false;
  }
};

onMounted(() => {
  // Ensure chatStore is properly initialized
  if (!chatStore.chats || !Array.isArray(chatStore.chats)) {
    console.warn('⚠️ ChatStore not properly initialized, initializing with empty array');
    chatStore.$patch({ chats: [] });
  }

  // Debug duplicate channels issue
  if (process.env.NODE_ENV === 'development') {
    // Run debug after a short delay to ensure data is loaded
    setTimeout(() => {
      console.log('🔍 [HOME] Running duplicate channels debug...');
      debugDuplicateChannels();
    }, 2000);

    // Load message debugger
    import('@/utils/messageDebugger').then(() => {
      console.log('📨 Message debugger loaded');
    });
  }

  document.addEventListener('click', handleClickOutside);

  // Listen for global keyboard shortcut events
  window.addEventListener('fechatter:create-channel', () => {
    showCreateChannelModal.value = true;
  });

  window.addEventListener('fechatter:create-dm', () => {
    showCreateDMModal.value = true;
  });

  window.addEventListener('fechatter:open-settings', () => {
    showWorkspaceSettings.value = true;
  });

  window.addEventListener('fechatter:open-admin', () => {
    navigateToAdmin();
  });

  window.addEventListener('fechatter:toggle-sidebar', () => {
    if (isMobile.value) {
      toggleMobileSidebar();
    }
  });

  window.addEventListener('fechatter:cancel-action', () => {
    // Close any open modals
    showCreateChannelModal.value = false;
    showCreateDMModal.value = false;
    showWorkspaceSettings.value = false;
    showKeyboardShortcutsModal.value = false;
    showUserMenu.value = false;
    if (isMobile.value) {
      closeMobileSidebar();
    }
  });

  // Listen for keyboard shortcuts events
  window.addEventListener('fechatter:show-shortcuts-help', () => {
    showKeyboardShortcutsModal.value = true;
  });

  window.addEventListener('fechatter:open-settings', () => {
    showWorkspaceSettings.value = true;
  });

  window.addEventListener('fechatter:open-admin', () => {
    navigateToAdmin();
  });

  window.addEventListener('fechatter:toggle-sidebar', () => {
    if (isMobile.value) {
      toggleMobileSidebar();
    }
  });

  window.addEventListener('fechatter:cancel-action', () => {
    // Close any open modals
    showCreateChannelModal.value = false;
    showCreateDMModal.value = false;
    showWorkspaceSettings.value = false;
    showKeyboardShortcutsModal.value = false;
    showUserMenu.value = false;
    if (isMobile.value) {
      closeMobileSidebar();
    }
  });
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);

  // Remove global keyboard shortcut event listeners
  window.removeEventListener('fechatter:create-channel', () => { });
  window.removeEventListener('fechatter:create-dm', () => { });
  window.removeEventListener('fechatter:open-settings', () => { });
  window.removeEventListener('fechatter:open-admin', () => { });
  window.removeEventListener('fechatter:toggle-sidebar', () => { });
  window.removeEventListener('fechatter:cancel-action', () => { });
  window.removeEventListener('fechatter:show-shortcuts-help', () => { });
});
</script>

<style scoped>
/* Slack-like Design System */
.slack-container {
  display: flex;
  height: 100vh;
  background: #f8f9fa;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
}

/* Left Sidebar */
.slack-sidebar {
  width: 260px;
  background: #3f0f40;
  color: white;
  display: flex;
  flex-direction: column;
  position: relative;
}

.slack-workspace-header {
  padding: 20px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.slack-workspace-info {
  flex: 1;
  min-width: 0;
}

.slack-workspace-name-container {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.slack-workspace-icon {
  flex-shrink: 0;
}

.slack-workspace-name {
  font-size: 18px;
  font-weight: 900;
  margin: 0;
  color: white;
}

.slack-user-status {
  display: flex;
  align-items: center;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.7);
}

/* 🔌 WebSocket连接状态样式 */
.slack-connection-status {
  display: flex;
  align-items: center;
  transition: all 0.3s ease;
}

.slack-connection-status.connected {
  color: rgba(255, 255, 255, 0.9);
}

.slack-connection-status.disconnected {
  color: #ff6b6b;
}

.slack-connection-status.reconnecting {
  color: #ffa500;
}

.slack-status-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 6px;
  transition: background-color 0.3s ease;
}

.slack-connection-status.connected .slack-status-indicator {
  background: #2eb67d;
  box-shadow: 0 0 8px rgba(46, 182, 125, 0.5);
}

.slack-connection-status.disconnected .slack-status-indicator {
  background: #ff6b6b;
  box-shadow: 0 0 8px rgba(255, 107, 107, 0.5);
}

.slack-connection-status.reconnecting .slack-status-indicator {
  background: #ffa500;
  animation: pulse 1.5s infinite;
}

.slack-latency {
  font-size: 11px;
  opacity: 0.7;
  margin-left: 4px;
}

.slack-reconnecting,
.slack-offline {
  font-size: 12px;
  font-weight: 500;
}

.slack-header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.slack-action-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.9);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.slack-action-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  color: white;
}

.slack-compose-btn {
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 50%;
  background: white;
  color: #3f0f40;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.slack-compose-btn:hover {
  background: #f1f2f6;
}

/* Navigation Content */
.slack-nav-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px 0;
}

.slack-nav-section {
  margin-bottom: 24px;
}

.slack-section-header {
  display: flex;
  align-items: center;
  padding: 0 16px;
  margin-bottom: 8px;
}

.slack-section-toggle {
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  padding: 4px;
  margin-right: 4px;
  border-radius: 4px;
  transition: all 0.2s;
}

.slack-section-toggle:hover {
  background: rgba(255, 255, 255, 0.1);
}

.slack-chevron {
  transition: transform 0.2s;
}

.rotate-90 {
  transform: rotate(90deg);
}

.slack-section-title {
  font-size: 15px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.7);
  flex: 1;
}

.slack-section-actions {
  display: flex;
  gap: 4px;
}

.slack-add-btn {
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 4px;
  background: none;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.slack-add-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.8);
}

/* Navigation Items */
.slack-nav-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0 8px;
}

.slack-nav-item {
  display: flex;
  align-items: center;
  padding: 6px 16px;
  text-decoration: none;
  color: rgba(255, 255, 255, 0.7);
  font-size: 15px;
  line-height: 1.4;
  border-radius: 6px;
  margin: 0 2px;
  transition: all 0.2s;
  position: relative;
}

.slack-nav-item:hover {
  background: rgba(255, 255, 255, 0.1);
  color: white;
}

.slack-nav-item.active {
  background: #1264a3;
  color: white;
  font-weight: 600;
}

.slack-nav-icon {
  margin-right: 8px;
  font-size: 16px;
  display: flex;
  align-items: center;
}

.slack-nav-text {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.slack-channels-container {
  padding: 0 8px;
}

.slack-unified-channels {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.slack-unified-channels :deep(.channel-list) {
  background: transparent;
  height: 100%;
  overflow-y: auto;
}

.slack-channels-container :deep(.channel-list h2) {
  display: none;
  /* Hide the title since we already have it in the section header */
}

/* Style ChannelList to match Slack theme */
.slack-unified-channels :deep(.section-header) {
  padding: 12px 16px 8px;
  margin-bottom: 4px;
}

.slack-unified-channels :deep(.section-toggle) {
  color: rgba(255, 255, 255, 0.7);
  background: none;
  border: none;
}

.slack-unified-channels :deep(.section-toggle:hover) {
  color: rgba(255, 255, 255, 0.9);
}

.slack-unified-channels :deep(.section-title) {
  color: rgba(255, 255, 255, 0.7);
  font-size: 15px;
  font-weight: 600;
  text-transform: none;
  letter-spacing: normal;
}

.slack-unified-channels :deep(.action-btn) {
  color: rgba(255, 255, 255, 0.5);
  background: none;
  border: none;
}

.slack-unified-channels :deep(.action-btn:hover) {
  color: rgba(255, 255, 255, 0.8);
  background: rgba(255, 255, 255, 0.1);
}

.slack-unified-channels :deep(.channel-item),
.slack-unified-channels :deep(.dm-item) {
  color: rgba(255, 255, 255, 0.7);
  background: transparent;
  border-radius: 6px;
  margin: 1px 8px;
  padding: 6px 12px;
}

.slack-unified-channels :deep(.channel-item:hover),
.slack-unified-channels :deep(.dm-item:hover) {
  background: rgba(255, 255, 255, 0.1);
  color: white;
}

.slack-unified-channels :deep(.channel-item.active),
.slack-unified-channels :deep(.dm-item.active) {
  background: #1264a3;
  color: white;
  font-weight: 600;
}

.slack-unified-channels :deep(.empty-state) {
  color: rgba(255, 255, 255, 0.5);
  text-align: center;
  padding: 20px 16px;
}

.slack-unified-channels :deep(.create-first-btn) {
  background: #1264a3;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
}

.slack-unified-channels :deep(.create-first-btn:hover) {
  background: #0e4f82;
}

/* Empty States */
.slack-empty-section {
  padding: 12px 16px;
  text-align: center;
}

.slack-empty-section p {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.5);
  margin: 0 0 8px 0;
}

.slack-empty-action {
  background: none;
  border: none;
  color: #1d9bd1;
  font-size: 13px;
  cursor: pointer;
  text-decoration: underline;
}

/* Loading */
.slack-loading {
  padding: 0 16px;
}

.slack-skeleton {
  height: 32px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  margin-bottom: 4px;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {

  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.5;
  }
}

/* User Section */
.slack-user-section {
  margin-top: auto;
  padding: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  position: relative;
}

.slack-user-profile {
  display: flex;
  align-items: center;
  cursor: pointer;
  padding: 8px;
  border-radius: 6px;
  transition: background 0.2s;
}

.slack-user-profile:hover {
  background: rgba(255, 255, 255, 0.1);
}

.slack-user-avatar {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: #4a154b;
  border: 2px solid rgba(255, 255, 255, 0.2);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 14px;
  margin-right: 8px;
}

.slack-user-info {
  flex: 1;
  min-width: 0;
}

.slack-user-name {
  font-size: 15px;
  font-weight: 600;
  color: white;
  margin-bottom: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.slack-user-email {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.7);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.slack-user-menu-btn {
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s;
}

.slack-user-menu-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: white;
}

/* User Menu */
.slack-user-menu {
  position: absolute;
  bottom: 100%;
  left: 16px;
  right: 16px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  border: 1px solid rgba(0, 0, 0, 0.1);
  overflow: hidden;
  z-index: 1000;
}

.slack-menu-item {
  padding: 12px 16px;
  color: #1d1c1d;
  cursor: pointer;
  font-size: 15px;
  transition: background 0.2s;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.slack-menu-item:hover {
  background: #f8f9fa;
}

.slack-menu-icon {
  flex-shrink: 0;
  opacity: 0.7;
}

.slack-menu-danger:hover {
  background: #ffe5e5;
  color: #e01e5a;
}

.slack-menu-divider {
  height: 1px;
  background: rgba(0, 0, 0, 0.1);
}

.shortcut-hint {
  font-size: 11px;
  color: #9ca3af;
  background: #f3f4f6;
  padding: 2px 6px;
  border-radius: 4px;
  font-family: 'SF Mono', Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
  font-weight: 500;
}

/* Main Content */
.slack-main {
  flex: 1;
  background: white;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}

/* 旋转动画 */
.animate-spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

/* 脉冲动画 */
@keyframes pulse {

  0%,
  100% {
    opacity: 1;
    transform: scale(1);
  }

  50% {
    opacity: 0.7;
    transform: scale(1.1);
  }
}

/* ✨ 预加载指示器 */
.slack-preloaded-indicator {
  font-size: 10px;
  margin-left: 4px;
  opacity: 0.8;
  color: #ffd700;
  text-shadow: 0 0 4px rgba(255, 215, 0, 0.5);
}

/* 🚀 性能状态面板 */
.performance-status-panel {
  position: fixed;
  top: 20px;
  right: 20px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  border: 1px solid #e1e5e9;
  z-index: 1000;
  min-width: 300px;
  max-width: 400px;
}

/* 📱 Mobile Optimizations */
.mobile-menu-button {
  position: fixed;
  top: 12px;
  left: 12px;
  z-index: 1001;
  background: #4a154b;
  color: white;
  border: none;
  border-radius: 8px;
  padding: 10px;
  cursor: pointer;
  box-shadow: 0 2px 8px rgba(74, 21, 75, 0.3);
  transition: all 0.2s ease;
}

.mobile-menu-button:hover {
  background: #5d1a5e;
  transform: scale(1.05);
}

.mobile-menu-button.mobile-menu-open {
  background: #e01e5a;
}

.mobile-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 999;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }

  to {
    opacity: 1;
  }
}

/* Mobile sidebar animations */
.slack-sidebar {
  transition: transform 0.3s ease-in-out;
}

.mobile-sidebar-closed {
  transform: translateX(-100%);
}

.mobile-sidebar-open {
  transform: translateX(0);
}

/* 📱 Mobile Responsive Design */
@media (max-width: 767px) {
  .slack-container {
    position: relative;
  }

  .slack-sidebar {
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    width: 280px;
    z-index: 1000;
    box-shadow: 2px 0 8px rgba(0, 0, 0, 0.1);
  }

  .slack-main {
    margin-left: 0;
    width: 100%;
  }

  /* Make workspace header more compact on mobile */
  .slack-workspace-header {
    padding: 12px 16px;
  }

  .slack-workspace-name {
    font-size: 16px;
  }

  .slack-workspace-icon {
    width: 18px;
    height: 18px;
  }

  /* More compact section headers */
  .slack-section-header {
    padding: 8px 16px;
  }

  .slack-section-title {
    font-size: 13px;
  }

  /* More compact navigation items */
  .slack-nav-item {
    padding: 8px 16px;
    font-size: 14px;
  }

  .slack-nav-icon {
    width: 16px;
    height: 16px;
  }

  /* More compact user section */
  .slack-user-section {
    padding: 12px;
  }

  .slack-user-avatar {
    width: 32px;
    height: 32px;
    font-size: 12px;
  }

  .slack-user-name {
    font-size: 14px;
  }

  .slack-user-email {
    font-size: 12px;
  }

  /* Performance panel mobile adjustments */
  .performance-status-panel {
    top: 70px;
    right: 12px;
    left: 12px;
    max-width: none;
    min-width: 0;
  }
}

/* 📱 Small Mobile Devices (< 480px) */
@media (max-width: 479px) {
  .slack-sidebar {
    width: 100vw;
  }

  .slack-workspace-name-container {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }

  .slack-workspace-name {
    font-size: 14px;
  }

  .slack-user-info {
    display: none;
    /* Hide user email on very small screens */
  }

  /* Stack user profile vertically on very small screens */
  .slack-user-profile {
    flex-direction: column;
    text-align: center;
    gap: 4px;
  }
}

/* 📱 Tablet Portrait (768px - 1024px) */
@media (min-width: 768px) and (max-width: 1024px) {
  .slack-sidebar {
    width: 240px;
  }

  .slack-workspace-name {
    font-size: 17px;
  }

  .slack-section-title {
    font-size: 14px;
  }
}

/* 🎯 Touch-friendly enhancements */
@media (hover: none) and (pointer: coarse) {

  .slack-nav-item,
  .slack-section-toggle,
  .slack-add-btn,
  .slack-user-profile,
  .slack-menu-item {
    min-height: 44px;
    display: flex;
    align-items: center;
  }

  .slack-header-action {
    min-height: 44px;
    min-width: 44px;
  }
}

.performance-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #e1e5e9;
  background: #f8f9fa;
  border-radius: 8px 8px 0 0;
}

.performance-header h3 {
  margin: 0;
  font-size: 16px;
  color: #1d1c1d;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  color: #616061;
  cursor: pointer;
  padding: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: background 0.2s;
}

.close-btn:hover {
  background: rgba(97, 96, 97, 0.1);
}

.performance-content {
  padding: 16px 20px;
}

.status-grid {
  display: grid;
  gap: 12px;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
}

.status-label {
  font-size: 14px;
  color: #616061;
  font-weight: 500;
}

.status-value {
  font-size: 14px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 4px;
}

.status-value.excellent {
  color: #2eb67d;
  background: rgba(46, 182, 125, 0.1);
}

.status-value.good {
  color: #36c5f0;
  background: rgba(54, 197, 240, 0.1);
}

.status-value.fair {
  color: #ecb22e;
  background: rgba(236, 178, 46, 0.1);
}

.status-value.poor {
  color: #e01e5a;
  background: rgba(224, 30, 90, 0.1);
}
</style>