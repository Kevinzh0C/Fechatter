<template>
  <div :data-message-id="message.id"
    class="group relative hover:bg-gray-50/50 transition-colors duration-150 py-1 px-4 rounded-lg" :class="{
      'bg-blue-50/30': isEditing,
      'bg-yellow-50/50': searchQuery && isHighlighted
    }">
    <div class="flex space-x-3">

      <!-- 用户头像 -->
      <div class="flex-shrink-0">
        <div v-if="showAvatar" class="w-10 h-10 bg-violet-100 rounded-full flex items-center justify-center">
          <span class="text-sm font-medium text-violet-600">
            {{ message.sender.fullname.charAt(0).toUpperCase() }}
          </span>
        </div>
        <div v-else class="w-10 h-10 flex items-center justify-center">
          <span class="text-xs text-gray-400 opacity-0 group-hover:opacity-100 transition-opacity">
            {{ formatTime(message.created_at) }}
          </span>
        </div>
      </div>

      <!-- 消息内容区域 -->
      <div class="flex-1 min-w-0">

        <!-- 消息头部（用户名和时间戳） -->
        <div v-if="showAvatar || showTimestamp" class="flex items-baseline space-x-2 mb-1">
          <span class="font-semibold text-gray-900">{{ message.sender.fullname }}</span>
          <span class="text-xs text-gray-500">{{ formatDateTime(message.created_at) }}</span>
          <span v-if="message.is_edited" class="text-xs text-gray-400">(edited)</span>
        </div>

        <!-- 回复预览 -->
        <div v-if="replyToMessage" class="mb-2 pl-3 border-l-2 border-gray-300 bg-gray-50 rounded p-2 text-sm">
          <div class="text-gray-600 mb-1">
            <span class="font-medium">{{ replyToMessage.sender.fullname }}</span>
          </div>
          <div class="text-gray-800 truncate">{{ replyToMessage.content }}</div>
        </div>

        <!-- 消息内容 -->
        <div class="text-gray-900">
          <div v-if="!isEditing" class="message-content" v-html="formattedContent"></div>

          <!-- 编辑模式 -->
          <div v-else class="space-y-2">
            <textarea ref="editInput" v-model="editContent"
              class="w-full p-2 border border-gray-300 rounded resize-none focus:ring-2 focus:ring-violet-500 focus:border-violet-500"
              rows="3" @keydown="handleEditKeydown"></textarea>
            <div class="flex space-x-2">
              <button @click="saveEdit"
                class="px-3 py-1 bg-violet-600 text-white text-sm rounded hover:bg-violet-700 transition-colors"
                :disabled="!editContent.trim() || editContent === message.content">
                Save
              </button>
              <button @click="cancelEdit"
                class="px-3 py-1 bg-gray-200 text-gray-700 text-sm rounded hover:bg-gray-300 transition-colors">
                Cancel
              </button>
            </div>
          </div>
        </div>

        <!-- 文件附件 -->
        <div v-if="message.files && message.files.length > 0" class="mt-2 space-y-2">
          <div v-for="file in message.files" :key="file.id"
            class="flex items-center space-x-2 p-2 bg-gray-100 rounded border">
            <div class="flex-shrink-0">
              <svg class="w-5 h-5 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z">
                </path>
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium text-gray-900 truncate">{{ file.filename }}</div>
              <div class="text-xs text-gray-500">{{ formatFileSize(file.size) }}</div>
            </div>
            <button @click="downloadFile(file)" class="flex-shrink-0 text-violet-600 hover:text-violet-700 text-sm">
              Download
            </button>
          </div>
        </div>

        <!-- 消息反应 -->
        <div v-if="message.reactions && message.reactions.length > 0" class="mt-2 flex flex-wrap gap-1">
          <button v-for="reaction in message.reactions" :key="reaction.emoji" @click="toggleReaction(reaction.emoji)"
            class="inline-flex items-center space-x-1 px-2 py-1 bg-gray-100 hover:bg-gray-200 rounded-full text-sm transition-colors"
            :class="{ 'bg-violet-100 text-violet-700': hasUserReacted(reaction) }">
            <span>{{ reaction.emoji }}</span>
            <span class="text-xs">{{ reaction.count }}</span>
          </button>
        </div>

      </div>

      <!-- 消息操作按钮 -->
      <div class="flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
        <div class="flex space-x-1">

          <!-- 反应按钮 -->
          <button @click="showReactionPicker = !showReactionPicker"
            class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded transition-colors"
            title="Add reaction">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
            </svg>
          </button>

          <!-- 回复按钮 -->
          <button @click="$emit('reply', message)"
            class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded transition-colors" title="Reply">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6"></path>
            </svg>
          </button>

          <!-- 更多操作按钮 -->
          <div class="relative">
            <button @click="showMoreMenu = !showMoreMenu"
              class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded transition-colors"
              title="More actions">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z">
                </path>
              </svg>
            </button>

            <!-- 更多操作菜单 -->
            <div v-if="showMoreMenu"
              class="absolute right-0 top-full mt-1 bg-white rounded-lg shadow-lg border border-gray-200 py-1 z-10 min-w-[120px]">
              <button v-if="canEdit" @click="startEdit"
                class="w-full px-3 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 transition-colors">
                Edit message
              </button>
              <button @click="copyMessage"
                class="w-full px-3 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 transition-colors">
                Copy text
              </button>
              <button @click="copyLink"
                class="w-full px-3 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 transition-colors">
                Copy link
              </button>
              <div class="border-t border-gray-100 my-1"></div>
              <button v-if="canDelete" @click="deleteMessage"
                class="w-full px-3 py-2 text-left text-sm text-red-600 hover:bg-red-50 transition-colors">
                Delete message
              </button>
            </div>
          </div>

        </div>
      </div>

    </div>

    <!-- 反应选择器 -->
    <div v-if="showReactionPicker"
      class="absolute top-full left-12 mt-1 bg-white rounded-lg shadow-lg border border-gray-200 p-2 z-20">
      <div class="grid grid-cols-8 gap-1">
        <button v-for="emoji in commonEmojis" :key="emoji" @click="addReaction(emoji)"
          class="p-2 hover:bg-gray-100 rounded text-lg transition-colors">
          {{ emoji }}
        </button>
      </div>
    </div>

  </div>
