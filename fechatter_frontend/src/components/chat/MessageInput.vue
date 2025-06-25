<template>
  <div class="bg-gradient-to-t from-gray-50/90 to-white/90 backdrop-blur-sm" 
       style="background: linear-gradient(to top, rgba(248, 250, 252, 0.9), rgba(255, 255, 255, 0.9));">
    <!-- 🎯 与MessageList一致的居中布局容器 -->
    <div class="max-w-3xl mx-auto">
      <!-- Global Drag Upload Overlay - shown when dragging over the entire component -->
      <div v-if="isDragging && !selectedFiles.length" 
           class="absolute inset-0 bg-violet-50/90 backdrop-blur-sm border-2 border-dashed border-violet-300 rounded-lg flex items-center justify-center z-50"
           @drop="handleDrop"
           @dragover.prevent
           @dragleave="handleDragLeave">
        <div class="text-center">
          <div class="w-16 h-16 bg-violet-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg class="w-8 h-8 text-violet-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"></path>
            </svg>
          </div>
          <h3 class="text-lg font-semibold text-violet-700 mb-2">Drop files to upload</h3>
          <p class="text-violet-600">Images, documents, videos, and more</p>
          <div class="mt-4 flex flex-wrap gap-2 justify-center">
            <span class="px-2 py-1 bg-violet-100 text-violet-700 rounded-full text-xs">Images</span>
            <span class="px-2 py-1 bg-violet-100 text-violet-700 rounded-full text-xs">Documents</span>
            <span class="px-2 py-1 bg-violet-100 text-violet-700 rounded-full text-xs">Videos</span>
          </div>
        </div>
      </div>

      <!-- File Upload Progress -->
      <div v-if="uploading" class="px-6 py-3 bg-violet-50/60 backdrop-blur-sm rounded-2xl mx-6 mb-2">
        <div class="flex items-center space-x-3">
          <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-violet-600"></div>
          <span class="text-sm text-violet-700 font-medium">Uploading files...</span>
          <div class="flex-1 bg-violet-200/60 rounded-full h-2">
            <div class="bg-gradient-to-r from-violet-500 to-violet-600 h-2 rounded-full transition-all duration-300" 
                 :style="{ width: uploadProgress + '%' }"></div>
          </div>
          <span class="text-xs text-violet-600 font-semibold">{{ uploadProgress }}%</span>
        </div>
      </div>

      <!-- Enhanced File Preview -->
      <EnhancedFilePreview
        :files="selectedFiles"
        :show-drop-zone="isDragging"
        :max-files="10"
        :max-file-size="maxFileSize"
        @file-removed="removeFile"
        @files-cleared="clearFiles"
        @files-dropped="handleFilesDropped"
      />

      <!-- Enhanced Code Block Preview -->
      <div v-if="hasCodeBlocks" class="px-6 pt-4 pb-0">
        <div class="max-w-2xl mx-auto">
          <div class="bg-gradient-to-br from-slate-50/80 to-slate-100/60 backdrop-blur-sm rounded-2xl shadow-sm">
            <!-- Preview Header -->
            <div class="flex items-center justify-between px-5 py-3 bg-gradient-to-r from-slate-100/80 to-slate-50/60 rounded-t-2xl">
              <div class="flex items-center space-x-3">
                <div class="w-8 h-8 bg-slate-200/60 rounded-full flex items-center justify-center">
                  <svg class="w-4 h-4 text-slate-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"></path>
                  </svg>
                </div>
                <div>
                  <span class="text-sm font-semibold text-slate-700">Code Preview</span>
                  <span class="text-xs text-slate-500 bg-slate-200/60 px-3 py-1 rounded-full ml-2">{{ detectedLanguages.join(', ') || 'text' }}</span>
                </div>
              </div>
              <button 
                @click="togglePreview" 
                class="flex items-center space-x-1 text-xs text-slate-600 hover:text-slate-800 transition-all duration-200 px-3 py-2 rounded-full hover:bg-slate-200/60"
              >
                <svg class="w-3 h-3 transition-transform duration-200" :class="{ 'rotate-180': !showPreview }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                </svg>
                <span>{{ showPreview ? 'Collapse' : 'Expand' }}</span>
              </button>
            </div>
            
            <!-- Preview Content -->
            <div v-if="showPreview" class="enhanced-code-preview">
              <div v-html="enhancedCodePreview" class="max-h-64 overflow-y-auto px-1 pb-1"></div>
            </div>
          </div>
        </div>
      </div>

      <!-- Main Input Area - 优化的简洁布局 -->
      <div class="px-6 py-4" :class="{ 'pt-2': hasCodeBlocks && showPreview }">
        <div class="max-w-4xl mx-auto">
          <div class="flex items-center gap-3">
            <!-- Message Input - 主要焦点区域 -->
            <div class="flex-1 relative">
              <div class="flex items-center bg-white border border-gray-300 rounded-xl focus-within:ring-2 focus-within:ring-violet-500/40 focus-within:border-violet-500 transition-all">
                <!-- File Upload Button - 内置到输入框 -->
                <div class="flex-shrink-0 pl-4">
                  <input ref="fileInput" 
                         type="file" 
                         multiple 
                         class="hidden" 
                         @change="handleFileSelect"
                         accept="image/*,application/pdf,.doc,.docx,.txt,.csv,.xlsx,.ppt,.pptx">
                  <button @click="$refs.fileInput.click()" 
                          class="w-9 h-9 rounded-xl flex items-center justify-center transition-all duration-200 group"
                          :disabled="uploading"
                          aria-label="Attach files">
                    <svg class="w-5 h-5 text-gray-400 group-hover:text-violet-500 transition-colors duration-200" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"></path>
                    </svg>
                  </button>
                </div>

                <!-- Text Input -->
                <textarea
                  ref="messageInput"
                  v-model="message"
                  @keydown="handleKeyDown"
                  @input="adjustHeight"
                  @dragover.prevent="handleDragOver"
                  @drop.prevent="handleDrop"
                  placeholder="输入消息..."
                  class="flex-1 px-4 py-4 bg-transparent resize-none focus:outline-none placeholder-gray-400 text-gray-900"
                  :class="{ 
                    'pr-20': message.trim() || showEmojiPicker,
                    'min-h-16': hasCodeBlocks && showPreview
                  }"
                  rows="1"
                  :disabled="uploading"
                  :style="{
                    minHeight: hasCodeBlocks && showPreview ? '64px' : '48px',
                    maxHeight: hasCodeBlocks && showPreview ? '160px' : '120px',
                    fontSize: '15px',
                    lineHeight: '1.5'
                  }"
                  aria-label="Type your message"
                  aria-describedby="message-input-help"
                  role="textbox"
                  aria-multiline="true"
                ></textarea>

                <!-- Right Side Controls -->
                <div class="flex items-center gap-2 pr-4">
                  <!-- Character Count -->
                  <div v-if="message.length > 1500" 
                       class="text-xs px-3 py-1.5 rounded-full font-medium"
                       :class="message.length > 1800 ? 'text-red-600 bg-red-50' : 'text-gray-600 bg-gray-100'">
                    {{ message.length }}/2000
                  </div>
                  
                  <!-- Emoji Picker Button -->
                  <button @click="showEmojiPicker = !showEmojiPicker" 
                          class="w-9 h-9 rounded-xl flex items-center justify-center transition-all duration-200 group"
                          :disabled="uploading"
                          aria-label="Insert emoji"
                          :aria-expanded="showEmojiPicker">
                    <svg class="w-5 h-5 text-gray-400 group-hover:text-yellow-500 transition-colors duration-200" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                    </svg>
                  </button>
                </div>
              </div>
              
              <!-- Code Block Hint -->
              <div v-if="message.length === 0" 
                   class="absolute bottom-2 right-16 text-xs text-gray-400 pointer-events-none">
                <kbd class="px-2 py-1 bg-gray-100 rounded-lg text-gray-500 font-medium">{{ isMac ? '⌘' : 'Ctrl+' }}E</kbd> 
                <span class="ml-1">代码块</span>
              </div>
            </div>

            <!-- Send Button - 突出显示 -->
            <div class="flex-shrink-0">
              <button @click="sendMessage" 
                      :disabled="!canSend || uploading || isSending"
                      class="w-12 h-12 rounded-2xl flex items-center justify-center transition-all duration-300"
                      :class="[
                        canSend && !isSending 
                          ? 'bg-gradient-to-r from-violet-600 to-violet-700 hover:from-violet-700 hover:to-violet-800 hover:scale-105 text-white shadow-lg hover:shadow-xl cursor-pointer' 
                          : 'bg-gray-100 text-gray-400 cursor-not-allowed',
                        isSending ? 'bg-gradient-to-r from-violet-800 to-violet-900 scale-95 text-white cursor-wait' : ''
                      ]"
                      :aria-label="isSending ? 'Sending message...' : 'Send message'">
                <!-- 发送状态 -->
                <div v-if="isSending" class="w-5 h-5">
                  <div class="animate-spin rounded-full h-5 w-5 border-2 border-white border-t-transparent"></div>
                </div>
                <svg v-else class="w-6 h-6" :class="canSend && !isSending ? 'text-white drop-shadow-sm' : 'text-gray-400'" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"></path>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Emoji Picker - 重新定位 -->
      <div v-if="showEmojiPicker" class="px-6 pb-6">
        <div class="max-w-4xl mx-auto">
          <div class="relative">
            <div class="absolute bottom-0 right-16 bg-gradient-to-br from-white/95 to-gray-50/95 backdrop-blur-xl rounded-3xl shadow-2xl p-6 z-20 w-80"
                 style="backdrop-filter: blur(20px);">
              
              <!-- Emoji Categories -->
              <div class="flex space-x-1 mb-4 pb-3 border-b border-gray-200/60">
                <button v-for="category in emojiCategories" :key="category.name"
                        @click="activeEmojiCategory = category.name"
                        class="px-3 py-2 text-sm rounded-2xl transition-all duration-200 font-medium"
                        :class="activeEmojiCategory === category.name 
                          ? 'bg-violet-100 text-violet-700 shadow-sm' 
                          : 'text-gray-600 hover:bg-gray-100 hover:text-gray-800'">
                  {{ category.icon }} {{ category.name }}
                </button>
              </div>
              
              <!-- Emoji Grid -->
              <div class="grid grid-cols-8 gap-1 max-h-48 overflow-y-auto">
                <button v-for="emoji in getCurrentEmojis()" :key="emoji.char"
                        @click="insertEmoji(emoji.char)"
                        class="w-8 h-8 flex items-center justify-center rounded-xl hover:bg-gray-100 transition-all duration-200 text-lg hover:scale-110"
                        :title="emoji.name">
                  {{ emoji.char }}
                </button>
              </div>
              
              <!-- Quick Gestures Bar -->
              <div class="mt-4 pt-4 border-t border-gray-200/60">
                <div class="text-xs text-gray-500 mb-3 font-medium">Quick Actions</div>
                <div class="flex space-x-2 flex-wrap gap-1">
                  <button v-for="gesture in quickGestures" :key="gesture.char"
                          @click="insertEmoji(gesture.char)"
                          class="px-3 py-2 bg-gray-100/60 hover:bg-gray-200/80 rounded-xl text-sm transition-all duration-200 font-medium hover:scale-105"
                          :title="gesture.name">
                    {{ gesture.char }} {{ gesture.name }}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      
      <!-- Hidden help text for screen readers -->
      <div id="message-input-help" class="sr-only">
        Press {{ isMac ? 'Cmd' : 'Ctrl' }}+Enter to send message, Enter for new line. Use {{ isMac ? 'Cmd' : 'Ctrl' }}+E for code block formatting.
      </div>
    </div>
    
    <!-- Language Selector for Code Blocks -->
    <LanguageSelector
      :visible="showLanguageSelector"
      :position="languageSelectorPosition"
      :languages="languages"
      :suggested-languages="getSuggestedLanguages()"
      @select="selectLanguage"
      @close="cancelSelection"
    />
  </div>
