<!--
  MemberPanel.vue
  Jobs-Inspired Member Management Panel
  
  Philosophy:
  - Integrate: Unified member management experience
  - Anticipate: Smart search, status prediction, quick actions
  - Humanize: Natural scrolling, breathing animations
  - Focus: Essential member information at a glance
-->
<template>
  <!-- Backdrop - Premium glass effect -->
  <div v-if="modelValue" class="member-panel-backdrop" @click="handleBackdropClick"
    :class="{ 'member-panel-backdrop--visible': showBackdrop }">
    <!-- Panel Container - Jobs loved slide-out drawers -->
    <div ref="panelRef" class="member-panel" :class="panelClasses" @click.stop>

      <!-- Panel Header - Clean, focused -->
      <div class="member-panel-header">
        <div class="header-content">
          <div class="header-info">
            <h3 class="panel-title">Members</h3>
            <div class="member-stats">
              <span class="total-count">{{ members.length }} total</span>
              <span v-if="onlineCount > 0" class="online-indicator">
                <div class="status-dot status-dot--online"></div>
                {{ onlineCount }} online
              </span>
            </div>
          </div>

          <!-- Quick Actions -->
          <div class="header-actions">
            <ActionButton icon="user-plus" tooltip="Invite members" variant="members" size="small"
              @click="handleInviteMembers" v-if="canInviteMembers" />

            <ActionButton icon="x" tooltip="Close panel" size="small" @click="handleClose" />
          </div>
        </div>

        <!-- Smart Search - Jobs loved universal search -->
        <div class="search-container">
          <div class="search-input-wrapper">
            <Icon name="search" class="search-icon" />
            <input ref="searchInputRef" v-model="searchQuery" type="text" placeholder="Search members..."
              class="search-input" @input="handleSearchInput" @focus="handleSearchFocus" @blur="handleSearchBlur"
              @keydown="handleSearchKeydown" />
            <button v-if="searchQuery" @click="clearSearch" class="search-clear">
              <Icon name="x" :size="14" />
            </button>
          </div>

          <!-- Search Suggestions -->
          <div v-if="showSuggestions && searchSuggestions.length > 0" class="search-suggestions">
            <div v-for="(suggestion, index) in searchSuggestions" :key="suggestion.id"
              :class="['suggestion-item', { 'suggestion-item--active': activeSuggestionIndex === index }]"
              @click="selectSuggestion(suggestion)" @mouseenter="activeSuggestionIndex = index">
              <MemberAvatar :member="suggestion" :size="24" />
              <span class="suggestion-name">{{ suggestion.fullname }}</span>
              <span class="suggestion-role">{{ suggestion.role || 'Member' }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Member List - Elegant, scannable -->
      <div class="member-list-container" ref="listContainerRef">

        <!-- Online Members Section -->
        <div v-if="onlineMembers.length > 0" class="member-section">
          <div class="section-header">
            <div class="section-indicator">
              <div class="status-dot status-dot--online"></div>
              <span>Online — {{ onlineMembers.length }}</span>
            </div>
          </div>

          <div class="member-grid">
            <MemberCard v-for="member in onlineMembers" :key="`online-${member.id}`" :member="member"
              :status="memberStatusMap[member.id] || 'online'" :is-current-user="member.id === currentUser?.id"
              @click="handleMemberClick" @start-dm="handleStartDM" @view-profile="handleViewProfile" />
          </div>
        </div>

        <!-- Offline Members Section -->
        <div v-if="offlineMembers.length > 0" class="member-section">
          <div class="section-header">
            <div class="section-indicator">
              <div class="status-dot status-dot--offline"></div>
              <span>Offline — {{ offlineMembers.length }}</span>
            </div>

            <button @click="toggleOfflineSection" class="section-toggle"
              :class="{ 'section-toggle--expanded': showOfflineMembers }">
              <Icon name="chevron-down" :size="14" />
            </button>
          </div>

          <Transition name="section-slide">
            <div v-if="showOfflineMembers" class="member-grid">
              <MemberCard v-for="member in offlineMembers" :key="`offline-${member.id}`" :member="member"
                :status="memberStatusMap[member.id] || 'offline'" :is-current-user="member.id === currentUser?.id"
                @click="handleMemberClick" @start-dm="handleStartDM" @view-profile="handleViewProfile" />
            </div>
          </Transition>
        </div>

        <!-- Empty State - Elegant messaging -->
        <div v-if="filteredMembers.length === 0" class="empty-state">
          <div class="empty-icon">
            <Icon name="users" :size="48" />
          </div>
          <h4 class="empty-title">
            {{ searchQuery ? 'No members found' : 'No members yet' }}
          </h4>
          <p class="empty-description">
            {{ searchQuery
              ? `Try searching for a different name or role`
              : 'Invite people to start collaborating'
            }}
          </p>
          <ActionButton v-if="!searchQuery && canInviteMembers" icon="user-plus" label="Invite Members"
            variant="primary" @click="handleInviteMembers" />
        </div>

      </div>

      <!-- Panel Footer - Contextual actions -->
      <div v-if="canManageMembers" class="member-panel-footer">
        <div class="footer-actions">
          <ActionButton icon="shield" label="Permissions" variant="settings" size="small"
            @click="handleManagePermissions" />

          <ActionButton icon="settings" label="Member Settings" variant="settings" size="small"
            @click="handleMemberSettings" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import ActionButton from './ActionButton.vue';
import MemberCard from './MemberCard.vue';
import MemberAvatar from './MemberAvatar.vue';
import Icon from '@/components/ui/Icon.vue';

// Props
const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  },
  members: {
    type: Array,
    default: () => []
  },
  onlineCount: {
    type: Number,
    default: 0
  },
  chat: {
    type: Object,
    required: true
  },
  currentUser: {
    type: Object,
    default: null
  }
});

