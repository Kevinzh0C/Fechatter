<template>
  <div class="channel-list">
    <!-- Channels Section -->
    <div class="channel-section">
      <!-- Section Header -->
      <div class="section-header">
        <button @click="toggleChannelsExpanded" class="section-toggle" :class="{ 'expanded': isChannelsExpanded }">
          <Icon name="chevron-down" class="toggle-icon" />
          <span class="section-title">Channels</span>
          <span class="section-count">{{ channels.length }}</span>
        </button>

        <div class="header-actions">
          <button @click="toggleChannelsDropdown" class="action-btn" title="Add channel" ref="channelsMenuBtn">
            <Icon name="plus" />
          </button>
          <!-- Channels Dropdown Menu -->
          <div v-if="showChannelsDropdown" class="dropdown-menu" @click.stop>
            <div class="dropdown-item" @click="createChannel">
              <Icon name="hash" class="dropdown-icon" />
              <span>Create channel</span>
            </div>
            <div class="dropdown-item" @click="browseChannels">
              <Icon name="search" class="dropdown-icon" />
              <span>Browse channels</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Loading State -->
      <div v-if="isLoading" class="loading-state">
        <div v-for="n in 3" :key="n" class="channel-skeleton">
          <div class="skeleton-icon"></div>
          <div class="skeleton-text"></div>
        </div>
      </div>

      <!-- Channels List -->
      <transition-group v-else-if="isChannelsExpanded && channels.length > 0" name="channel-list" tag="div"
        class="channels-container">
        <ChannelItem v-for="channel in channels" :key="channel.id" :channel="channel"
          :isActive="currentChatId === channel.id" @click="selectChannel(channel)"
          @context-menu="showChannelContextMenu(channel, $event)" />
      </transition-group>

      <!-- Empty State -->
      <div v-else-if="isChannelsExpanded && channels.length === 0" class="empty-state">
        <p class="empty-text">No channels yet</p>
        <button @click="createChannel" class="empty-action">
          Create your first channel
        </button>
      </div>
    </div>

    <!-- Direct Messages Section -->
    <div class="channel-section">
      <!-- Section Header -->
      <div class="section-header">
        <button @click="toggleDMsExpanded" class="section-toggle" :class="{ 'expanded': isDMsExpanded }">
          <Icon name="chevron-down" class="toggle-icon" />
          <span class="section-title">Direct Messages</span>
          <span class="section-count">{{ directMessages.length }}</span>
        </button>

        <div class="header-actions">
          <button @click="toggleDMsDropdown" class="action-btn" title="New message" ref="dmsMenuBtn">
            <Icon name="plus" />
          </button>
          <!-- DMs Dropdown Menu -->
          <div v-if="showDMsDropdown" class="dropdown-menu" @click.stop>
            <div class="dropdown-item" @click="startDirectMessage">
              <Icon name="message-circle" class="dropdown-icon" />
              <span>New message</span>
            </div>
            <div class="dropdown-item" @click="browsePeople">
              <Icon name="users" class="dropdown-icon" />
              <span>Find people</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Direct Messages List -->
      <transition-group v-if="isDMsExpanded && directMessages.length > 0" name="channel-list" tag="div"
        class="channels-container">
        <DMItem v-for="dm in directMessages" :key="dm.id" :dm="dm" :isActive="currentChatId === dm.id"
          @click="selectChannel(dm)" />
      </transition-group>

      <!-- DMs Empty State -->
      <div v-else-if="isDMsExpanded && directMessages.length === 0" class="empty-state">
        <p class="empty-text">No conversations yet</p>
        <button @click="startDirectMessage" class="empty-action">
          Start a conversation
        </button>
      </div>
    </div>

    <!-- Group Messages Section -->
    <div class="channel-section">
      <!-- Section Header -->
      <div class="section-header">
        <button @click="toggleGroupsExpanded" class="section-toggle" :class="{ 'expanded': isGroupsExpanded }">
          <Icon name="chevron-down" class="toggle-icon" />
          <span class="section-title">Groups</span>
          <span class="section-count">{{ groupMessages.length }}</span>
        </button>

        <div class="header-actions">
          <button @click="toggleGroupsDropdown" class="action-btn" title="Create group" ref="groupsMenuBtn">
            <Icon name="plus" />
          </button>
          <!-- Groups Dropdown Menu -->
          <div v-if="showGroupsDropdown" class="dropdown-menu" @click.stop>
            <div class="dropdown-item" @click="createGroupMessage">
              <Icon name="users" class="dropdown-icon" />
              <span>Create group</span>
            </div>
            <div class="dropdown-item" @click="browseGroups">
              <Icon name="search" class="dropdown-icon" />
              <span>Browse groups</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Group Messages List -->
      <transition-group v-if="isGroupsExpanded && groupMessages.length > 0" name="channel-list" tag="div"
        class="channels-container">
        <DMItem v-for="group in groupMessages" :key="group.id" :dm="group" :isActive="currentChatId === group.id"
          @click="selectChannel(group)" />
      </transition-group>

      <!-- Groups Empty State -->
      <div v-else-if="isGroupsExpanded && groupMessages.length === 0" class="empty-state">
        <p class="empty-text">No groups yet</p>
        <button @click="createGroupMessage" class="empty-action">
          Create a group
        </button>
      </div>
    </div>

    <!-- Context Menu -->
    <ContextMenu v-if="contextMenu.show" :x="contextMenu.x" :y="contextMenu.y" :items="contextMenuItems"
      @close="closeContextMenu" @action="handleContextAction" />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useChatStore } from '@/stores/chat'