</template>

<script setup>
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue';
import { useChatStore } from '@/stores/chat';
import api from '@/services/api';
import { useCodeBlockWrapper } from '@/composables/useCodeBlockWrapper';
import LanguageSelector from '@/components/ui/LanguageSelector.vue';
import EnhancedFilePreview from './EnhancedFilePreview.vue';

const emit = defineEmits(['send']);
const props = defineProps({
  chatId: {
    type: Number,
    required: true
  },
  chatType: {
    type: String,
    default: null
  }
});

// 🔧 通知功能（简化版，避免循环依赖）
const showNotification = (message, type = 'error') => {
  console.log(`[${type.toUpperCase()}] ${message}`);
  // 这里可以后续集成具体的通知组件
};

// State Management
const message = ref('');
const selectedFiles = ref([]);
const uploading = ref(false);
const uploadProgress = ref(0);
const error = ref('');
const showEmojiPicker = ref(false);
const activeEmojiCategory = ref('Smileys');
const isDragging = ref(false);

// 🔧 新增：发送状态管理
const isSending = ref(false);
const lastSendTime = ref(0);

// Code block preview
const showPreview = ref(true);

// References
const messageInput = ref(null);
const fileInput = ref(null);

// Chat store
const chatStore = useChatStore();

// Code block wrapper
const {
  showLanguageSelector,
  languageSelectorPosition,
  languages,
  handleKeyboardShortcut: handleCodeBlockShortcut,
  selectLanguage,
  cancelSelection,
  searchLanguages,
  getSuggestedLanguages,
  detectCodePattern
} = useCodeBlockWrapper(messageInput);

