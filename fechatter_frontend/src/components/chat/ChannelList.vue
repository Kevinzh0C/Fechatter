<template>
  <!-- 🎨 人体工学设计的频道列表 -->
  <div class="modern-channel-list">
    <!-- 🔄 优雅的加载状态 -->
    <div v-if="loading && allChats.length === 0" class="loading-container">
      <div v-for="i in 4" :key="`skeleton-${i}`" class="skeleton-item">
        <div class="skeleton-avatar"></div>
        <div class="skeleton-content">
          <div class="skeleton-name"></div>
          <div class="skeleton-message"></div>
        </div>
      </div>
    </div>

    <!-- 📋 频道列表 (Channels) -->
    <div v-else-if="listType === 'channels' && sortedChannels.length > 0" class="channels-container">
      <div v-for="channel in sortedChannels" :key="`channel-${channel.id}`" @click="navigateToChat(channel.id)"
        class="channel-card" :class="{ 'active': isActiveChat(channel.id) }" :title="channel.name">
        <!-- 频道图标 -->
        <div class="channel-icon-container">
          <div class="channel-icon" :class="{ 'private': isPrivateChannel(channel) }">
            <svg v-if="isPrivateChannel(channel)" class="icon-svg" fill="none" stroke="currentColor"
              viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
            <span v-else class="hash-symbol">#</span>
          </div>
        </div>

        <!-- 频道信息 -->
        <div class="channel-info">
          <div class="channel-header">
            <h3 class="channel-name">{{ channel.name }}</h3>
            <time v-if="channel.last_message" class="channel-time">
              {{ formatSmartTimestamp(channel.last_message.created_at) }}
            </time>
          </div>

          <!-- 最新消息预览 -->
          <div v-if="channel.last_message" class="channel-preview">
            <span class="sender">{{ getLastMessageSender(channel) }}</span>
            <span class="message">{{ truncateMessage(channel.last_message.content) }}</span>
          </div>
          <div v-else class="channel-empty">
            <span>暂无消息</span>
          </div>
        </div>

        <!-- 状态指示器 -->
        <div class="channel-status">
          <div v-if="channel.unread_count && channel.unread_count > 0" class="unread-indicator">
            {{ channel.unread_count > 99 ? '99+' : channel.unread_count }}
          </div>
          <div v-else-if="channel.member_count && channel.member_count > 0" class="member-indicator">
            <svg class="member-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
            </svg>
            <span>{{ channel.member_count }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 💬 直接消息列表 (DMs) -->
    <div v-else-if="listType === 'dms' && sortedDMs.length > 0" class="dms-container">
      <div v-for="dm in sortedDMs" :key="`dm-${dm.id}`" @click="navigateToChat(dm.id)" class="dm-card"
        :class="{ 'active': isActiveChat(dm.id) }" :title="getDMDisplayName(dm)">
        <!-- 用户头像 -->
        <div class="dm-avatar-container">
          <div class="dm-avatar">
            <span class="avatar-text">{{ getDMDisplayName(dm).charAt(0).toUpperCase() }}</span>
            <div class="presence-indicator" :class="getOnlineStatus(dm)"></div>
          </div>
        </div>

        <!-- 用户信息 -->
        <div class="dm-info">
          <div class="dm-header">
            <h3 class="dm-username">{{ getDMDisplayName(dm) }}</h3>
            <time v-if="dm.last_message" class="dm-time">
              {{ formatSmartTimestamp(dm.last_message?.created_at) }}
            </time>
          </div>

          <!-- 最新消息 -->
          <div class="dm-message">
            <div v-if="dm.last_message" class="message-content">
              <span v-if="dm.last_message.sender_id === authStore.user?.id" class="you-indicator">You:</span>
              <span class="message-text">{{ truncateMessage(dm.last_message.content, 28) }}</span>
            </div>
            <div v-else class="no-messages">
              <span>Start conversation</span>
            </div>
          </div>
        </div>

        <!-- 操作区域 -->
        <div class="dm-actions">
          <!-- 未读消息指示器 -->
          <div v-if="dm.unread_count && dm.unread_count > 0" class="unread-indicator">
            {{ dm.unread_count > 99 ? '99+' : dm.unread_count }}
          </div>

          <!-- 关闭按钮 -->
          <button @click.stop.prevent="closeDM(dm.id)" class="close-dm-btn" title="Close conversation"
            aria-label="Close conversation">
            <svg class="close-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- 🎭 空状态提示 -->
    <div v-else-if="!loading" class="empty-state">
      <div class="empty-illustration">
        <svg v-if="listType === 'channels'" class="empty-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
            d="M7 20l4-16m2 16l4-16M6 9h14M4 15h14" />
        </svg>
        <svg v-else class="empty-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
            d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
        </svg>
      </div>
      <h3 class="empty-title">
        {{ listType === 'channels' ? 'No channels yet' : 'No direct messages yet' }}
      </h3>
      <p class="empty-description">
        {{ listType === 'channels' ? 'Create or join a channel to start chatting' : 'Start a new direct conversation' }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, watch, ref, onMounted, onUnmounted } from 'vue';
import { useAuthStore } from '@/stores/auth';
import { useChatStore } from '@/stores/chat';
import { useToast } from '@/composables/useToast';
import { useRouter, useRoute } from 'vue-router';
import type { Chat } from '@/types/api';

const props = defineProps({
  listType: { type: String, required: true, validator: (v: string) => ['channels', 'dms'].includes(v) }
});

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();
const chatStore = useChatStore();
const { error: notifyError } = useToast();

const loading = ref(false);

// Navigation function
async function navigateToChat(chatId: number) {
  try {
    console.log('🔧 Navigating to chat:', chatId);
    await router.push(`/chat/${chatId}`);
  } catch (error) {
    console.error('Navigation error:', error);
    // Fallback to direct navigation
    window.location.href = `/chat/${chatId}`;
  }
}

// Check if chat is active
function isActiveChat(chatId: number): boolean {
  return route.params.id === String(chatId);
}

// Use chat store data with proper filtering
const allChats = computed(() => {
  // Use visible chats from store (excludes hidden DMs)
  return (chatStore.visibleChats as any[]) || (chatStore.chats as any[]) || [];
});

const sortedChannels = computed(() => {
  console.log('🔍 [ChannelList] All chats for debugging:', allChats.value);
  console.log('🔍 [ChannelList] Filtering for channels, listType:', props.listType);

  const filtered = allChats.value.filter(c => {
    const chat = c as any; // Type bypass
    console.log(`🔍 [ChannelList] Chat ${chat.id}: name="${chat.name}", chat_type="${chat.chat_type}"`);
    // Filter for channels (both public and private)
    return chat.chat_type === 'PublicChannel' ||
      chat.chat_type === 'PrivateChannel' ||
      chat.chat_type === 'channel' ||
      chat.chat_type === 'public_channel' ||
      chat.chat_type === 'private_channel';
  });

  console.log('🔍 [ChannelList] Filtered channels:', filtered);
  return filtered.sort((a: any, b: any) => (a.name || '').localeCompare(b.name || ''));
});

const sortedDMs = computed(() => {
  console.log('🔍 [ChannelList] Filtering for DMs, listType:', props.listType);

  const filtered = allChats.value.filter(c => {
    const chat = c as any; // Type bypass
    console.log(`🔍 [ChannelList] Chat ${chat.id}: name="${chat.name}", chat_type="${chat.chat_type}"`);
    // Filter for direct messages/single chats
    return chat.chat_type === 'Single' ||
      chat.chat_type === 'direct' ||
      chat.chat_type === 'single' ||
      chat.chat_type === 'direct_message';
  });

  console.log('🔍 [ChannelList] Filtered DMs:', filtered);
  return filtered.sort((a: any, b: any) => {
    const aTime = a.last_message?.created_at || a.updated_at || '';
    const bTime = b.last_message?.created_at || b.updated_at || '';
    return new Date(bTime).getTime() - new Date(aTime).getTime();
  });
});

function formatSmartTimestamp(timestamp?: string) {
  if (!timestamp) return '';
  const date = new Date(timestamp);
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const yesterday = new Date(today);
  yesterday.setDate(today.getDate() - 1);
  const sixDaysAgo = new Date(today);
  sixDaysAgo.setDate(today.getDate() - 6);

  if (date > today) {
    return date.toLocaleTimeString('zh-CN', { hour: 'numeric', minute: '2-digit', hour12: false });
  } else if (date > yesterday) {
    return 'Yesterday';
  } else if (date > sixDaysAgo) {
    return date.toLocaleDateString('zh-CN', { weekday: 'long' });
  } else {
    return date.toLocaleDateString('zh-CN', { year: 'numeric', month: 'numeric', day: 'numeric' });
  }
}

function getDMDisplayName(dm: Chat): string {
  // Use name if available
  if (dm.name && dm.name.trim() && dm.name !== 'Direct Message') {
    return dm.name;
  }

  // 如果没有名称，返回默认值
  return 'Direct Message';
}

// 辅助函数：判断是否为私有频道
function isPrivateChannel(chat: any): boolean {
  return chat.chat_type === 'PrivateChannel' ||
    chat.chat_type === 'private_channel' ||
    chat.is_public === false;
}

// 辅助函数：获取最后消息发送者
function getLastMessageSender(chat: any): string {
  if (!chat.last_message) return '';

  // 从 sender 对象获取名称
  if (chat.last_message.sender?.fullname) {
    return chat.last_message.sender.fullname;
  }

  // 备选：使用当前用户信息判断
  if (chat.last_message.sender_id === authStore.user?.id) {
    return 'You';
  }

  return 'Someone';
}

// 辅助函数：截断消息内容
function truncateMessage(content: string, maxLength: number = 30): string {
  if (!content) return '';
  if (content.length <= maxLength) return content;
  return content.substring(0, maxLength) + '...';
}

// 辅助函数：获取在线状态
function getOnlineStatus(dm: Chat): string {
  // TODO: 实际实现需要根据用户在线状态数据
  // 这里先返回随机状态用于演示
  const statuses = ['online', 'away', 'offline'];
  return statuses[dm.id % 3]; // 简单的演示逻辑
}

const fetchChats = async () => {
  try {
    loading.value = true;
    // Use chat store instead of direct service call
    await chatStore.fetchChats();
    // No need to set allChats since it's computed from store
  } catch (error: any) {
    console.error('获取聊天列表失败:', error);
    notifyError('获取聊天列表失败');
  } finally {
    loading.value = false;
  }
};

async function closeDM(dmId: number) {
  try {
    // Use chat store method instead
    await chatStore.leaveChat(dmId);
    // Store will automatically update the chats list
  } catch (error: any) {
    console.error('关闭私信失败:', error);
    notifyError('关闭私信失败');
  }
}



// 监听认证状态变化
watch(() => authStore.user, (newUser) => {
  if (newUser) {
    fetchChats();
  }
}, { immediate: true });

// Watch for chat store changes to ensure real-time updates
watch(() => chatStore.chats, () => {
  // Computed properties will automatically update when store changes
  // No need to manually update allChats since it's computed
}, { immediate: true, deep: true });

// Watch for loading state from chat store
watch(() => chatStore.loading, (newLoading) => {
  // Sync local loading state with store loading state
  if (newLoading !== undefined) {
    loading.value = newLoading;
  }
});

// 组件挂载时加载数据
onMounted(() => {
  if (authStore.user) {
    fetchChats();
  }

  // Set up real-time updates listener if available
  if (typeof window !== 'undefined') {
    // Listen for chat list refresh events
    const handleChatListRefresh = () => {
      fetchChats();
    };

    // Listen for chat store updates
    const handleChatsUpdated = (event: any) => {
      // The computed properties will automatically update
      // This is just for debugging/logging if needed
      console.log('📱 [ChannelList] Chats updated from store:', event.detail);
    };

    window.addEventListener('fechatter:refresh-chats', handleChatListRefresh);
    window.addEventListener('fechatter:chats-updated', handleChatsUpdated);

    // Cleanup on unmount
    onUnmounted(() => {
      window.removeEventListener('fechatter:refresh-chats', handleChatListRefresh);
      window.removeEventListener('fechatter:chats-updated', handleChatsUpdated);
    });
  }

  // 暴露刷新方法供父组件使用
  defineExpose({
    refresh: fetchChats
  });
});
</script>

<style scoped>
/* 🎨 现代人体工学设计的频道列表 */
.modern-channel-list {
  padding: 8px 12px;
  max-height: 100%;
  overflow-y: auto;
  scroll-behavior: smooth;
}

/* 🔄 优雅的加载骨架 */
.loading-container {
  padding: 8px 0;
}

.skeleton-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  margin-bottom: 8px;
  border-radius: 12px;
  background: linear-gradient(90deg,
      rgba(255, 255, 255, 0.06) 0%,
      rgba(255, 255, 255, 0.12) 50%,
      rgba(255, 255, 255, 0.06) 100%);
  background-size: 200% 100%;
  animation: skeleton-shimmer 1.8s ease-in-out infinite;
}

