<template>
  <!-- 现代人体工学设计的侧边栏 -->
  <aside class="modern-sidebar">
    <!-- 📍 工作区域头部 -->
    <header class="workspace-header" @click="toggleWorkspaceMenu">
      <h1 class="workspace-name">{{ workspaceName }}</h1>
      <Icon name="chevron-down" class="workspace-chevron" :class="{ 'rotated': isWorkspaceMenuOpen }" />
    </header>

    <!-- 🎛️ 下拉菜单 -->
    <Transition name="workspace-dropdown">
      <div v-if="isWorkspaceMenuOpen" class="workspace-dropdown">
        <nav class="dropdown-menu">
          <button @click="invitePeople" class="menu-item">
            <Icon name="user-plus" />
            <span>Invite People</span>
          </button>
          <button @click="workspaceSettings" class="menu-item">
            <Icon name="settings" />
            <span>Workspace Settings</span>
          </button>
          <button @click="createWorkspace" class="menu-item">
            <Icon name="plus-circle" />
            <span>Create Workspace</span>
          </button>
        </nav>
      </div>
    </Transition>

    <!-- 📋 导航区域 -->
    <nav class="sidebar-navigation">
      <!-- 📢 公开频道 -->
      <section class="nav-section">
        <div class="section-header">
          <div class="section-title">
            <svg class="section-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M7 20l4-16m2 16l4-16M6 9h14M4 15h14" />
            </svg>
            <h2>Public Channels</h2>
          </div>
          <button @click="addPublicChannel" class="add-button" title="Create public channel"
            aria-label="Create public channel">
            <svg class="add-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>

        <ul class="channel-list">
          <li v-for="channel in publicChannels" :key="`public-${channel.id}`">
            <button @click="selectChannel(channel.id)" class="channel-item"
              :class="{ 'active': channel.id === activeChannelId }" :aria-label="`Public channel ${channel.name}`">
              <div class="channel-icon public">
                <span class="hash">#</span>
              </div>
              <div class="channel-info">
                <span class="channel-name">{{ channel.name }}</span>
              </div>
              <div v-if="channel.unread_count > 0" class="unread-badge">
                {{ channel.unread_count > 99 ? '99+' : channel.unread_count }}
              </div>
            </button>
          </li>
        </ul>
      </section>

      <!-- 🔒 私有频道 -->
      <section class="nav-section">
        <div class="section-header">
          <div class="section-title">
            <svg class="section-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
            <h2>Private Channels</h2>
          </div>
          <button @click="addPrivateChannel" class="add-button" title="Create private channel"
            aria-label="Create private channel">
            <svg class="add-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>

        <ul class="channel-list">
          <li v-for="channel in privateChannels" :key="`private-${channel.id}`">
            <button @click="selectChannel(channel.id)" class="channel-item"
              :class="{ 'active': channel.id === activeChannelId }" :aria-label="`Private channel ${channel.name}`">
              <div class="channel-icon private">
                <svg class="lock-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
              </div>
              <div class="channel-info">
                <span class="channel-name">{{ channel.name }}</span>
              </div>
              <div v-if="channel.unread_count > 0" class="unread-badge">
                {{ channel.unread_count > 99 ? '99+' : channel.unread_count }}
              </div>
            </button>
          </li>
        </ul>
      </section>

      <!-- 👥 群组聊天 -->
      <section class="nav-section">
        <div class="section-header">
          <div class="section-title">
            <svg class="section-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
            </svg>
            <h2>群组聊天</h2>
          </div>
          <button @click="addGroupChat" class="add-button" title="创建群组聊天" aria-label="创建群组聊天">
            <svg class="add-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>

        <ul class="channel-list">
          <li v-for="group in groupChats" :key="`group-${group.id}`">
            <button @click="selectChannel(group.id)" class="channel-item"
              :class="{ 'active': group.id === activeChannelId }" :aria-label="`群组聊天 ${group.name}`">
              <div class="group-avatars">
                <div v-for="(member, index) in group.members.slice(0, 2)" :key="member.id" class="member-avatar"
                  :style="{ zIndex: 10 - index }">
                  {{ member.name.charAt(0).toUpperCase() }}
                </div>
                <div v-if="group.members.length > 2" class="more-members">
                  +{{ group.members.length - 2 }}
                </div>
              </div>
              <div class="channel-info">
                <span class="channel-name">{{group.name || group.members.map(m => m.name).join(', ')}}</span>
              </div>
              <div v-if="group.unread_count > 0" class="unread-badge">
                {{ group.unread_count > 99 ? '99+' : group.unread_count }}
              </div>
            </button>
          </li>
        </ul>
      </section>

      <!-- MESSAGE: 私信 -->
      <section class="nav-section direct-messages">
        <div class="section-header">
          <div class="section-title">
            <svg class="section-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
            </svg>
            <h2>Direct Messages</h2>
          </div>
          <button @click="addDirectMessage" class="add-button" title="Start direct message"
            aria-label="Start direct message">
            <svg class="add-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>

        <div class="dm-container">
          <ul class="dm-list">
            <li v-for="dm in directMessages" :key="`dm-${dm.id}`">
              <button @click="selectChannel(dm.id)" class="dm-item" :class="{ 'active': dm.id === activeChannelId }"
                :aria-label="`Direct message with ${dm.name}`">
                <div class="dm-avatar-container">
                  <div class="dm-avatar">
                    <span class="avatar-text">{{ dm.name.charAt(0).toUpperCase() }}</span>
                    <div class="presence-indicator" :class="{ 'online': dm.online }"></div>
                  </div>
                </div>
                <div class="dm-info">
                  <span class="dm-name">{{ dm.name }}</span>
                </div>
                <div v-if="dm.unread_count > 0" class="unread-badge">
                  {{ dm.unread_count > 99 ? '99+' : dm.unread_count }}
                </div>
              </button>
            </li>
          </ul>
        </div>
      </section>
    </nav>
  </aside>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useWorkspaceStore } from '@/stores/workspace';