// Typing indicator state
let typingTimer = null;
let isTyping = false;

// Emoji Data - Enterprise Optimization
const emojiCategories = ref([
  { 
    name: 'Smileys', 
    icon: '😊',
    emojis: [
      { char: '😊', name: 'Smiling Face' },
      { char: '😂', name: 'Joy' },
      { char: '🤣', name: 'Rolling on Floor' },
      { char: '😍', name: 'Heart Eyes' },
      { char: '🥰', name: 'Smiling Face with Hearts' },
      { char: '😘', name: 'Kiss' },
      { char: '😉', name: 'Wink' },
      { char: '😎', name: 'Cool' },
      { char: '🤔', name: 'Thinking' },
      { char: '😅', name: 'Sweat Smile' },
      { char: '😇', name: 'Angel' },
      { char: '🙂', name: 'Slight Smile' },
      { char: '🙃', name: 'Upside Down' },
      { char: '😋', name: 'Yum' },
      { char: '😛', name: 'Tongue Out' },
      { char: '🤪', name: 'Zany Face' }
    ]
  },
  { 
    name: 'Gestures', 
    icon: '👍',
    emojis: [
      { char: '👍', name: 'Thumbs Up' },
      { char: '👎', name: 'Thumbs Down' },
      { char: '👏', name: 'Clap' },
      { char: '🙌', name: 'Raised Hands' },
      { char: '👌', name: 'OK Hand' },
      { char: '✌️', name: 'Victory' },
      { char: '🤞', name: 'Crossed Fingers' },
      { char: '🤝', name: 'Handshake' },
      { char: '👋', name: 'Wave' },
      { char: '🤚', name: 'Raised Back' },
      { char: '🖐️', name: 'Hand with Fingers' },
      { char: '✋', name: 'Raised Hand' },
      { char: '🖖', name: 'Vulcan Salute' },
      { char: '👊', name: 'Fist Bump' },
      { char: '✊', name: 'Raised Fist' },
      { char: '🤛', name: 'Left Fist' }
    ]
  },
  { 
    name: 'Objects', 
    icon: '🎯',
    emojis: [
      { char: '🎯', name: 'Target' },
      { char: '🚀', name: 'Rocket' },
      { char: '💡', name: 'Light Bulb' },
      { char: '⚡', name: 'Lightning' },
      { char: '🔥', name: 'Fire' },
      { char: '💎', name: 'Diamond' },
      { char: '🏆', name: 'Trophy' },
      { char: '🎉', name: 'Party' },
      { char: '🎊', name: 'Confetti' },
      { char: '📈', name: 'Chart Up' },
      { char: '📊', name: 'Bar Chart' },
      { char: '💰', name: 'Money Bag' },
      { char: '⭐', name: 'Star' },
      { char: '✨', name: 'Sparkles' },
      { char: '🌟', name: 'Glowing Star' },
      { char: '💫', name: 'Dizzy' }
    ]
  }
]);