import { useAuthStore } from '@/stores/auth'
import Icon from '@/components/ui/Icon.vue'
import ChannelItem from './ChannelItem.vue'
import DMItem from './DMItem.vue'
import ContextMenu from '@/components/ui/ContextMenu.vue'
import { errorHandler } from '@/utils/errorHandler'

// Props
const props = defineProps({
  channels: {
    type: Array,
    default: () => []
  },
  directMessages: {
    type: Array,
    default: () => []
  },
  groupMessages: {
    type: Array,
    default: () => []
  },
  isLoading: {
    type: Boolean,
    default: false
  },
  currentChatId: {
    type: [Number, String],
    default: null
  }
})

// Emits
const emit = defineEmits([
  'channel-selected',
  'create-channel',
  'create-dm',
  'refresh'
])

// State
const router = useRouter()
const chatStore = useChatStore()
const authStore = useAuthStore()

const isChannelsExpanded = ref(true)
const isDMsExpanded = ref(true)
const isGroupsExpanded = ref(true)
const isRefreshing = ref(false)
const showChannelsDropdown = ref(false)
const showDMsDropdown = ref(false)
const showGroupsDropdown = ref(false)

// Context Menu
const contextMenu = ref({
  show: false,
  x: 0,
  y: 0,
  channel: null
})

// Computed
const isDev = computed(() => import.meta.env.DEV)
const totalChats = computed(() => props.channels.length + props.directMessages.length + props.groupMessages.length)
const currentListType = computed(() => {
  if (props.channels.length > 0 && props.directMessages.length > 0 && props.groupMessages.length > 0) return 'mixed'
  if (props.channels.length > 0) return 'channels'
  if (props.directMessages.length > 0) return 'dms'
  if (props.groupMessages.length > 0) return 'groups'
  return 'empty'
})

const contextMenuItems = computed(() => [
  { id: 'view', label: 'View Channel', icon: 'eye' },
  { id: 'edit', label: 'Edit Channel', icon: 'edit' },
  { id: 'members', label: 'Manage Members', icon: 'users' },
  { type: 'divider' },
  { id: 'mute', label: 'Mute Channel', icon: 'bell-off' },
  { id: 'leave', label: 'Leave Channel', icon: 'log-out', danger: true }
])

// Methods
const toggleChannelsExpanded = () => {
  isChannelsExpanded.value = !isChannelsExpanded.value
  localStorage.setItem('channels_expanded', isChannelsExpanded.value.toString())
}

const toggleDMsExpanded = () => {
  isDMsExpanded.value = !isDMsExpanded.value
  localStorage.setItem('dms_expanded', isDMsExpanded.value.toString())
}

const toggleGroupsExpanded = () => {
  isGroupsExpanded.value = !isGroupsExpanded.value
  localStorage.setItem('groups_expanded', isGroupsExpanded.value.toString())
}

const toggleChannelsDropdown = () => {
  showChannelsDropdown.value = !showChannelsDropdown.value
  showDMsDropdown.value = false // Close other dropdown
  showGroupsDropdown.value = false
}

const toggleDMsDropdown = () => {
  showDMsDropdown.value = !showDMsDropdown.value
  showChannelsDropdown.value = false // Close other dropdown
  showGroupsDropdown.value = false
}

const toggleGroupsDropdown = () => {
  showGroupsDropdown.value = !showGroupsDropdown.value
  showChannelsDropdown.value = false // Close other dropdown
  showDMsDropdown.value = false
}

const closeAllDropdowns = () => {
  showChannelsDropdown.value = false
  showDMsDropdown.value = false
  showGroupsDropdown.value = false
}

const selectChannel = (channel) => {
  emit('channel-selected', channel)
}

