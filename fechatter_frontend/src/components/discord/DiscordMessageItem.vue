<template>
  <div 
    class="discord-message discord-fade-in"
    :class="{ 
      'is-own': isOwnMessage,
      'is-sending': message.status === 'sending',
      'is-failed': message.status === 'failed',
      'is-highlighted': isHighlighted,
      'is-compact': isCompact
    }"
    @mouseenter="showToolbar = true"
    @mouseleave="showToolbar = false"
    :data-message-id="message.id || message.temp_id"
    role="article"
    :aria-label="`Message from ${senderName}, sent at ${formattedTime}`"
    :aria-describedby="`message-content-${message.id || message.temp_id}`"
    tabindex="0"
  >
    <!-- 🎯 三槽布局：Avatar + Meta + Content -->
    <div class="message-layout">
      <!-- 头像槽 -->
      <div class="message-avatar-slot">
        <DiscordAvatar
          :src="message.sender?.avatar_url"
          :name="senderName"
          :user-id="message.sender?.id"
          :status="getUserStatus(message.sender?.id)"
          :show-status="showUserStatus"
          :clickable="!isOwnMessage"
          size="medium"
          @click="handleAvatarClick"
          :aria-label="`${senderName}'s avatar`"
          role="button"
          :tabindex="!isOwnMessage ? 0 : -1"
        />
      </div>

      <!-- 消息内容槽 -->
      <div class="message-content-slot">
        <!-- Meta信息：用户名 + 时间戳 + 状态 -->
        <div class="message-meta">
          <span 
            class="sender-name"
            :style="{ color: getUserColor(message.sender) }"
            @click="handleSenderClick"
          >
            {{ senderName }}
          </span>
          
          <span class="message-timestamp">
            {{ formattedTime }}
          </span>
          
          <!-- 消息状态指示器 -->
          <div v-if="isOwnMessage && message.status" class="message-status">
            <div 
              v-if="message.status === 'sending'"
              class="status-sending"
              title="Sending..."
            >
              <div class="spinner"></div>
            </div>
            <div 
              v-else-if="message.status === 'failed'"
              class="status-failed"
              title="Failed to send"
            >
              <ExclamationTriangleIcon class="w-4 h-4" />
            </div>
            <div 
              v-else-if="message.status === 'sent'"
              class="status-sent"
              title="Sent"
            >
              <CheckIcon class="w-4 h-4" />
            </div>
          </div>

          <!-- 编辑标识 -->
          <span v-if="message.edited_at" class="edited-indicator" title="Edited">
            (edited)
          </span>
        </div>

        <!-- 消息内容 -->
        <div class="message-body">
          <!-- 回复引用 -->
          <div v-if="message.reply_to" class="message-reply" @click="scrollToMessage(message.reply_to.id)">
            <div class="reply-line"></div>
            <div class="reply-content">
              <DiscordAvatar
                :src="message.reply_to.sender?.avatar_url"
                :name="message.reply_to.sender?.fullname || 'Unknown'"
                :user-id="message.reply_to.sender?.id"
                size="small"
              />
              <span class="reply-author">{{ message.reply_to.sender?.fullname || 'Unknown' }}</span>
              <span class="reply-text">{{ truncateReply(message.reply_to.content) }}</span>
            </div>
          </div>

          <!-- 主要文本内容 -->
          <div 
            v-if="message.content" 
            class="message-text"
            :id="`message-content-${message.id || message.temp_id}`"
            role="region"
            :aria-label="`Message content from ${senderName}`"
          >
            <DiscordMarkdown
              :content="message.content"
              @link-click="handleLinkClick"
              @mention-click="handleMentionClick"
              @channel-click="handleChannelClick"
            />
          </div>

          <!-- 文件附件 -->
          <div v-if="message.files && message.files.length > 0" class="message-attachments">
            <div 
              v-for="(file, index) in message.files" 
              :key="file.id || index" 
              class="attachment-item"
            >
              <!-- 图片附件 -->
              <div 
                v-if="isImageFile(file)" 
                class="image-attachment"
                @click="openImageViewer(index)"
              >
                <img 
                  :src="getFileUrl(file)"
                  :alt="file.filename || 'Image'"
                  class="attachment-image"
                  loading="lazy"
                />
                <div class="image-overlay">
                  <ArrowsPointingOutIcon class="w-5 h-5" />
                </div>
              </div>
              
              <!-- 一般文件附件 -->
              <a 
                v-else 
                :href="getFileUrl(file)"
                :download="getFileName(file)"
                class="file-attachment"
                target="_blank"
                rel="noopener noreferrer"
              >
                <div class="file-icon">
                  <DocumentIcon class="w-5 h-5" />
                </div>
                <div class="file-details">
                  <div class="file-name">{{ getFileName(file) }}</div>
                  <div class="file-size">{{ getFileSize(file) }}</div>
                </div>
                <ArrowDownTrayIcon class="download-icon w-4 h-4" />
              </a>
            </div>
          </div>

          <!-- 错误重试 -->
          <div v-if="message.status === 'failed' && isOwnMessage" class="message-error">
            <span class="error-text">Failed to send message</span>
            <button @click="retryMessage" class="retry-button">
              <ArrowPathIcon class="w-4 h-4" />
              Retry
            </button>
          </div>

          <!-- 反应表情 -->
          <div v-if="message.reactions && message.reactions.length > 0" class="message-reactions">
            <div 
              v-for="reaction in message.reactions" 
              :key="reaction.emoji"
              class="reaction-item"
              :class="{ 'is-reacted': reaction.hasReacted }"
              @click="toggleReaction(reaction.emoji)"
            >
              <span class="reaction-emoji">{{ reaction.emoji }}</span>
              <span class="reaction-count">{{ reaction.count }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 🎯 Discord风格工具栏 -->
    <Transition name="toolbar">
      <div 
        v-if="showToolbar" 
        class="message-toolbar"
        role="toolbar"
        :aria-label="`Message actions for ${senderName}'s message`"
      >
        <div class="toolbar-actions">
          <button 
            class="toolbar-btn" 
            title="Add reaction" 
            @click="showEmojiPicker = true"
            aria-label="Add reaction to this message"
            role="button"
          >
            <FaceSmileIcon class="w-4 h-4" aria-hidden="true" />
          </button>
          <button 
            class="toolbar-btn" 
            title="Reply" 
            @click="handleReply"
            aria-label="Reply to this message"
            role="button"
          >
            <ArrowUturnLeftIcon class="w-4 h-4" aria-hidden="true" />
          </button>
          <button 
            class="toolbar-btn" 
            title="Copy message link" 
            @click="copyMessageLink"
            aria-label="Copy link to this message"
            role="button"
          >
            <LinkIcon class="w-4 h-4" aria-hidden="true" />
          </button>
          <button 
            class="toolbar-btn" 
            title="More actions" 
            @click="showContextMenu"
            aria-label="More message actions"
            role="button"
            aria-haspopup="menu"
          >
            <EllipsisHorizontalIcon class="w-4 h-4" aria-hidden="true" />
          </button>
        </div>
      </div>
    </Transition>

    <!-- Emoji选择器 -->
    <div 
      v-if="showEmojiPicker" 
      class="emoji-picker-overlay" 
      @click="showEmojiPicker = false"
      role="dialog"
      aria-label="Choose an emoji reaction"
      aria-modal="true"
    >
      <div class="emoji-picker" @click.stop>
        <div class="emoji-grid" role="grid" aria-label="Emoji reactions">
          <button 
            v-for="emoji in commonEmojis" 
            :key="emoji"
            class="emoji-btn"
            @click="addReaction(emoji)"
            :aria-label="`React with ${emoji}`"
            role="gridcell"
          >
            <span aria-hidden="true">{{ emoji }}</span>
          </button>
        </div>
      </div>
    </div>
    
    <!-- Image Viewer -->
    <ImageViewer
      v-if="imageFiles.length > 0"
      ref="imageViewerRef"
      :images="imageFiles"
      :initial-index="0"
    />
  </div>
</template>

<script setup>
import { ref, computed, nextTick } from 'vue';
import { 
  ExclamationTriangleIcon, 
  CheckIcon,
  FaceSmileIcon,
  ArrowUturnLeftIcon,
  LinkIcon,
  EllipsisHorizontalIcon,
  DocumentIcon,
  ArrowDownTrayIcon,
  ArrowsPointingOutIcon,
  ArrowPathIcon
} from '@heroicons/vue/24/outline';

import DiscordAvatar from './DiscordAvatar.vue';
import DiscordMarkdown from './DiscordMarkdown.vue';
import ImageViewer from '@/components/common/ImageViewer.vue';
import { useChatStore } from '@/stores/chat';

const props = defineProps({
  message: {
    type: Object,
    required: true
  },
  currentUserId: {
    type: Number,
    required: false,
    default: 0
  },
  chatId: {
    type: [Number, String],
    default: null
  },
  isHighlighted: {
    type: Boolean,
    default: false
  },
  isCompact: {
    type: Boolean,
    default: false
  },
  showUserStatus: {
    type: Boolean,
    default: true
  }
});

const emit = defineEmits([
  'user-profile-opened', 
  'dm-created', 
  'reply', 
  'scroll-to-message',
  'context-menu'
]);

const chatStore = useChatStore();

// 响应式状态
const showToolbar = ref(false);
const showEmojiPicker = ref(false);
const imageViewerRef = ref(null);

// 计算属性
const isOwnMessage = computed(() => props.message.sender?.id === props.currentUserId);

const senderName = computed(() => {
  return props.message.sender?.fullname || 
         props.message.sender?.username || 
         'Unknown User';
});

// 计算图片文件列表
const imageFiles = computed(() => {
  if (!props.message.files || !Array.isArray(props.message.files)) return [];
  
  return props.message.files
    .filter(file => isImageFile(file))
    .map(file => {
      // 转换为ImageViewer期望的格式
      if (typeof file === 'string') {
        return {
          url: getFileUrl(file),
          filename: getFileName(file)
        };
      }
      return {
        url: getFileUrl(file),
        filename: file.filename || getFileName(file),
        size: file.size
      };
    });
});

const formattedTime = computed(() => {
  if (!props.message.created_at) return '';
  
  const date = new Date(props.message.created_at);
  const now = new Date();
  const diffInHours = (now - date) / (1000 * 60 * 60);
  
  if (diffInHours < 24) {
    return date.toLocaleTimeString('en-US', { 
      hour: '2-digit', 
      minute: '2-digit',
      hour12: false 
    });
  } else if (diffInHours < 24 * 7) {
    return date.toLocaleDateString('en-US', { 
      weekday: 'short',
      hour: '2-digit', 
      minute: '2-digit',
      hour12: false 
    });
  } else {
    return date.toLocaleDateString('en-US', { 
      month: 'short', 
      day: 'numeric',
      hour: '2-digit', 
      minute: '2-digit',
      hour12: false 
    });
  }
});

// Discord风格的用户颜色
const discordColors = [
  '#f23f43', '#f0b232', '#23a55a', '#5865f2', '#eb459e',
  '#3ba55c', '#faa61a', '#ed4245', '#9146ff', '#00d4aa'
];

const getUserColor = (sender) => {
  if (!sender || !sender.id) return '#5865f2';
  const index = sender.id % discordColors.length;
  return discordColors[index];
};

const getUserStatus = (userId) => {
  // 这里可以从在线用户状态获取
  return 'online'; // 默认在线
};

// 常用表情
const commonEmojis = ['👍', '❤️', '😂', '😮', '😢', '😡', '👎', '🎉'];

// 事件处理方法
const handleAvatarClick = () => {
  if (!isOwnMessage.value) {
    emit('user-profile-opened', props.message.sender);
  }
};

const handleSenderClick = () => {
  if (!isOwnMessage.value) {
    emit('user-profile-opened', props.message.sender);
  }
};

const handleReply = () => {
  emit('reply', props.message);
};

const handleLinkClick = (url) => {
  window.open(url, '_blank', 'noopener,noreferrer');
};

const handleMentionClick = (userId) => {
  // 处理@用户点击
  console.log('Mention clicked:', userId);
};

const handleChannelClick = (channelId) => {
  // 处理#频道点击
  console.log('Channel clicked:', channelId);
};

const copyMessageLink = async () => {
  try {
    const url = `${window.location.origin}${window.location.pathname}?message=${props.message.id}`;
    await navigator.clipboard.writeText(url);
    // 可以显示一个提示
  } catch (error) {
    console.error('Failed to copy message link:', error);
  }
};

const showContextMenu = (event) => {
  emit('context-menu', {
    message: props.message,
    event
  });
};

const scrollToMessage = (messageId) => {
  emit('scroll-to-message', messageId);
};

const toggleReaction = (emoji) => {
  // 切换反应表情
  console.log('Toggle reaction:', emoji);
};

const addReaction = (emoji) => {
  showEmojiPicker.value = false;
  toggleReaction(emoji);
};

const retryMessage = () => {
  chatStore.retrySendMessage(props.message);
};

// 文件处理
const isImageFile = (file) => {
  if (!file) return false;
  
  // 如果file是字符串URL，检查扩展名
  if (typeof file === 'string') {
    const imageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'bmp'];
    const extension = file.split('.').pop()?.toLowerCase();
    return imageExtensions.includes(extension);
  }
  
  // 如果是对象，检查mime_type或filename
  const imageMimeTypes = ['image/jpeg', 'image/png', 'image/gif', 'image/webp', 'image/svg+xml'];
  const imageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'bmp'];
  
  return imageMimeTypes.includes(file.mime_type?.toLowerCase()) ||
         imageExtensions.includes(file.filename?.split('.').pop()?.toLowerCase());
};

