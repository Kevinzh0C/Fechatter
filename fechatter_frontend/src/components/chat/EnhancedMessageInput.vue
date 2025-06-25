<template>
  <div class="bg-white/95 backdrop-blur-sm border-t border-gray-200/60" 
       style="background: rgba(253, 252, 250, 0.95);">
    <!-- 居中布局容器 -->
    <div class="max-w-3xl mx-auto">
      
      <!-- 输入状态指示器 -->
      <div v-if="typingDisplay" class="px-6 py-2 text-sm text-gray-600 border-b border-gray-100">
        {{ typingDisplay }}
      </div>

      <!-- 回复消息预览 -->
      <div v-if="replyToMessage" class="px-6 py-2 bg-gray-50 border-b border-gray-200">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6"></path>
            </svg>
            <span class="text-sm font-medium">Replying to {{ replyToMessage.sender.fullname }}</span>
          </div>
          <button @click="cancelReply" class="text-gray-400 hover:text-gray-600">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
            </svg>
          </button>
        </div>
        <div class="mt-1 text-sm text-gray-600 truncate">
          {{ replyToMessage.content }}
        </div>
      </div>

      <!-- 编辑消息模式 -->
      <div v-if="messageSystem.state.editing.messageId" class="px-6 py-2 bg-blue-50 border-b border-blue-200">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <svg class="w-4 h-4 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
            </svg>
            <span class="text-sm font-medium text-blue-700">Editing message</span>
          </div>
          <button @click="cancelEdit" class="text-blue-400 hover:text-blue-600">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
            </svg>
          </button>
        </div>
      </div>

      <!-- 主输入区域 -->
      <div class="relative px-6 py-4">
        <div class="flex items-end space-x-3">
          
          <!-- 文件上传按钮 -->
          <button 
            @click="triggerFileUpload"
            class="flex-shrink-0 p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
            :disabled="messageSystem.state.sending"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"></path>
            </svg>
          </button>

          <!-- 消息输入框 -->
          <div class="flex-1 relative">
            <textarea
              ref="messageInput"
              v-model="messageContent"
              @keydown="handleKeyDown"
              @input="handleInput"
              @focus="handleFocus"
              @blur="handleBlur"
              :placeholder="placeholderText"
              class="w-full resize-none rounded-lg border border-gray-300 px-4 py-3 pr-12 focus:border-violet-500 focus:ring-1 focus:ring-violet-500 transition-colors"
              :class="{
                'border-red-300': error,
                'bg-gray-50': messageSystem.state.sending
              }"
              :disabled="messageSystem.state.sending"
              :style="{ minHeight: '44px', maxHeight: '120px' }"
              rows="1"
            ></textarea>

            <!-- 表情符号按钮 -->
            <button 
              @click="toggleEmojiPicker"
              class="absolute right-3 bottom-3 text-gray-400 hover:text-gray-600 transition-colors"
              :disabled="messageSystem.state.sending"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              </svg>
            </button>
          </div>

          <!-- 发送按钮 -->
          <button 
            @click="handleSendMessage"
            :disabled="!canSend"
            class="flex-shrink-0 p-2 rounded-lg transition-all duration-200"
            :class="canSend 
              ? 'bg-violet-600 text-white hover:bg-violet-700 transform hover:scale-105' 
              : 'bg-gray-200 text-gray-400 cursor-not-allowed'"
          >
            <svg v-if="!messageSystem.state.sending" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"></path>
            </svg>
            <div v-else class="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
          </button>

        </div>

        <!-- 错误提示 -->
        <div v-if="error" class="mt-2 text-sm text-red-600">
          {{ error }}
        </div>

        <!-- Mentions 建议列表 -->
        <div v-if="showMentions" class="absolute bottom-full left-0 right-0 mb-2 bg-white rounded-lg shadow-lg border border-gray-200 max-h-48 overflow-y-auto z-10">
          <div v-for="user in mentionSuggestions" :key="user.id" 
               @click="selectMention(user)"
               class="px-4 py-2 hover:bg-gray-100 cursor-pointer flex items-center space-x-2">
            <div class="w-8 h-8 bg-violet-100 rounded-full flex items-center justify-center">
              <span class="text-sm font-medium text-violet-600">{{ user.fullname.charAt(0) }}</span>
            </div>
            <span class="text-sm">{{ user.fullname }}</span>
            <span class="text-xs text-gray-500">@{{ user.username }}</span>
          </div>
        </div>
      </div>

      <!-- 文件上传隐藏输入 -->
      <input
        ref="fileInput"
        type="file"
        multiple
        class="hidden"
        @change="handleFileSelect"
        accept="image/*,.pdf,.doc,.docx,.txt,.zip,.rar"
      />

    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import { useMessageSystem } from '@/composables/useMessageSystem';