const refreshChannels = async () => {
  if (isRefreshing.value) return

  isRefreshing.value = true
  try {
    emit('refresh')
    await new Promise(resolve => setTimeout(resolve, 500)) // Minimum visual feedback
  } catch (error) {
    errorHandler.handle(error, { context: 'Refresh channels' })
  } finally {
    isRefreshing.value = false
  }
}

// Dropdown menu actions
const createChannel = () => {
  closeAllDropdowns()
  emit('create-channel')
}

const browseChannels = () => {
  closeAllDropdowns()
  // TODO: Implement browse channels functionality
  console.log('Browse channels clicked')
}

const startDirectMessage = () => {
  closeAllDropdowns()
  emit('create-dm')
}

const browsePeople = () => {
  closeAllDropdowns()
  // TODO: Implement browse people functionality
  console.log('Browse people clicked')
}

const createGroupMessage = () => {
  closeAllDropdowns()
  // TODO: Implement create group message functionality
  console.log('Create group message clicked')
}

const browseGroups = () => {
  closeAllDropdowns()
  // TODO: Implement browse groups functionality
  console.log('Browse groups clicked')
}

const showChannelContextMenu = (channel, event) => {
  event.preventDefault()
  contextMenu.value = {
    show: true,
    x: event.clientX,
    y: event.clientY,
    channel
  }
}

const closeContextMenu = () => {
  contextMenu.value.show = false
}

const handleContextAction = (action) => {
  const channel = contextMenu.value.channel
  closeContextMenu()

  switch (action.id) {
    case 'view':
      selectChannel(channel)
      break
    case 'edit':
      // Handle edit channel
      break
    case 'members':
      // Handle manage members
      break
    case 'mute':
      // Handle mute channel
      break
    case 'leave':
      // Handle leave channel
      break
  }
}

// Click outside handler
const handleClickOutside = (event) => {
  // Close dropdowns if clicking outside
  if (!event.target.closest('.dropdown-container')) {
    closeAllDropdowns()
  }

  // Close context menu if clicking outside
  if (contextMenu.value.show && !event.target.closest('.context-menu')) {
    closeContextMenu()
  }
}

// Lifecycle
onMounted(() => {
  // Restore expansion states
  const channelsExpanded = localStorage.getItem('channels_expanded')
  const dmsExpanded = localStorage.getItem('dms_expanded')
  const groupsExpanded = localStorage.getItem('groups_expanded')

  if (channelsExpanded !== null) {
    isChannelsExpanded.value = channelsExpanded === 'true'
  }
  if (dmsExpanded !== null) {
    isDMsExpanded.value = dmsExpanded === 'true'
  }
  if (groupsExpanded !== null) {
    isGroupsExpanded.value = groupsExpanded === 'true'
  }

  // Add global click listener for closing dropdowns
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  // Remove global click listener
  document.removeEventListener('click', handleClickOutside)
  closeContextMenu()
  closeAllDropdowns()
})
</script>

<style scoped>
/* 🎨 Modern Channel List Design - Inspired by Discord & Linear */
.channel-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: transparent;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Helvetica Neue', sans-serif;
}

/* 📱 Section Container */
.channel-section {
  margin-bottom: 24px;
  display: flex;
  flex-direction: column;
}

.channel-section:last-child {
  margin-bottom: 0;
}

/* 🏷️ Section Header - Clean & Minimal */
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  margin-bottom: 4px;
  height: 28px;
}

.section-toggle {
  display: flex;
  align-items: center;
  gap: 4px;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.6);
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  cursor: pointer;
  padding: 4px;
  margin: -4px;
  border-radius: 4px;
  transition: all 0.15s ease;
  flex: 1;
}

.section-toggle:hover {
  color: rgba(255, 255, 255, 0.8);
  background: rgba(255, 255, 255, 0.04);
}

.section-toggle:active {
  background: rgba(255, 255, 255, 0.08);
}

.toggle-icon {
  width: 10px;
  height: 10px;
  transition: transform 0.15s ease;
  flex-shrink: 0;
  color: rgba(255, 255, 255, 0.4);
}

.section-toggle:hover .toggle-icon {
  color: rgba(255, 255, 255, 0.6);
}

.section-toggle.expanded .toggle-icon {
  transform: rotate(0deg);
}

.section-toggle:not(.expanded) .toggle-icon {
  transform: rotate(-90deg);
}

.section-title {
  flex: 1;
  text-align: left;
}

.section-count {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
  font-weight: 500;
  margin-left: 4px;
}

/* 🎯 Header Actions */
.header-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  position: relative;
}