.skeleton-avatar {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.15);
  margin-right: 16px;
  flex-shrink: 0;
}

.skeleton-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-name {
  width: 60%;
  height: 16px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.15);
}

.skeleton-message {
  width: 80%;
  height: 14px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.1);
}

@keyframes skeleton-shimmer {
  0% {
    background-position: -200% 0;
  }

  100% {
    background-position: 200% 0;
  }
}

/* 📋 频道卡片容器 */
.channels-container,
.dms-container {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

/* 🎯 频道卡片 - 人体工学设计 */
.channel-card,
.dm-card {
  display: flex;
  align-items: center;
  padding: 14px 16px;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  user-select: none;
  position: relative;

  /* 确保足够的触摸目标 (最小44px高度) */
  min-height: 56px;

  /* 背景和边框 */
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid transparent;

  /* 焦点可访问性 */
  outline: none;
}

.channel-card:hover,
.dm-card:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.1);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.channel-card:focus,
.dm-card:focus {
  outline: 2px solid #5865f2;
  outline-offset: 2px;
}

/* 激活状态 */
.channel-card.active,
.dm-card.active {
  background: linear-gradient(135deg, #5865f2, #7983f5);
  border-color: #7983f5;
  color: white;
  box-shadow: 0 2px 8px rgba(88, 101, 242, 0.3);
}

.channel-card.active:hover,
.dm-card.active:hover {
  transform: translateY(0);
  background: linear-gradient(135deg, #4752c4, #6366f1);
}

/* 🎭 频道图标容器 */
.channel-icon-container {
  margin-right: 16px;
  flex-shrink: 0;
}

.channel-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.7);
  transition: all 0.2s ease;
  font-size: 20px;
  font-weight: 600;
}