const quickGestures = ref([
  { char: '👍', name: 'Like' },
  { char: '❤️', name: 'Love' },
  { char: '🔥', name: 'Fire' },
  { char: '💯', name: '100' },
  { char: '✅', name: 'Done' }
]);

// Computed Properties
const currentEmojis = computed(() => {
  const category = emojiCategories.value.find(cat => cat.name === activeEmojiCategory.value);
  return category ? category.emojis : [];
});

const canSend = computed(() => {
  return (message.value.trim() || selectedFiles.value.length > 0) && !uploading.value && !isSending.value;
});

// Code block detection
const hasCodeBlocks = computed(() => {
  const codeBlockRegex = /```[\s\S]*?```/g;
  return codeBlockRegex.test(message.value);
});

// Detect languages in code blocks
const detectedLanguages = computed(() => {
  if (!hasCodeBlocks.value) return [];
  
  const languages = [];
  const regex = /```(\w+)?\n([\s\S]*?)```/g;
  let match;
  
  while ((match = regex.exec(message.value)) !== null) {
    const lang = match[1] || 'text';
    if (!languages.includes(lang)) {
      languages.push(lang);
    }
  }
  
  return languages;
});

// Basic syntax highlighting function
function applySyntaxHighlighting(code, language) {
  let highlightedCode = escapeHtml(code);
  
  try {
    switch (language.toLowerCase()) {
      case 'javascript':
      case 'js':
      case 'typescript':
      case 'ts':
        // Keywords
        highlightedCode = highlightedCode.replace(
          /\b(const|let|var|function|class|if|else|for|while|return|import|export|default|async|await)\b/g,
          '<span class="token keyword">$1</span>'
        );
        // Strings (simplified)
        highlightedCode = highlightedCode.replace(
          /(["'`])([^"'`]*?)\1/g,
          '<span class="token string">$1$2$1</span>'
        );
        // Numbers
        highlightedCode = highlightedCode.replace(
          /\b\d+\.?\d*\b/g,
          '<span class="token number">$&</span>'
        );
        // Comments
        highlightedCode = highlightedCode.replace(
          /(\/\/.*$)/gm,
          '<span class="token comment">$1</span>'
        );
        break;
        
      case 'rust':
        // Keywords
        highlightedCode = highlightedCode.replace(
          /\b(fn|let|mut|const|struct|enum|impl|trait|pub|use|mod|if|else|match|while|for|return)\b/g,
          '<span class="token keyword">$1</span>'
        );
        // Strings
        highlightedCode = highlightedCode.replace(
          /(["'])([^"']*?)\1/g,
          '<span class="token string">$1$2$1</span>'
        );
        break;
        
      case 'python':
        // Keywords
        highlightedCode = highlightedCode.replace(
          /\b(def|class|if|else|for|while|return|import|from|try|except)\b/g,
          '<span class="token keyword">$1</span>'
        );
        // Strings
        highlightedCode = highlightedCode.replace(
          /(["'])([^"']*?)\1/g,
          '<span class="token string">$1$2$1</span>'
        );
        break;
    }
  } catch (error) {
    console.warn('Syntax highlighting error:', error);
    return highlightedCode;
  }
  
  return highlightedCode;
}

// Enhanced code block preview with syntax highlighting
const enhancedCodePreview = computed(() => {
  if (!hasCodeBlocks.value) return '';
  
  let preview = message.value;
  
  // Replace code blocks with enhanced syntax highlighting
  preview = preview.replace(/```(\w+)?\n([\s\S]*?)```/g, (match, lang, code) => {
    const language = lang || 'text';
    const trimmedCode = code.trim();
    const highlightedCode = applySyntaxHighlighting(trimmedCode, language);
    
    // Create a more VS Code-like preview
    return '<div class="code-block-container">' +
      '<div class="code-block-header">' +
        '<div class="flex items-center justify-between">' +
          '<span class="language-tag">' + language + '</span>' +
          '<div class="code-block-controls">' +
            '<button class="copy-btn" title="Copy code">' +
              '<svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">' +
                '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"></path>' +
              '</svg>' +
            '</button>' +
          '</div>' +
        '</div>' +
      '</div>' +
      '<div class="code-block-content">' +
        '<pre class="language-' + language + '"><code>' + highlightedCode + '</code></pre>' +
      '</div>' +
    '</div>';
  });
  
  // Handle remaining text (non-code parts)
  preview = preview.replace(/\n/g, '<br>');
  
  return preview;
});

// Legacy code block preview (keeping for backward compatibility)
const codeBlockPreview = computed(() => {
  return enhancedCodePreview.value;
});

// Helper function to escape HTML
function escapeHtml(text) {
  const map = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#039;'
  };
  return text.replace(/[&<>"']/g, m => map[m]);
}

// Toggle preview
function togglePreview() {
  showPreview.value = !showPreview.value;
}

// Platform detection
const isMac = computed(() => {
  return navigator.platform.toUpperCase().indexOf('MAC') >= 0;
});

// File Handling Functions
function handleKeyDown(event) {
  // Handle code block shortcut first
  if ((event.metaKey || event.ctrlKey) && event.key === 'e') {
    handleCodeBlockShortcut(event);
    return;
  }
  
  if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) {
    event.preventDefault();
    if (canSend.value) {
      sendMessage();
    }
  }
}

function adjustHeight() {
  nextTick(() => {
    const textarea = messageInput.value;
    if (textarea) {
      textarea.style.height = 'auto';
      textarea.style.height = Math.min(textarea.scrollHeight, 120) + 'px';
    }
  });
  
  // Handle typing indicator
  handleTypingIndicator();
}

// Typing indicator handler
async function handleTypingIndicator() {
  // Only send typing status for DM/Single chats
  if (props.chatType !== 'Single') {
    return;
  }
  
  if (!props.chatId || !message.value.trim()) {
    // Stop typing if message is empty
    if (isTyping) {
      stopTyping();
    }
    return;
  }
  
  if (!isTyping) {
    // Start typing
    try {
      await api.post('/realtime/typing/start', { chat_id: props.chatId });
      isTyping = true;
      } catch (error) {
      console.error('Failed to send typing status:', error);
    }
  }
  
  // Reset typing timer
  clearTimeout(typingTimer);
  typingTimer = setTimeout(() => {
    stopTyping();
  }, 3000); // Stop after 3 seconds of no typing
}

// Stop typing
async function stopTyping() {
  if (!isTyping || !props.chatId || props.chatType !== 'Single') return;
  
  try {
    await api.post('/realtime/typing/stop', { chat_id: props.chatId });
    isTyping = false;
    } catch (error) {
    console.error('Failed to send stop typing status:', error);
  }
  
  clearTimeout(typingTimer);
}

// Enhanced Drag Upload Handling
function handleDragOver(event) {
  event.preventDefault();
  if (!isDragging.value) {
    isDragging.value = true;
  }
}

function handleDragLeave(event) {
  // Only set to false if we're leaving the component entirely
  if (!event.currentTarget.contains(event.relatedTarget)) {
    isDragging.value = false;
  }
}

function handleDrop(event) {
  event.preventDefault();
  isDragging.value = false;
  
  const files = Array.from(event.dataTransfer.files);
  if (files.length > 0) {
    addFiles(files);
  }
}

function handleFileSelect(event) {
  const files = Array.from(event.target.files);
  addFiles(files);
  event.target.value = ''; // Clear input
}

// File size constants
const maxFileSize = ref(50 * 1024 * 1024); // 50MB

function addFiles(files) {
  const maxFiles = 10;
  
  const validFiles = files.filter(file => {
    if (file.size > maxFileSize.value) {
      error.value = `File "${file.name}" is too large. Maximum size is 50MB.`;
      return false;
    }
    return true;
  });
  
  if (selectedFiles.value.length + validFiles.length > maxFiles) {
    error.value = `Maximum ${maxFiles} files allowed.`;
    return;
  }
  
  // Add unique IDs to files for better tracking
  const filesWithIds = validFiles.map(file => ({
    ...file,
    id: Date.now() + Math.random(),
    loading: false,
    progress: 0,
    error: false
  }));
  
  selectedFiles.value.push(...filesWithIds);
  error.value = '';
}

// Enhanced file drop handler
function handleFilesDropped(droppedFiles) {
  addFiles(droppedFiles);
}

function removeFile(index) {
  selectedFiles.value.splice(index, 1);
}

function clearFiles() {
  selectedFiles.value = [];
  error.value = '';
}

function getFilePreview(file) {
  return URL.createObjectURL(file);
}

function formatFileSize(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

// Emoji Handling
function insertEmoji(emoji) {
  const textarea = messageInput.value;
  const cursorPos = textarea.selectionStart;
  const textBefore = message.value.substring(0, cursorPos);
  const textAfter = message.value.substring(cursorPos);
  message.value = textBefore + emoji + textAfter;
  
  nextTick(() => {
    textarea.focus();
    textarea.setSelectionRange(cursorPos + emoji.length, cursorPos + emoji.length);
    adjustHeight();
  });
  
  showEmojiPicker.value = false;
}

// Get Current Category Emojis
function getCurrentEmojis() {
  const category = emojiCategories.value.find(cat => cat.name === activeEmojiCategory.value);
  return category ? category.emojis : [];
}

// 🔧 优化后的发送消息函数
async function sendMessage() {
  if (!canSend.value) {
    return;
  }
  
  // 🔧 防重复点击检查
  const now = Date.now();
  if (isSending.value || (now - lastSendTime.value) < 1000) {
    return;
  }
  
  try {
    // 🔧 标记开始发送
    isSending.value = true;
    lastSendTime.value = now;
    uploading.value = true;
    error.value = '';
    uploadProgress.value = 0;
    
    // Stop typing indicator
    stopTyping();
    
    // 🔧 保存当前消息内容和文件（防止在发送过程中被修改）
    const messageContent = message.value.trim();
    const currentFiles = [...selectedFiles.value];
    
    // Upload files first if any
    let uploadedFiles = [];
    if (currentFiles.length > 0) {
      // Update file loading status
      currentFiles.forEach(file => {
        file.loading = true;
        file.progress = 0;
      });
      
      try {
        uploadedFiles = await chatStore.uploadFiles(currentFiles, {
          onProgress: (fileIndex, progress) => {
            if (currentFiles[fileIndex]) {
              currentFiles[fileIndex].progress = progress;
            }
          }
        });
        
        // Mark files as successfully uploaded
        currentFiles.forEach(file => {
          file.loading = false;
          file.progress = 100;
        });
      } catch (error) {
        // Mark files as failed
        currentFiles.forEach(file => {
          file.loading = false;
          file.error = true;
        });
        throw error;
      }
    }
    
    // Emit send event
    emit('send', {
      content: messageContent,
      files: uploadedFiles
    });
    
    // 🔧 只有在发送成功后才清空表单
    // 这样如果发送失败，用户不会丢失输入的内容
    message.value = '';
    selectedFiles.value = [];
    adjustHeight();
    
    } catch (err) {
    console.error('📤 [INPUT] Message send failed:', err);
    error.value = err.message || 'Failed to send message';
    
    // 🔧 发送失败时不清空表单，让用户可以重试
    showNotification('Failed to send message. Please try again.', 'error');
    
  } finally {
    // 🔧 重置发送状态
    uploading.value = false;
    uploadProgress.value = 0;
    
    // 🔧 延迟重置发送状态，防止立即重复点击
    setTimeout(() => {
      isSending.value = false;
    }, 500);
  }
}

// Monitor Upload Progress
function updateUploadProgress() {
  uploadProgress.value = chatStore.uploadProgress;
}

// Close Emoji Picker on Outside Click
function handleClickOutside(event) {
  if (!event.target.closest('.relative') && showEmojiPicker.value) {
    showEmojiPicker.value = false;
  }
}

// Lifecycle
onMounted(() => {
  document.addEventListener('click', handleClickOutside);
  document.addEventListener('dragover', (e) => e.preventDefault());
  document.addEventListener('drop', (e) => e.preventDefault());
  adjustHeight();
  
  // Monitor store upload progress
  chatStore.$subscribe((mutation, state) => {
    if (mutation.type === 'direct' && mutation.payload.uploadProgress !== undefined) {
      uploadProgress.value = state.uploadProgress;
    }
  });
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
  document.removeEventListener('dragover', (e) => e.preventDefault());
  document.removeEventListener('drop', (e) => e.preventDefault());
  
  // Clean up typing indicator
  clearTimeout(typingTimer);
  if (isTyping) {
    stopTyping();
  }
  
  // Clean up object URLs
  selectedFiles.value.forEach(file => {
    if (file.type.startsWith('image/')) {
      URL.revokeObjectURL(getFilePreview(file));
    }
  });
});

// File Handling Functions - Performance Optimization
function isImageFile(file) {
  return file.type.startsWith('image/');
}

// File Upload - 现在由chatStore.uploadFiles处理
// 这个函数已经被移除，文件上传在sendMessage中通过chatStore.uploadFiles完成
</script>

<style scoped>
/* Custom Scrollbar */
.scrollbar-thin::-webkit-scrollbar {
  width: 4px;
}

.scrollbar-thin::-webkit-scrollbar-track {
  background: transparent;
}

.scrollbar-thin::-webkit-scrollbar-thumb {
  background: rgba(156, 163, 175, 0.3);
  border-radius: 2px;
}

.scrollbar-thin::-webkit-scrollbar-thumb:hover {
  background: rgba(156, 163, 175, 0.5);
}

/* Keyboard Hint */
.kbd-hint {
  @apply inline-flex items-center px-1.5 py-0.5 text-xs bg-gray-100 border border-gray-200 rounded;
  font-family: 'SF Mono', 'Monaco', 'Consolas', monospace;
}

/* Smooth Font Rendering */
* {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* Enhanced Code Block Preview */
.enhanced-code-preview {
  font-family: 'SF Mono', 'Monaco', 'Consolas', 'Menlo', monospace;
  font-size: 13px;
  line-height: 1.5;
}

.code-block-container {
  @apply mb-3 last:mb-0 bg-slate-900 rounded-b-lg overflow-hidden;
  border: 1px solid #e2e8f0;
}

.code-block-header {
  @apply bg-slate-800 px-3 py-2 border-b border-slate-700;
}

.language-tag {
  @apply text-xs font-medium text-slate-300 bg-slate-700 px-2 py-1 rounded;
}

.code-block-controls {
  @apply flex items-center space-x-1;
}

.copy-btn {
  @apply text-slate-400 hover:text-slate-200 p-1 rounded transition-colors;
}

.code-block-content {
  @apply bg-slate-900 p-3 overflow-x-auto;
}

.code-block-content pre {
  @apply m-0 text-slate-100;
  white-space: pre;
  word-break: normal;
  overflow-wrap: normal;
}

.code-block-content code {
  @apply text-slate-100 bg-transparent;
  font-family: inherit;
  font-size: inherit;
  padding: 0;
  border-radius: 0;
}

/* Syntax highlighting for common languages */
.language-javascript .token.keyword,
.language-js .token.keyword,
.language-typescript .token.keyword,
.language-ts .token.keyword {
  color: #c792ea;
}

.language-javascript .token.string,
.language-js .token.string,
.language-typescript .token.string,
.language-ts .token.string {
  color: #c3e88d;
}

.language-javascript .token.function,
.language-js .token.function,
.language-typescript .token.function,
.language-ts .token.function {
  color: #82aaff;
}

.language-javascript .token.comment,
.language-js .token.comment,
.language-typescript .token.comment,
.language-ts .token.comment {
  color: #546e7a;
  font-style: italic;
}

.language-javascript .token.number,
.language-js .token.number,
.language-typescript .token.number,
.language-ts .token.number {
  color: #f78c6c;
}

/* Rust syntax highlighting */
.language-rust .token.keyword {
  color: #c792ea;
}

.language-rust .token.string {
  color: #c3e88d;
}

.language-rust .token.macro {
  color: #ffcb6b;
}

/* Python syntax highlighting */
.language-python .token.keyword {
  color: #c792ea;
}

.language-python .token.string {
  color: #c3e88d;
}

.language-python .token.function {
  color: #82aaff;
}

/* Legacy Code Block Preview (fallback) */
.code-block-preview {
  @apply font-mono text-sm bg-gray-50 border border-gray-200 rounded-lg p-3;
  white-space: pre-wrap;
  word-break: break-word;
}

.code-block-preview pre {
  margin: 0;
  font-family: 'SF Mono', 'Monaco', 'Consolas', monospace;
}

.code-block-preview code {
  @apply bg-gray-100 px-1 py-0.5 rounded text-gray-800;
}

/* 📱 Mobile MessageInput Optimizations */
@media (max-width: 767px) {
  /* Container adjustments */
  .bg-white\/95 {
    padding-bottom: env(safe-area-inset-bottom, 0);
  }

  /* Main input area - more compact on mobile */
  .p-6 {
    padding: 12px 16px;
  }

  /* File upload button - larger touch target */
  .w-10.h-10 {
    width: 44px;
    height: 44px;
    min-width: 44px;
    min-height: 44px;
  }

  /* Send button - larger touch target */
  .w-10.h-10.bg-violet-600 {
    width: 44px;
    height: 44px;
    min-width: 44px;
    min-height: 44px;
  }

  /* Message input - adjust for mobile */
  textarea {
    font-size: 16px !important; /* Prevent zoom on iOS */
    min-height: 44px !important;
    padding: 12px 16px !important;
  }

  /* Emoji picker - full width on mobile */
  .w-80 {
    width: calc(100vw - 32px);
    max-width: 320px;
    left: 50%;
    transform: translateX(-50%);
    right: auto;
  }

  /* Emoji picker positioning */
  .bottom-14 {
    bottom: 60px;
  }

  /* Selected files grid - fewer columns on mobile */
  .grid-cols-2 {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  
  .sm\:grid-cols-3 {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  
  .md\:grid-cols-4 {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
  
  .lg\:grid-cols-6 {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  /* File preview items - larger touch targets */
  .relative.group {
    min-height: 80px;
    padding: 12px 8px;
  }

  /* Code block preview - smaller on mobile */
  .code-block-preview {
    font-size: 12px;
    padding: 8px;
  }

  /* Drag upload overlay - adjust for mobile */
  .w-16.h-16 {
    width: 48px;
    height: 48px;
  }

  /* Upload progress - more compact */
  .px-6.py-2 {
    padding: 8px 16px;
  }

  /* Character count - adjust position */
  .absolute.bottom-2.right-4 {
    bottom: 8px;
    right: 8px;
    font-size: 11px;
  }

  /* Code block hint - adjust position */
  .absolute.bottom-1.right-4 {
    bottom: 4px;
    right: 8px;
    font-size: 11px;
  }

  /* Keyboard hint */
  .kbd-hint {
    padding: 2px 4px;
    font-size: 10px;
  }
}

/* 📱 Small mobile devices */
@media (max-width: 479px) {
  /* Even more compact layout */
  .p-6 {
    padding: 8px 12px;
  }

  /* Reduce spacing between elements */
  .space-x-3 > * + * {
    margin-left: 8px;
  }

  /* Smaller emoji picker */
  .w-80 {
    width: calc(100vw - 24px);
    max-width: 280px;
  }

  /* Smaller emoji grid */
  .grid-cols-8 {
    grid-template-columns: repeat(6, minmax(0, 1fr));
  }

  /* Single column for file preview on very small screens */
  .grid-cols-2,
  .sm\:grid-cols-3,
  .md\:grid-cols-4,
  .lg\:grid-cols-6 {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  /* Hide some less important UI elements on very small screens */
  .slack-latency,
  .code-block-preview {
    display: none;
  }
}

/* 📱 Landscape mobile optimization */
@media (max-width: 767px) and (orientation: landscape) {
  /* Reduce top/bottom padding in landscape */
  .p-6 {
    padding: 8px 16px;
  }

  /* Adjust emoji picker height for landscape */
  .max-h-48 {
    max-height: 30vh;
  }

  /* Smaller upload overlay in landscape */
  .w-16.h-16 {
    width: 40px;
    height: 40px;
  }
}

/* 🎯 Touch-friendly improvements */
@media (hover: none) and (pointer: coarse) {
  /* Ensure all interactive elements meet touch size requirements */
  button, .group button {
    min-height: 44px;
    min-width: 44px;
  }

  /* Larger touch targets for emoji buttons */
  .w-8.h-8 {
    width: 44px;
    height: 44px;
    min-width: 44px;
    min-height: 44px;
  }

  /* Improve touch feedback */
  button:active {
    transform: scale(0.95);
    transition: transform 0.1s ease;
  }

  /* Better touch spacing */
  .gap-1 {
    gap: 8px;
  }

  .gap-2 {
    gap: 12px;
  }
}
</style>