const getFileUrl = (file) => {
  // 如果file已经是完整的URL字符串，直接返回
  if (typeof file === 'string') {
    // 如果是相对路径，添加正确的基础URL
    if (file.startsWith('/')) {
      // 文件URL格式: /files/{workspace_id}/{path}
      // 需要通过API访问: /api/files/{workspace_id}/{path}
      if (file.startsWith('/files/')) {
        const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080';
        // 将 /files/ 替换为 /api/files/
        return `${baseUrl}/api${file}`;
      }
      const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080';
      return `${baseUrl}${file}`;
    }
    return file;
  }
  
  // 如果是对象，尝试获取URL
  if (file.url) {
    // 递归处理URL
    return getFileUrl(file.url);
  }
  const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080';
  return `${baseUrl}/api/files/${file.id || file.filename}`;
};

const getFileName = (file) => {
  // 如果是字符串URL，从路径中提取文件名
  if (typeof file === 'string') {
    const parts = file.split('/');
    return parts[parts.length - 1] || 'Unnamed file';
  }
  // 如果是对象，返回filename属性
  return file.filename || 'Unnamed file';
};

const getFileSize = (file) => {
  // 如果是字符串URL，无法获取大小
  if (typeof file === 'string') {
    return '';
  }
  // 如果是对象且有size属性
  if (file.size) {
    return formatFileSize(file.size);
  }
  return '';
};