</template>

<script setup>
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue';
import { useAuthStore } from '@/stores/auth';

// Props
const props = defineProps({
  message: {
    type: Object,
    required: true
  },
  showAvatar: {
    type: Boolean,
    default: true
  },
  showTimestamp: {
    type: Boolean,
    default: true
  },
  isEditing: {
    type: Boolean,
    default: false
  },
  searchQuery: {
    type: String,
    default: ''
  }
});

// Emits
const emit = defineEmits(['edit', 'delete', 'reply', 'react', 'mention-click']);

// ================================
// 响应式状态
// ================================

const editContent = ref(props.message.content);
const showMoreMenu = ref(false);
const showReactionPicker = ref(false);
const editInput = ref(null);

// 认证状态
const authStore = useAuthStore();

// ================================
// 计算属性
// ================================

const canEdit = computed(() => {
  return authStore.user?.id === props.message.sender_id;
});

const canDelete = computed(() => {
  return authStore.user?.id === props.message.sender_id;
});

const isHighlighted = computed(() => {
  return props.searchQuery && props.message.content.toLowerCase().includes(props.searchQuery.toLowerCase());
});

const replyToMessage = computed(() => {
  // 基于数据库结构，使用 reply_to 字段和关联数据
  return props.message.reply_to_message || null;
});

const formattedContent = computed(() => {
  let content = props.message.content;

  // 高亮搜索关键词
  if (props.searchQuery && props.searchQuery.trim()) {
    const regex = new RegExp(`(${escapeRegex(props.searchQuery)})`, 'gi');
    content = content.replace(regex, '<mark class="bg-yellow-200">$1</mark>');
  }

  // 处理 @ 提及
  content = content.replace(/@(\w+)/g, '<span class="text-violet-600 font-medium cursor-pointer" data-mention="$1">@$1</span>');

  // 处理链接
  const urlRegex = /(https?:\/\/[^\s]+)/g;
  content = content.replace(urlRegex, '<a href="$1" target="_blank" class="text-violet-600 underline hover:text-violet-700">$1</a>');

  // 处理换行
  content = content.replace(/\n/g, '<br>');

  return content;
});

// ================================
// 常用表情符号
// ================================

const commonEmojis = ['👍', '👎', '❤️', '😂', '😮', '😢', '😡', '🎉'];