import ChannelList from '@/components/layout/ChannelList.vue';
import Icon from '@/components/ui/Icon.vue';

const workspaceStore = useWorkspaceStore();
const isWorkspaceMenuOpen = ref(false);

const props = defineProps({
  workspaceName: {
    type: String,
    default: 'Fechatter 工作区'
  },
  channels: {
    type: Array,
    default: () => []
  },
  directMessages: {
    type: Array,
    default: () => []
  },
  groupChats: {
    type: Array,
    default: () => []
  },
  activeChannelId: {
    type: String,
    default: null
  }
});

const emit = defineEmits([
  'logout',
  'add-public-channel',
  'add-private-channel',
  'add-group-chat',
  'add-direct-message',
  'select-channel',
  'go-to-admin'
]);

// 分离不同类型的频道
const publicChannels = computed(() => {
  return props.channels.filter(channel => channel.type === 'public' || !channel.type);
});

const privateChannels = computed(() => {
  return props.channels.filter(channel => channel.type === 'private');
});

// 切换下拉菜单
function toggleWorkspaceMenu() {
  isWorkspaceMenuOpen.value = !isWorkspaceMenuOpen.value;
}

// 处理登出
function handleLogout() {
  emit('logout');
  isWorkspaceMenuOpen.value = false;
}

// 前往管理后台
function goToAdmin() {
  emit('go-to-admin');
  isWorkspaceMenuOpen.value = false;
}

// 添加频道方法
function addPublicChannel() {
  emit('add-public-channel');
}

function addPrivateChannel() {
  emit('add-private-channel');
}

function addGroupChat() {
  emit('add-group-chat');
}

function addDirectMessage() {
  emit('add-direct-message');
}

// 选择频道
function selectChannel(channelId) {
  emit('select-channel', channelId);
}

// 处理外部点击关闭下拉菜单
function handleOutsideClick(event) {
  if (!event.target.closest('.workspace-header')) {
    isWorkspaceMenuOpen.value = false;
  }
}

// 生命周期
onMounted(() => {
  document.addEventListener('click', handleOutsideClick);
});

onUnmounted(() => {
  document.removeEventListener('click', handleOutsideClick);
});
</script>