const formatFileSize = (bytes) => {
  if (!bytes) return 'Unknown size';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};

const truncateReply = (text) => {
  if (!text) return '';
  return text.length > 50 ? text.substring(0, 50) + '...' : text;
};

const openImageViewer = (index) => {
  // 打开图片查看器
  if (imageViewerRef.value) {
    imageViewerRef.value.open(index);
  }
};
</script>

<style scoped>
/* 🎨 Discord消息容器 */
.discord-message {
  /* 🔒 关键：固定最小高度确保视觉锚点稳定 */
  min-height: 44px;
  padding: 2px 16px 2px 72px;
  position: relative;
  transition: background-color 0.06s ease;
  /* 🔒 使用contain防止布局偏移 */
  contain: layout style;
  /* 🔒 确保消息间距一致 */
  margin-bottom: 0;
  border-bottom: 1px solid transparent;
}

.discord-message:hover {
  background-color: var(--bg-message-hover);
}

.discord-message.is-highlighted {
  background-color: rgba(88, 101, 242, 0.1);
  border-left: 4px solid var(--discord-primary);
  padding-left: 68px;
}

.discord-message.is-sending {
  opacity: 0.7;
}

.discord-message.is-failed {
  background-color: rgba(237, 66, 69, 0.1);
}

/* 🎯 三槽布局 */
.message-layout {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  /* 🔒 固定布局，防止内容变化导致偏移 */
  min-height: 40px;
}