// Emits
const emit = defineEmits([
  'update:modelValue',
  'member-selected',
  'invite-members',
  'manage-permissions',
  'start-dm',
  'view-profile'
]);

// Reactive state
const panelRef = ref(null);
const searchInputRef = ref(null);
const listContainerRef = ref(null);
const showBackdrop = ref(false);
const searchQuery = ref('');
const searchFocused = ref(false);
const showSuggestions = ref(false);
const activeSuggestionIndex = ref(-1);
const showOfflineMembers = ref(false);

// Composables - Mock implementations for now
const memberStatusMap = computed(() => {
  const statusMap = {};
  props.members.forEach(member => {
    statusMap[member.id] = member.status || 'offline';
  });
  return statusMap;
});

const isOnline = (memberId) => {
  return memberStatusMap.value[memberId] === 'online';
};

const canInviteMembers = computed(() => {
  return props.currentUser?.role === 'admin' || props.currentUser?.role === 'owner';
});

const canManageMembers = computed(() => {
  return props.currentUser?.role === 'admin' || props.currentUser?.role === 'owner';
});

// Computed properties - Smart filtering and grouping
const filteredMembers = computed(() => {
  if (!searchQuery.value.trim()) return props.members;

  const query = searchQuery.value.toLowerCase();
  return props.members.filter(member =>
    member.fullname?.toLowerCase().includes(query) ||
    member.email?.toLowerCase().includes(query) ||
    member.role?.toLowerCase().includes(query)
  );
});

const onlineMembers = computed(() =>
  filteredMembers.value.filter(member => isOnline(member.id))
);

const offlineMembers = computed(() =>
  filteredMembers.value.filter(member => !isOnline(member.id))
);

const searchSuggestions = computed(() => {
  if (!searchQuery.value.trim() || searchQuery.value.length < 2) return [];

  const query = searchQuery.value.toLowerCase();
  return props.members
    .filter(member =>
      member.fullname?.toLowerCase().includes(query) ||
      member.email?.toLowerCase().includes(query)
    )
    .slice(0, 5); // Limit suggestions
});

const panelClasses = computed(() => [
  'member-panel--slide',
  {
    'member-panel--visible': props.modelValue && showBackdrop.value
  }
]);

// Event handlers
const handleBackdropClick = () => {
  handleClose();
};