<style scoped>
/* 现代人体工学侧边栏设计 */
.modern-sidebar {
  width: 280px;
  height: 100vh;
  background: linear-gradient(180deg, #1e1f22 0%, #2b2d31 100%);
  color: #ffffff;
  display: flex;
  flex-direction: column;
  border-right: 1px solid rgba(255, 255, 255, 0.1);
  position: relative;
  overflow: hidden;
}

/* 📍 工作区域头部 */
.workspace-header {
  padding: 16px;
  cursor: pointer;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.02);
  position: relative;
}

.workspace-name {
  font-size: 16px;
  font-weight: 600;
  color: #ffffff;
  margin: 0;
  line-height: 1.2;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.workspace-chevron {
  width: 20px;
  height: 20px;
  color: rgba(255, 255, 255, 0.6);
  transition: all 0.2s ease;
  transform-origin: center;
}

.workspace-chevron.rotated {
  transform: rotate(180deg);
  color: #5865f2;
}

/* 🎛️ 下拉菜单 */
.workspace-dropdown {
  position: absolute;
  top: calc(100% + 8px);
  left: 20px;
  right: 20px;
  background: #1e1f22;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  z-index: 1000;
  overflow: hidden;
}

.dropdown-menu {
  padding: 8px;
}

.menu-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.8);
  font-size: 14px;
  font-weight: 500;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
}

.menu-item:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #ffffff;
}

.menu-item.logout:hover {
  background: rgba(237, 66, 69, 0.1);
  color: #ed4245;
}

.menu-icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
}

.menu-divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.1);
  margin: 8px 0;
}

/* 🎬 下拉菜单动画 */
.workspace-dropdown-enter-active,
.workspace-dropdown-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.workspace-dropdown-enter-from,
.workspace-dropdown-leave-to {
  opacity: 0;
  transform: translateY(-8px) scale(0.95);
}

.workspace-dropdown-enter-to,
.workspace-dropdown-leave-from {
  opacity: 1;
  transform: translateY(0) scale(1);
}

/* 📋 导航区域 */
.sidebar-navigation {
  flex: 1;
  overflow-y: auto;
  padding: 16px 0;
  scroll-behavior: smooth;
}

.nav-section {
  margin-bottom: 24px;
  padding: 0 20px;
}

.nav-section.direct-messages {
  flex: 1;
  margin-bottom: 0;
}

/* 📂 节区头部 */
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  padding: 8px 0;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.section-title h2 {
  font-size: 13px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.7);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin: 0;
  line-height: 1;
}

.section-icon {
  width: 16px;
  height: 16px;
  color: rgba(255, 255, 255, 0.5);
}

.add-button {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.05);
  border: none;
  color: rgba(255, 255, 255, 0.6);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.add-button:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #ffffff;
  transform: scale(1.05);
}

.add-button:focus {
  outline: 2px solid #5865f2;
  outline-offset: 2px;
}

.add-icon {
  width: 16px;
  height: 16px;
}

/* 📋 频道列表 */
.channel-list,
.dm-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.channel-list li,
.dm-list li {
  margin-bottom: 4px;
}

/* 频道项目 */
.channel-item,
.dm-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.8);
  font-size: 15px;
  font-weight: 500;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
  position: relative;

  /* 确保足够的触摸目标 */
  min-height: 44px;
}

.channel-item:hover,
.dm-item:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #ffffff;
  transform: translateX(4px);
}

.channel-item:focus,
.dm-item:focus {
  outline: 2px solid #5865f2;
  outline-offset: 2px;
}