.channel-icon.private {
  background: linear-gradient(135deg, #f59e0b, #d97706);
  color: white;
}

.channel-icon:not(.private) {
  background: linear-gradient(135deg, #6b7280, #9ca3af);
  color: white;
}

.icon-svg {
  width: 20px;
  height: 20px;
}

.hash-symbol {
  font-size: 24px;
  line-height: 1;
}

/* 激活状态下的图标 */
.active .channel-icon {
  background: rgba(255, 255, 255, 0.2);
  color: white;
  transform: scale(1.05);
}

/* 📝 频道信息区域 */
.channel-info,
.dm-info {
  flex: 1;
  min-width: 0;
  overflow: hidden;
}

.channel-header,
.dm-header {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 4px;
}

.channel-name,
.dm-username {
  font-size: 16px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  line-height: 1.2;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin: 0;
}

.active .channel-name,
.active .dm-username {
  color: white;
  font-weight: 700;
}

.channel-time,
.dm-time {
  font-size: 12px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.5);
  flex-shrink: 0;
  margin-left: 12px;
  line-height: 1;
}

.active .channel-time,
.active .dm-time {
  color: rgba(255, 255, 255, 0.8);
}

/* 消息预览 */
.channel-preview,
.dm-message {
  display: flex;
  align-items: center;
  gap: 4px;
  overflow: hidden;
}

.channel-preview .sender,
.you-indicator {
  font-size: 13px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.7);
  flex-shrink: 0;
}

