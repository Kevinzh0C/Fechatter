<!--
  ChatHeaderActions.vue
  Jobs-Inspired Production-Grade Chat Header Actions
  
  Design Philosophy: 
  - Simplify: Remove visual clutter, focus on essential actions
  - Focus: Prioritize most-used functions (members, search, settings)
  - Humanize: Natural, intuitive interactions with smooth animations
  - Perfect: Pixel-perfect alignment, consistent spacing, premium feel
  - Integrate: Seamless integration with chat context
  - Anticipate: Smart defaults and predictive UI states
-->
<template>
  <div class="chat-header-actions">
    <!-- Primary Action Zone: Most Important Functions -->
    <div class="primary-actions">

      <!-- 🔍 Universal Search - Jobs believed in universal search as core UX -->
      <ActionButton @click="handleSearch" :active="searchActive" icon="search" label="Search"
        tooltip="Search messages (⌘K)" variant="search" :badge="searchResultsCount" />

      <!-- 👥 Members Hub - Central member management -->
      <ActionButton @click="handleMembers" :active="membersActive" icon="users" :label="membersLabel"
        :tooltip="membersTooltip" variant="members" :badge="onlineMembersCount" />

    </div>

    <!-- Secondary Action Zone: Context Actions -->
    <div class="secondary-actions">

      <!-- ⚙️ Smart Settings - Context-aware settings -->
      <ContextualMenu v-model:open="settingsOpen" :items="settingsMenuItems" @action="handleSettingsAction">
        <ActionButton :active="settingsOpen" icon="settings" label="More" tooltip="Chat options" variant="settings" />
      </ContextualMenu>

    </div>

    <!-- Floating Member Panel - Jobs-style slide-out -->
    <MemberPanel v-model:open="memberPanelOpen" :members="members" :online-count="onlineMembersCount" :chat="chat"
      :current-user="currentUser" @member-selected="handleMemberSelected" @invite-members="handleInviteMembers"
      @manage-permissions="handleManagePermissions" />

  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import ActionButton from './ActionButton.vue';
import ContextualMenu from './ContextualMenu.vue';
import MemberPanel from './MemberPanel.vue';

// Props
const props = defineProps({
  chat: {
    type: Object,
    required: true
  },
  members: {
    type: Array,
    default: () => []
  },
  currentUser: {
    type: Object,
    required: true
  }
});

// Emits - Jobs believed in clear, predictable API
const emit = defineEmits([
  'search-opened',
  'search-result-selected',
  'members-panel-opened',
  'member-selected',
  'member-invited',
  'settings-opened',
  'settings-updated',
  'chat-action-performed'
]);

// Reactive state - Minimal, focused state management
const searchActive = ref(false);
const membersActive = ref(false);
const settingsOpen = ref(false);
const memberPanelOpen = ref(false);
const searchResultsCount = ref(0);

// Mock online members count - replace with real presence logic
const onlineMembersCount = computed(() => {
  return props.members.filter(member => member.status === 'online').length;
});

// Mock permissions - replace with real permission logic
const canManageMembers = computed(() => {
  return props.currentUser?.role === 'admin' || props.currentUser?.role === 'owner';
});

const canEditSettings = computed(() => {
  return props.currentUser?.role === 'admin' || props.currentUser?.role === 'owner';
});

const canDeleteChat = computed(() => {
  return props.currentUser?.id === props.chat?.owner_id;
});

// Computed properties - Smart, context-aware labels
const membersLabel = computed(() => {
  const count = props.members.length;
  if (count === 0) return 'Members';
  if (count === 1) return '1 Member';
  return `${count} Members`;
});

const membersTooltip = computed(() => {
  if (onlineMembersCount.value === 0) {
    return 'View all members';
  }
  return `${onlineMembersCount.value} online • View all members`;
});

// Settings menu - Dynamic based on permissions and context
const settingsMenuItems = computed(() => {
  const baseItems = [
    {
      id: 'notifications',
      label: 'Notifications',
      icon: 'bell',
      action: () => handleNotificationSettings()
    }
  ];

  if (canEditSettings.value) {
    baseItems.unshift({
      id: 'chat-info',
      label: 'Chat Info',
      icon: 'info',
      action: () => handleChatInfo()
    });
  }

  if (canManageMembers.value) {
    baseItems.splice(1, 0, {
      id: 'permissions',
      label: 'Permissions',
      icon: 'shield',
      action: () => handlePermissions()
    });
  }

  // Danger zone
  if (props.chat.chat_type !== 'Single') {
    baseItems.push(
      { type: 'divider' },
      {
        id: 'leave',
        label: 'Leave Chat',
        icon: 'log-out',
        variant: 'danger',
        action: () => handleLeaveChat()
      }
    );
  }

  if (canDeleteChat.value) {
    baseItems.push({
      id: 'delete',
      label: 'Delete Chat',
      icon: 'trash',
      variant: 'danger',
      action: () => handleDeleteChat()
    });
  }

  return baseItems;
});

// Action handlers - Jobs emphasized clear, predictable actions
const handleSearch = () => {
  searchActive.value = true;
  emit('search-opened');
};

const handleMembers = () => {
  membersActive.value = true;
  memberPanelOpen.value = true;
  emit('members-panel-opened');
};

const handleSettingsAction = (action) => {
  action.action();
  settingsOpen.value = false;
};

const handleMemberSelected = (member) => {
  emit('member-selected', member);
};

const handleInviteMembers = () => {
  emit('member-invited');
};

const handleManagePermissions = () => {
  emit('settings-opened', { type: 'permissions' });
};

const handleChatInfo = () => {
  emit('settings-opened', { type: 'info' });
};

const handleNotificationSettings = () => {
  emit('settings-opened', { type: 'notifications' });
};

const handlePermissions = () => {
  emit('settings-opened', { type: 'permissions' });
};

const handleLeaveChat = () => {
  emit('chat-action-performed', { action: 'leave', chat: props.chat });
};

const handleDeleteChat = () => {
  emit('chat-action-performed', { action: 'delete', chat: props.chat });
};

// Watch for external state changes
watch(() => memberPanelOpen.value, (open) => {
  membersActive.value = open;
});
</script>

<style scoped>
/*
  Jobs-Inspired Design System
  - Minimal visual hierarchy
  - Perfect spacing and alignment
  - Premium materials and shadows
  - Smooth, purposeful animations
*/

.chat-header-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0;
  /* Jobs believed in invisible containers */
}

.primary-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  /* Tight grouping for primary actions */
}

.secondary-actions {
  display: flex;
  align-items: center;
  margin-left: 8px;
  /* Subtle separation from primary */
}

/* 
  Responsive Design - Jobs insisted on pixel-perfect across devices
*/
@media (max-width: 768px) {
  .chat-header-actions {
    gap: 2px;
  }

  .secondary-actions {
    margin-left: 4px;
  }
}

/*
  Touch-first Design - Jobs pioneered touch interaction
*/
@media (hover: none) and (pointer: coarse) {

  .primary-actions,
  .secondary-actions {
    gap: 8px;
    /* Larger touch targets */
  }
}

/*
  Accessibility - Jobs cared about inclusive design
*/
@media (prefers-reduced-motion: reduce) {
  .chat-header-actions * {
    transition: none !important;
  }
}

/*
  Dark mode - Seamless theme integration
*/
@media (prefers-color-scheme: dark) {
  .chat-header-actions {
    /* Dark mode styles handled by child components */
  }
}
</style>