// Props & Emits
const props = defineProps({
  chatId: {
    type: [Number, String],
    required: true
  },
  replyToMessage: {
    type: Object,
    default: null
  }
});

const emit = defineEmits(['reply-cancelled', 'message-sent']);

// ================================
// 消息系统集成
// ================================

const messageSystem = useMessageSystem(props.chatId);

// ================================
// 响应式状态
// ================================

const messageContent = ref('');
const error = ref('');
const isTyping = ref(false);
const showEmojiPicker = ref(false);
const showMentions = ref(false);
const mentionSuggestions = ref([]);
const mentionQuery = ref('');

// DOM 引用
const messageInput = ref(null);
const fileInput = ref(null);

// 编辑模式：监听消息系统的编辑状态
watch(() => messageSystem.state.editing.messageId, (messageId) => {
  if (messageId) {
    messageContent.value = messageSystem.state.editing.content;
    nextTick(() => {
      messageInput.value?.focus();
    });
  }
});

// ================================
// 计算属性
// ================================

const canSend = computed(() => {
  if (messageSystem.state.editing.messageId) {
    return messageContent.value.trim() !== messageSystem.state.editing.content;
  }
  return messageContent.value.trim().length > 0 && !messageSystem.state.sending;
});

const placeholderText = computed(() => {
  if (messageSystem.state.editing.messageId) {
    return 'Edit your message...';
  }
  if (props.replyToMessage) {
    return `Reply to ${props.replyToMessage.sender.fullname}...`;
  }
  return 'Type a message...';
});

const typingDisplay = computed(() => messageSystem.isTypingDisplayText);

// ================================
// 消息发送
// ================================

const handleSendMessage = async () => {
  if (!canSend.value) return;

  const content = messageContent.value.trim();
  if (!content) return;

  error.value = '';

  try {
    if (messageSystem.state.editing.messageId) {
      // 编辑模式
      messageSystem.state.editing.content = content;
      await messageSystem.saveEditMessage();
    } else {
      // 发送新消息
      const messageData = {
        content,
        message_type: 'text',
        ...(props.replyToMessage && { reply_to_id: props.replyToMessage.id })
      };

      const sentMessage = await messageSystem.sendTextMessage(content, {
        ...(props.replyToMessage && { reply_to: props.replyToMessage.id })
      });

      // 只有成功发送时才触发事件
      if (sentMessage) {
        emit('message-sent', sentMessage);
      }
    }

    // 清空输入
    messageContent.value = '';
    adjustTextareaHeight();
    
  } catch (err) {
    console.error('❌ [EnhancedMessageInput] Send message failed:', err);
    error.value = err.message || 'Failed to send message';
  }
};

// ================================
// 输入处理
// ================================

const handleInput = () => {
  adjustTextareaHeight();
  handleTypingIndicator();
  checkForMentions();
};

const handleKeyDown = (event) => {
  if (event.key === 'Enter') {
    if (event.shiftKey) {
      // Shift+Enter: 换行
      return;
    } else {
      // Enter: 发送消息
      event.preventDefault();
      handleSendMessage();
    }
  } else if (event.key === 'Escape') {
    if (messageSystem.state.editing.messageId) {
      cancelEdit();
    } else if (props.replyToMessage) {
      cancelReply();
    }
  }
};

const handleFocus = () => {
  // 开始输入时发送打字状态
  if (!isTyping.value) {
    isTyping.value = true;
    messageSystem.startTyping();
  }
};