/* 激活状态 */
.channel-item.active,
.dm-item.active {
  background: linear-gradient(90deg, #5865f2, #7983f5);
  color: #ffffff;
  font-weight: 600;
  box-shadow: 0 2px 8px rgba(88, 101, 242, 0.3);
}

.channel-item.active:hover,
.dm-item.active:hover {
  transform: translateX(0);
  background: linear-gradient(90deg, #4752c4, #6366f1);
}

/* 🎭 频道图标 */
.channel-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.2s ease;
}

.channel-icon.public {
  background: linear-gradient(135deg, #6b7280, #9ca3af);
  color: #ffffff;
}

.channel-icon.private {
  background: linear-gradient(135deg, #f59e0b, #d97706);
  color: #ffffff;
}

.hash {
  font-size: 18px;
  font-weight: 700;
  line-height: 1;
}

.lock-icon {
  width: 16px;
  height: 16px;
}

/* 激活状态下的图标 */
.active .channel-icon {
  background: rgba(255, 255, 255, 0.2);
  transform: scale(1.1);
}

/* 频道信息 */
.channel-info,
.dm-info {
  flex: 1;
  min-width: 0;
}

.channel-name,
.dm-name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.2;
}

/* 👥 群组头像 */
.group-avatars {
  display: flex;
  align-items: center;
  gap: -4px;
  margin-right: 8px;
}

.member-avatar {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  background: linear-gradient(135deg, #5865f2, #7983f5);
  color: #ffffff;
  font-size: 10px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px solid #1e1f22;
  position: relative;
}

.more-members {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.8);
  font-size: 9px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px solid #1e1f22;
}

/* USER: 私信头像 */
.dm-container {
  flex: 1;
  overflow-y: auto;
  padding-right: 8px;
}

.dm-avatar-container {
  position: relative;
  margin-right: 4px;
  flex-shrink: 0;
}

.dm-avatar {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: linear-gradient(135deg, #5865f2, #7983f5);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  box-shadow: 0 2px 4px rgba(88, 101, 242, 0.2);
  transition: all 0.2s ease;
}

.avatar-text {
  font-size: 14px;
  font-weight: 700;
  color: #ffffff;
  line-height: 1;
}

/* 🟢 在线状态 */
.presence-indicator {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #80848e;
  border: 2px solid #1e1f22;
  transition: all 0.2s ease;
}

.presence-indicator.online {
  background: #23a55a;
  box-shadow: 0 0 8px rgba(35, 165, 90, 0.5);
}

/* 未读消息指示器 */
.unread-badge {
  background: linear-gradient(135deg, #ed4245, #c73a3e);
  color: #ffffff;
  font-size: 11px;
  font-weight: 700;
  border-radius: 12px;
  padding: 4px 8px;
  min-width: 18px;
  text-align: center;
  line-height: 1;
  box-shadow: 0 2px 4px rgba(237, 66, 69, 0.3);
  flex-shrink: 0;
}

/* 📱 响应式设计 */
@media (max-width: 1024px) {
  .modern-sidebar {
    width: 260px;
  }

  .workspace-header {
    padding: 12px 16px;
  }

  .nav-section {
    padding: 0 16px;
  }
}

@media (max-width: 768px) {
  .modern-sidebar {
    width: 240px;
    position: fixed;
    left: 0;
    top: 0;
    z-index: 1000;
    box-shadow: 4px 0 12px rgba(0, 0, 0, 0.3);
  }

  .channel-item,
  .dm-item {
    padding: 8px 10px;
    min-height: 40px;
  }

  .channel-icon,
  .dm-avatar {
    width: 32px;
    height: 32px;
  }
}

/* 高对比度支持 */
@media (prefers-contrast: high) {
  .modern-sidebar {
    border-right: 2px solid rgba(255, 255, 255, 0.3);
  }

  .channel-item,
  .dm-item {
    border: 1px solid transparent;
  }

  .channel-item.active,
  .dm-item.active {
    border-color: #ffffff;
  }
}

/* 🎬 减少动画 */
@media (prefers-reduced-motion: reduce) {

  .channel-item,
  .dm-item,
  .workspace-header,
  .add-button,
  .workspace-chevron {
    transition: none;
  }

  .workspace-dropdown-enter-active,
  .workspace-dropdown-leave-active {
    transition: none;
  }
}

/* 滚动条样式 */
.sidebar-navigation::-webkit-scrollbar,
.dm-container::-webkit-scrollbar {
  width: 6px;
}

.sidebar-navigation::-webkit-scrollbar-track,
.dm-container::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 3px;
}

.sidebar-navigation::-webkit-scrollbar-thumb,
.dm-container::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 3px;
}

.sidebar-navigation::-webkit-scrollbar-thumb:hover,
.dm-container::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}
</style>