.channel-preview .message,
.message-text {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.6);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
}

.channel-empty,
.no-messages {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.4);
  font-style: italic;
}

.active .channel-preview .sender,
.active .you-indicator {
  color: rgba(255, 255, 255, 0.9);
}

.active .channel-preview .message,
.active .message-text {
  color: rgba(255, 255, 255, 0.8);
}

/* 📊 状态指示器和操作区域 */
.channel-status,
.dm-actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
  margin-left: 12px;
  flex-shrink: 0;
}

.unread-indicator {
  background: linear-gradient(135deg, #ed4245, #c73a3e);
  color: white;
  font-size: 11px;
  font-weight: 700;
  border-radius: 12px;
  padding: 4px 8px;
  min-width: 20px;
  text-align: center;
  line-height: 1;
  box-shadow: 0 2px 4px rgba(237, 66, 69, 0.3);
}

.member-indicator {
  display: flex;
  align-items: center;
  gap: 4px;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.7);
  font-size: 11px;
  font-weight: 600;
  border-radius: 8px;
  padding: 4px 8px;
  line-height: 1;
}

.member-icon {
  width: 12px;
  height: 12px;
}

/* 👤 用户头像设计 */
.dm-avatar-container {
  margin-right: 16px;
  flex-shrink: 0;
  position: relative;
}