/* 🎯 头像槽 */
.message-avatar-slot {
  position: absolute;
  left: 16px;
  top: 2px;
  /* 🔒 固定尺寸，防止头像大小变化 */
  width: 40px;
  height: 40px;
  flex-shrink: 0;
}

/* 🎯 内容槽 */
.message-content-slot {
  flex: 1;
  min-width: 0;
  /* 🔒 确保内容不会影响整体布局 */
  margin-left: 0;
}

/* 🎯 Meta信息 */
.message-meta {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 2px;
  /* 🔒 固定行高 */
  line-height: 1.375;
  min-height: 18px;
}

.sender-name {
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  color: var(--text-primary);
  transition: text-decoration 0.1s ease;
}

.sender-name:hover {
  text-decoration: underline;
}

.message-timestamp {
  font-size: 0.75rem;
  color: var(--text-muted);
  font-weight: 400;
  margin-left: 4px;
}

.message-timestamp:hover {
  color: var(--text-secondary);
}

.edited-indicator {
  font-size: 0.625rem;
  color: var(--text-muted);
  font-weight: 400;
}

/* 🎯 状态指示器 */
.message-status {
  display: flex;
  align-items: center;
  margin-left: 4px;
}

.status-sending .spinner {
  width: 12px;
  height: 12px;
  border: 2px solid var(--bg-tertiary);
  border-top: 2px solid var(--discord-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.status-failed {
  color: var(--text-danger);
}

.status-sent {
  color: var(--text-positive);
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* 🎯 消息主体 */
.message-body {
  /* 🔒 固定行高确保稳定性 */
  line-height: 1.375;
  /* 🔒 防止内容溢出影响布局 */
  word-wrap: break-word;
  overflow-wrap: break-word;
}

/* 🎯 回复引用 */
.message-reply {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
  padding: 4px 8px;
  background: var(--bg-secondary);
  border-radius: 3px;
  cursor: pointer;
  transition: background-color 0.1s ease;
  position: relative;
}

.message-reply:hover {
  background: var(--bg-accent);
}

.reply-line {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: var(--text-muted);
  border-radius: 1px;
}

.reply-content {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-left: 12px;
}

.reply-author {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-secondary);
}

.reply-text {
  font-size: 0.875rem;
  color: var(--text-muted);
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 🎯 消息文本 */
.message-text {
  font-size: 1rem;
  color: var(--text-primary);
  /* 🔒 固定行高确保布局稳定 */
  line-height: 1.375;
  margin: 0;
}

/* 🎯 附件样式 */
.message-attachments {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.image-attachment {
  position: relative;
  display: inline-block;
  cursor: pointer;
  border-radius: 8px;
  overflow: hidden;
  max-width: 400px;
  max-height: 300px;
  border: 1px solid var(--border-primary);
  transition: border-color 0.1s ease;
}

.image-attachment:hover {
  border-color: var(--border-focus);
}

.attachment-image {
  display: block;
  width: 100%;
  height: auto;
  max-height: 300px;
  object-fit: cover;
}

.image-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.1s ease;
  color: white;
}

.image-attachment:hover .image-overlay {
  opacity: 1;
}

.file-attachment {
  display: inline-flex;
  align-items: center;
  gap: 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 8px;
  padding: 12px 16px;
  text-decoration: none;
  color: inherit;
  transition: all 0.1s ease;
  max-width: 400px;
}

.file-attachment:hover {
  background: var(--bg-accent);
  border-color: var(--border-focus);
  text-decoration: none;
  color: inherit;
}

.file-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: var(--discord-primary);
  border-radius: 6px;
  color: white;
  flex-shrink: 0;
}

.file-details {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-weight: 500;
  color: var(--text-primary);
  font-size: 0.875rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-size {
  font-size: 0.75rem;
  color: var(--text-muted);
  margin-top: 2px;
}

.download-icon {
  flex-shrink: 0;
  color: var(--text-muted);
  transition: color 0.1s ease;
}

.file-attachment:hover .download-icon {
  color: var(--text-secondary);
}

/* 🎯 错误状态 */
.message-error {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
  padding: 6px 8px;
  background: rgba(237, 66, 69, 0.1);
  border-radius: 4px;
}

.error-text {
  font-size: 0.75rem;
  color: var(--text-danger);
}

.retry-button {
  display: flex;
  align-items: center;
  gap: 4px;
  background: var(--text-danger);
  color: white;
  border: none;
  border-radius: 3px;
  padding: 4px 8px;
  font-size: 0.75rem;
  cursor: pointer;
  transition: opacity 0.1s ease;
}

.retry-button:hover {
  opacity: 0.8;
}

/* 🎯 反应表情 */
.message-reactions {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 4px;
}

.reaction-item {
  display: flex;
  align-items: center;
  gap: 4px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 12px;
  padding: 2px 6px;
  cursor: pointer;
  transition: all 0.1s ease;
  font-size: 0.75rem;
}

.reaction-item:hover {
  background: var(--bg-accent);
}

.reaction-item.is-reacted {
  background: rgba(88, 101, 242, 0.15);
  border-color: var(--discord-primary);
  color: var(--discord-primary);
}

.reaction-emoji {
  font-size: 0.875rem;
}

.reaction-count {
  font-weight: 500;
  color: var(--text-secondary);
}

/* 🎯 工具栏 */
.message-toolbar {
  position: absolute;
  top: -12px;
  right: 16px;
  background: var(--bg-floating);
  border: 1px solid var(--border-primary);
  border-radius: 8px;
  padding: 4px;
  box-shadow: var(--shadow-medium);
  z-index: 10;
}

.toolbar-actions {
  display: flex;
  gap: 2px;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: var(--interactive-normal);
  cursor: pointer;
  transition: all 0.1s ease;
  /* 遵循触摸目标最小尺寸 */
  min-width: 44px;
  min-height: 44px;
  padding: 4px;
  touch-action: manipulation;
}

.toolbar-btn:hover {
  background: var(--bg-secondary);
  color: var(--interactive-hover);
}

/* 🎯 工具栏动画 */
.toolbar-enter-active,
.toolbar-leave-active {
  transition: all 0.1s ease;
}

.toolbar-enter-from,
.toolbar-leave-to {
  opacity: 0;
  transform: translateY(4px) scale(0.95);
}

/* 🎯 表情选择器 */
.emoji-picker-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.1);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.emoji-picker {
  background: var(--bg-floating);
  border: 1px solid var(--border-primary);
  border-radius: 8px;
  padding: 12px;
  box-shadow: var(--shadow-high);
  max-width: 200px;
}

.emoji-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 4px;
}

.emoji-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: transparent;
  border: none;
  border-radius: 6px;
  font-size: 1.125rem;
  cursor: pointer;
  transition: background-color 0.1s ease;
  /* 遵循触摸目标最小尺寸 */
  min-width: 44px;
  min-height: 44px;
  padding: 4px;
  touch-action: manipulation;
}

.emoji-btn:hover {
  background: var(--bg-secondary);
}

/* 🎯 紧凑模式 */
.discord-message.is-compact {
  padding: 1px 16px 1px 72px;
  min-height: 22px;
}

.discord-message.is-compact .message-meta {
  margin-bottom: 0;
}

.discord-message.is-compact .message-timestamp {
  position: absolute;
  left: 16px;
  width: 40px;
  text-align: center;
  opacity: 0;
  transition: opacity 0.1s ease;
}

.discord-message.is-compact:hover .message-timestamp {
  opacity: 1;
}

/* 🎯 响应式 */
@media (max-width: 768px) {
  .discord-message {
    padding: 8px 12px;
  }
  
  .message-avatar-slot {
    left: 12px;
  }
  
  .message-toolbar {
    right: 12px;
  }
  
  .image-attachment,
  .file-attachment {
    max-width: 100%;
  }
}

/* 🎯 无障碍支持 */
@media (prefers-reduced-motion: reduce) {
  .discord-message,
  .toolbar-btn,
  .reaction-item {
    transition: none;
  }
}
</style> 