.action-btn {
  width: 20px;
  height: 20px;
  border: none;
  border-radius: 4px;
  background: none;
  color: rgba(255, 255, 255, 0.4);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
  position: relative;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.7);
}

.action-btn:active {
  background: rgba(255, 255, 255, 0.12);
}

.action-btn svg {
  width: 14px;
  height: 14px;
}

/* 📋 Dropdown Menu - Discord Style */
.dropdown-menu {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  background: #111214;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.24);
  min-width: 180px;
  z-index: 1000;
  overflow: hidden;
  animation: dropdownSlide 0.15s ease-out;
}

@keyframes dropdownSlide {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  color: rgba(255, 255, 255, 0.9);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.1s ease;
}

.dropdown-item:hover {
  background: #5865F2;
  color: white;
}

.dropdown-icon {
  width: 16px;
  height: 16px;
  color: rgba(255, 255, 255, 0.6);
  flex-shrink: 0;
}

.dropdown-item:hover .dropdown-icon {
  color: white;
}

/* 📦 Channels Container */
.channels-container {
  display: flex;
  flex-direction: column;
  padding: 0 8px;
}

/* 🔄 Loading State */
.loading-state {
  padding: 0 8px;
}

.channel-skeleton {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 8px;
  margin-bottom: 2px;
  border-radius: 6px;
}

.skeleton-icon {
  width: 20px;
  height: 20px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 4px;
  animation: shimmer 1.5s infinite;
}

.skeleton-text {
  flex: 1;
  height: 16px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 4px;
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% {
    opacity: 0.6;
  }

  50% {
    opacity: 1;
  }

  100% {
    opacity: 0.6;
  }
}

/* 🎯 Empty State - Minimal & Elegant */
.empty-state {
  padding: 16px 16px;
  text-align: center;
}

.empty-text {
  margin: 0 0 8px;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.4);
  font-weight: 400;
}

.empty-action {
  background: none;
  border: none;
  color: #5865F2;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.empty-action:hover {
  background: rgba(88, 101, 242, 0.1);
  color: #7289DA;
}

.empty-action:active {
  background: rgba(88, 101, 242, 0.15);
}

/* 🎬 Smooth Animations */
.channel-list-enter-active,
.channel-list-leave-active {
  transition: all 0.2s ease;
}

.channel-list-enter-from {
  opacity: 0;
  transform: translateX(-10px);
}

.channel-list-leave-to {
  opacity: 0;
  transform: translateX(10px);
}

.channel-list-move {
  transition: transform 0.2s ease;
}

/* 🔄 Spinning Animation */
.spinning {
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

/* 📱 Mobile Responsive Design */
@media (max-width: 768px) {
  .channel-section {
    margin-bottom: 20px;
  }

  .section-header {
    padding: 0 8px;
    height: 32px;
  }

  .section-toggle {
    font-size: 13px;
    padding: 6px;
    margin: -6px;
  }

  .section-count {
    font-size: 12px;
  }

  .action-btn {
    width: 24px;
    height: 24px;
  }

  .action-btn svg {
    width: 16px;
    height: 16px;
  }

  .dropdown-menu {
    min-width: 200px;
  }

  .dropdown-item {
    padding: 10px 14px;
    font-size: 15px;
  }

  .empty-state {
    padding: 20px 16px;
  }

  .empty-text {
    font-size: 14px;
  }

  .empty-action {
    font-size: 14px;
    padding: 6px 12px;
  }
}

/* 🎯 Touch-friendly enhancements */
@media (hover: none) and (pointer: coarse) {

  .section-toggle,
  .action-btn,
  .dropdown-item,
  .empty-action {
    min-height: 44px;
  }
}

/* 🎭 Reduced Motion */
@media (prefers-reduced-motion: reduce) {

  .toggle-icon,
  .action-btn,
  .dropdown-item,
  .empty-action {
    transition: none;
  }

  .dropdown-menu {
    animation: none;
  }

  .channel-list-enter-active,
  .channel-list-leave-active,
  .channel-list-move {
    transition: none;
  }
}

/* 🌙 Focus States for Accessibility */
.section-toggle:focus-visible,
.action-btn:focus-visible,
.dropdown-item:focus-visible,
.empty-action:focus-visible {
  outline: 2px solid rgba(88, 101, 242, 0.5);
  outline-offset: 2px;
}

/* 🎨 High Contrast Mode */
@media (prefers-contrast: high) {
  .section-toggle {
    color: white;
  }

  .section-count {
    color: rgba(255, 255, 255, 0.7);
  }

  .dropdown-menu {
    border: 2px solid white;
  }

  .empty-text {
    color: rgba(255, 255, 255, 0.7);
  }
}
</style>