const handleClose = () => {
  emit('update:modelValue', false);
};

const handleMemberClick = (member) => {
  emit('member-selected', member);
};

const handleInviteMembers = () => {
  emit('invite-members');
};

const handleManagePermissions = () => {
  emit('manage-permissions');
};

const handleMemberSettings = () => {
  // Handle member settings
};

const handleStartDM = (member) => {
  emit('start-dm', member);
};

const handleViewProfile = (member) => {
  emit('view-profile', member);
};

const handleSearchInput = () => {
  showSuggestions.value = searchQuery.value.length > 0;
  activeSuggestionIndex.value = -1;
};

const handleSearchFocus = () => {
  searchFocused.value = true;
  if (searchQuery.value.length > 0) {
    showSuggestions.value = true;
  }
};

const handleSearchBlur = () => {
  searchFocused.value = false;
  // Delay hiding suggestions to allow clicks
  setTimeout(() => {
    showSuggestions.value = false;
  }, 150);
};

const handleSearchKeydown = (event) => {
  if (!showSuggestions.value || searchSuggestions.value.length === 0) return;

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault();
      activeSuggestionIndex.value = Math.min(
        activeSuggestionIndex.value + 1,
        searchSuggestions.value.length - 1
      );
      break;

    case 'ArrowUp':
      event.preventDefault();
      activeSuggestionIndex.value = Math.max(activeSuggestionIndex.value - 1, -1);
      break;

    case 'Enter':
      event.preventDefault();
      if (activeSuggestionIndex.value >= 0) {
        selectSuggestion(searchSuggestions.value[activeSuggestionIndex.value]);
      }
      break;

    case 'Escape':
      showSuggestions.value = false;
      activeSuggestionIndex.value = -1;
      searchInputRef.value?.blur();
      break;
  }
};

const selectSuggestion = (member) => {
  searchQuery.value = member.fullname;
  showSuggestions.value = false;
  activeSuggestionIndex.value = -1;
  handleMemberClick(member);
};

const clearSearch = () => {
  searchQuery.value = '';
  showSuggestions.value = false;
  searchInputRef.value?.focus();
};

const toggleOfflineSection = () => {
  showOfflineMembers.value = !showOfflineMembers.value;
};

// Watchers
watch(() => props.modelValue, (newValue) => {
  if (newValue) {
    showBackdrop.value = true;
    nextTick(() => {
      searchInputRef.value?.focus();
    });
  } else {
    showBackdrop.value = false;
    searchQuery.value = '';
    showSuggestions.value = false;
  }
});
</script>

<style scoped>
/*
  Jobs-Inspired Member Panel Design
  - Elegant slide-out drawer
  - Premium glass effects
  - Smart progressive disclosure
  - Natural scrolling and interactions
*/

.member-panel-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.2);
  backdrop-filter: blur(4px);
  z-index: var(--z-modal-backdrop, 9000);
  opacity: 0;
  visibility: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.member-panel-backdrop--visible {
  opacity: 1;
  visibility: visible;
}

.member-panel {
  position: fixed;
  top: 0;
  right: 0;
  height: 100vh;
  width: min(400px, 90vw);
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(40px);
  border-left: 1px solid rgba(255, 255, 255, 0.2);
  box-shadow: -8px 0 32px rgba(0, 0, 0, 0.12);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  z-index: var(--z-modal, 9500);
}

.member-panel--slide {
  transform: translateX(100%);
  transition: transform 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.member-panel--visible {
  transform: translateX(0);
}

/* Panel Header */
.member-panel-header {
  padding: 24px 20px 16px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.8) 0%, rgba(248, 250, 252, 0.8) 100%);
}

.header-content {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 16px;
}

.header-info {
  flex: 1;
}

.panel-title {
  font-size: 20px;
  font-weight: 700;
  color: #1a1a1a;
  margin: 0 0 4px 0;
  letter-spacing: -0.02em;
}

.member-stats {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 13px;
  color: #666;
}

.total-count {
  font-weight: 500;
}

.online-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  border: 2px solid rgba(255, 255, 255, 0.8);
}