const handleBlur = () => {
  // 失去焦点时停止打字状态
  if (isTyping.value) {
    isTyping.value = false;
    messageSystem.stopTyping();
  }
};

// ================================
// UI 辅助功能
// ================================

const adjustTextareaHeight = () => {
  nextTick(() => {
    const textarea = messageInput.value;
    if (textarea && textarea.style) {
      try {
        textarea.style.height = 'auto';
        const newHeight = Math.min(textarea.scrollHeight || 44, 120);
        textarea.style.height = newHeight + 'px';
      } catch (error) {
        console.warn('⚠️ [EnhancedMessageInput] Failed to adjust textarea height:', error);
      }
    }
  });
};

// ================================
// 输入状态管理
// ================================

let typingTimer = null;

const handleTypingIndicator = () => {
  if (!isTyping.value) {
    isTyping.value = true;
    messageSystem.startTyping();
  }

  // 重置计时器
  if (typingTimer) clearTimeout(typingTimer);
  typingTimer = setTimeout(() => {
    isTyping.value = false;
    messageSystem.stopTyping();
  }, 2000);
};

// ================================
// 编辑和回复功能
// ================================

const cancelEdit = () => {
  messageSystem.cancelEditMessage();
  messageContent.value = '';
};

const cancelReply = () => {
  emit('reply-cancelled');
};

// ================================
// 文件上传
// ================================

const triggerFileUpload = () => {
  try {
    if (fileInput.value && typeof fileInput.value.click === 'function') {
      fileInput.value.click();
    } else {
      console.warn('⚠️ [EnhancedMessageInput] File input not available');
    }
  } catch (error) {
    console.warn('⚠️ [EnhancedMessageInput] Failed to trigger file upload:', error);
  }
};

const handleFileSelect = (event) => {
  const files = Array.from(event.target.files);
  if (files.length > 0) {
    // TODO: 实现文件上传功能
    console.log('Selected files:', files);
  }
};

// ================================
// @ 提及功能
// ================================

const checkForMentions = () => {
  const content = messageContent.value;
  const cursorPosition = messageInput.value?.selectionStart || 0;
  
  // 查找最近的 @
  const textBeforeCursor = content.slice(0, cursorPosition);
  const mentionMatch = textBeforeCursor.match(/@(\w*)$/);
  
  if (mentionMatch) {
    mentionQuery.value = mentionMatch[1];
    showMentions.value = true;
    // TODO: 获取用户建议
    fetchMentionSuggestions(mentionQuery.value);
  } else {
    showMentions.value = false;
  }
};

const fetchMentionSuggestions = async (query) => {
  // TODO: 实现用户搜索 API
  mentionSuggestions.value = [];
};

const selectMention = (user) => {
  // TODO: 实现提及插入逻辑
  showMentions.value = false;
};

// ================================
// 表情符号
// ================================

const toggleEmojiPicker = () => {
  showEmojiPicker.value = !showEmojiPicker.value;
};

// ================================
// 生命周期
// ================================

onMounted(() => {
  // 定期更新正在输入的用户
  const interval = setInterval(() => {
    messageSystem.updateTypingUsers();
  }, 3000);

  onUnmounted(() => {
    clearInterval(interval);
    if (typingTimer) clearTimeout(typingTimer);
    if (isTyping.value) {
      messageSystem.stopTyping();
    }
  });
});

// ================================
// 暴露给父组件的方法
// ================================

defineExpose({
  focus: () => messageInput.value?.focus(),
  clear: () => { messageContent.value = ''; },
  startEdit: (messageId, content) => {
    messageSystem.startEditMessage(messageId, content);
  }
});
</script>

<style scoped>
/* 自定义滚动条样式 */
textarea::-webkit-scrollbar {
  width: 4px;
}

textarea::-webkit-scrollbar-track {
  background: transparent;
}

textarea::-webkit-scrollbar-thumb {
  background: #cbd5e0;
  border-radius: 2px;
}

textarea::-webkit-scrollbar-thumb:hover {
  background: #a0aec0;
}
</style>