// ================================
// 格式化函数
// ================================

const formatTime = (dateString) => {
  return new Date(dateString).toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit'
  });
};

const formatDateTime = (dateString) => {
  const date = new Date(dateString);
  const now = new Date();
  const diffInHours = (now - date) / (1000 * 60 * 60);

  if (diffInHours < 24) {
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit'
    });
  } else {
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }
};

const formatFileSize = (bytes) => {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const escapeRegex = (string) => {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
};

// ================================
// 编辑功能
// ================================

const startEdit = () => {
  emit('edit', props.message);
  showMoreMenu.value = false;

  nextTick(() => {
    if (editInput.value) {
      editInput.value.focus();
      editInput.value.setSelectionRange(0, editInput.value.value.length);
    }
  });
};

const saveEdit = () => {
  if (editContent.value.trim() && editContent.value !== props.message.content) {
    // 通过消息系统保存编辑
    // 这里应该通过父组件或事件总线来处理
    emit('edit', { ...props.message, content: editContent.value.trim() });
  }
};

const cancelEdit = () => {
  editContent.value = props.message.content;
  emit('edit', null); // 取消编辑
};

const handleEditKeydown = (event) => {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault();
    saveEdit();
  } else if (event.key === 'Escape') {
    cancelEdit();
  }
};

// ================================
// 操作功能
// ================================

const deleteMessage = () => {
  showMoreMenu.value = false;
  emit('delete', props.message);
};

const copyMessage = async () => {
  try {
    await navigator.clipboard.writeText(props.message.content);
    showMoreMenu.value = false;
    // TODO: 显示成功提示
  } catch (error) {
    console.error('Failed to copy message:', error);
  }
};

const copyLink = async () => {
  try {
    const url = `${window.location.origin}/chat/${props.message.chat_id}?message=${props.message.id}`;
    await navigator.clipboard.writeText(url);
    showMoreMenu.value = false;
    // TODO: 显示成功提示
  } catch (error) {
    console.error('Failed to copy link:', error);
  }
};

const downloadFile = (file) => {
  // TODO: 实现文件下载逻辑
  console.log('Download file:', file);
};

// ================================
// 反应功能
// ================================

const addReaction = (emoji) => {
  emit('react', props.message, emoji);
  showReactionPicker.value = false;
};

const toggleReaction = (emoji) => {
  emit('react', props.message, emoji);
};

const hasUserReacted = (reaction) => {
  return reaction.users && reaction.users.includes(authStore.user?.id);
};

// ================================
// 点击处理
// ================================

const handleContentClick = (event) => {
  const target = event.target;

  // 处理 @ 提及点击
  if (target.hasAttribute('data-mention')) {
    const username = target.getAttribute('data-mention');
    emit('mention-click', username);
  }
};

// ================================
// 生命周期
// ================================

onMounted(() => {
  // 监听内容点击
  const contentEl = document.querySelector('.message-content');
  if (contentEl) {
    contentEl.addEventListener('click', handleContentClick);
  }
});

onUnmounted(() => {
  const contentEl = document.querySelector('.message-content');
  if (contentEl) {
    contentEl.removeEventListener('click', handleContentClick);
  }
});

// 监听外部点击，关闭菜单
onMounted(() => {
  const handleClickOutside = (event) => {
    if (!event.target.closest('.relative')) {
      showMoreMenu.value = false;
      showReactionPicker.value = false;
    }
  };

  document.addEventListener('click', handleClickOutside);

  onUnmounted(() => {
    document.removeEventListener('click', handleClickOutside);
  });
});
</script>

<style scoped>
/* 自定义标记样式 */
:deep(mark) {
  background-color: #fef3c7;
  padding: 0 2px;
  border-radius: 2px;
}

/* 消息内容链接样式 */
:deep(.message-content a) {
  color: #7c3aed;
  text-decoration: underline;
}

:deep(.message-content a:hover) {
  color: #5b21b6;
}

/* @ 提及样式 */
:deep(.message-content [data-mention]) {
  color: #7c3aed;
  font-weight: 500;
  cursor: pointer;
  border-radius: 3px;
  padding: 0 2px;
  transition: background-color 0.2s;
}

:deep(.message-content [data-mention]:hover) {
  background-color: #ede9fe;
}
</style>