.status-dot--online {
  background: #23a55a;
  box-shadow: 0 0 8px rgba(35, 165, 90, 0.4);
}

.status-dot--offline {
  background: #80848e;
}

.header-actions {
  display: flex;
  gap: 4px;
  margin-left: 12px;
}

/* Search Container */
.search-container {
  position: relative;
}

.search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.search-input {
  width: 100%;
  padding: 10px 12px 10px 36px;
  border: 1px solid rgba(0, 0, 0, 0.12);
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(20px);
  font-size: 14px;
  font-weight: 400;
  color: #1a1a1a;
  transition: all 0.2s ease;
  outline: none;
}

.search-input:focus {
  border-color: #007AFF;
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
  background: rgba(255, 255, 255, 0.95);
}

.search-input::placeholder {
  color: #999;
}

.search-icon {
  position: absolute;
  left: 12px;
  z-index: 1;
  color: #999;
  pointer-events: none;
}

.search-clear {
  position: absolute;
  right: 8px;
  padding: 4px;
  border: none;
  background: none;
  color: #999;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.2s ease;
}

.search-clear:hover {
  background: rgba(0, 0, 0, 0.05);
  color: #666;
}

/* Search Suggestions */
.search-suggestions {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  margin-top: 4px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
  z-index: 10;
  overflow: hidden;
}

.suggestion-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.suggestion-item:hover,
.suggestion-item--active {
  background: rgba(0, 122, 255, 0.08);
}

.suggestion-name {
  font-weight: 500;
  color: #1a1a1a;
}

.suggestion-role {
  font-size: 12px;
  color: #666;
  margin-left: auto;
}

/* Member List */
.member-list-container {
  flex: 1;
  overflow-y: auto;
  padding: 0 16px 16px;
}

.member-section {
  margin-bottom: 24px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 4px 8px;
  margin-bottom: 8px;
}

.section-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 600;
  color: #666;
  text-transform: uppercase;
  letter-spacing: 0.02em;
}

.section-toggle {
  padding: 4px;
  border: none;
  background: none;
  color: #999;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.2s ease;
}

.section-toggle:hover {
  background: rgba(0, 0, 0, 0.05);
  color: #666;
}

.section-toggle--expanded {
  transform: rotate(0deg);
}

.section-toggle:not(.section-toggle--expanded) {
  transform: rotate(-90deg);
}

.member-grid {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

/* Section Transitions */
.section-slide-enter-active,
.section-slide-leave-active {
  transition: all 0.3s ease;
}

.section-slide-enter-from,
.section-slide-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

/* Empty State */
.empty-state {
  text-align: center;
  padding: 60px 20px;
}

.empty-icon {
  margin-bottom: 16px;
  opacity: 0.4;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  color: #1a1a1a;
  margin: 0 0 8px 0;
}

.empty-description {
  font-size: 14px;
  color: #666;
  margin: 0 0 24px 0;
  line-height: 1.4;
}

/* Panel Footer */
.member-panel-footer {
  padding: 16px 20px;
  border-top: 1px solid rgba(0, 0, 0, 0.08);
  background: linear-gradient(180deg, rgba(248, 250, 252, 0.8) 0%, rgba(255, 255, 255, 0.8) 100%);
}

.footer-actions {
  display: flex;
  gap: 8px;
}

/* Dark mode */
@media (prefers-color-scheme: dark) {
  .member-panel {
    background: rgba(32, 32, 32, 0.95);
    border-left-color: rgba(255, 255, 255, 0.1);
  }

  .member-panel-header,
  .member-panel-footer {
    background: linear-gradient(180deg, rgba(40, 40, 40, 0.8) 0%, rgba(32, 32, 32, 0.8) 100%);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .panel-title {
    color: #ffffff;
  }

  .search-input {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.12);
    color: #ffffff;
  }

  .search-suggestions {
    background: rgba(32, 32, 32, 0.95);
    border-color: rgba(255, 255, 255, 0.12);
  }
}

/* Responsive design */
@media (max-width: 480px) {
  .member-panel {
    width: 100vw;
  }
}
</style>