.dm-avatar {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  background: linear-gradient(135deg, #5865f2, #7983f5);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  box-shadow: 0 2px 8px rgba(88, 101, 242, 0.2);
  transition: all 0.2s ease;
}

.dm-avatar:hover {
  transform: scale(1.05);
  box-shadow: 0 4px 12px rgba(88, 101, 242, 0.3);
}

.avatar-text {
  font-size: 18px;
  font-weight: 700;
  color: white;
  text-transform: uppercase;
  line-height: 1;
}

/* 🟢 在线状态指示器 */
.presence-indicator {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 3px solid var(--bg-secondary);
  transition: all 0.2s ease;
}

.presence-indicator.online {
  background: #23a55a;
  box-shadow: 0 0 8px rgba(35, 165, 90, 0.5);
}

.presence-indicator.away {
  background: #f0b132;
  box-shadow: 0 0 8px rgba(240, 177, 50, 0.5);
}

.presence-indicator.offline {
  background: #80848e;
}

/* ❌ 关闭按钮 */
.close-dm-btn {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.1);
  border: none;
  color: rgba(255, 255, 255, 0.6);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: all 0.2s ease;
  margin-left: 8px;
}

.dm-card:hover .close-dm-btn {
  opacity: 1;
}

.close-dm-btn:hover {
  background: rgba(237, 66, 69, 0.8);
  color: white;
  transform: scale(1.1);
}

.close-dm-btn:focus {
  outline: 2px solid #ed4245;
  outline-offset: 2px;
  opacity: 1;
}

.close-icon {
  width: 16px;
  height: 16px;
}

/* 🎭 空状态设计 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  text-align: center;
  color: rgba(255, 255, 255, 0.6);
}

.empty-illustration {
  width: 80px;
  height: 80px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.05);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 24px;
}

.empty-icon {
  width: 40px;
  height: 40px;
  color: rgba(255, 255, 255, 0.4);
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.8);
  margin: 0 0 8px 0;
  line-height: 1.3;
}

.empty-description {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.5);
  margin: 0;
  line-height: 1.4;
  max-width: 200px;
}

/* 📱 响应式设计 */
@media (max-width: 768px) {
  .modern-channel-list {
    padding: 4px 8px;
  }

  .channel-card,
  .dm-card {
    padding: 12px 14px;
    min-height: 52px;
  }

  .channel-icon,
  .dm-avatar {
    width: 44px;
    height: 44px;
  }

  .channel-name,
  .dm-username {
    font-size: 15px;
  }

  .channel-preview .message,
  .message-text {
    font-size: 12px;
  }
}

/* 🎯 高对比度支持 */
@media (prefers-contrast: high) {

  .channel-card,
  .dm-card {
    border: 2px solid rgba(255, 255, 255, 0.2);
  }

  .channel-card.active,
  .dm-card.active {
    border-color: #ffffff;
  }
}

/* 🎬 减少动画 */
@media (prefers-reduced-motion: reduce) {

  .channel-card,
  .dm-card,
  .channel-icon,
  .dm-avatar,
  .close-dm-btn {
    transition: none;
  }

  .skeleton-item {
    animation: none;
  }
}
</style>