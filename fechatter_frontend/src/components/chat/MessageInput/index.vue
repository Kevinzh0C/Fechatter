<template>
  <div class="message-input" :class="{ 'has-preview': showPreview }" style="position: relative;">
    <!-- Preview Container (at top) -->
    <div v-if="showPreview" class="preview-container">
      <div class="preview-header">
        <span class="preview-title">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
            <circle cx="12" cy="12" r="3"></circle>
          </svg>
          {{ getPreviewTitle() }}
        </span>
        <button @click="closePreview" class="preview-close">×</button>
      </div>
      <div class="preview-content">
        <!-- Markdown Preview -->
        <div v-if="formatMode === 'markdown'" class="markdown-preview" v-html="renderedMarkdown"></div>

        <!-- Code Preview -->
        <div v-else-if="formatMode === 'code'" class="code-preview">
          <div class="code-header">
            <select v-model="selectedLanguage" class="language-selector">
              <option value="javascript">JavaScript</option>
              <option value="typescript">TypeScript</option>
              <option value="python">Python</option>
              <option value="rust">Rust</option>
              <option value="html">HTML</option>
              <option value="css">CSS</option>
              <option value="json">JSON</option>
              <option value="sql">SQL</option>
              <option value="bash">Bash</option>
            </select>
            <span class="code-info">{{ messageContent.split('\n').length }} lines</span>
          </div>
          <pre><code :class="`language-${selectedLanguage}`">{{ messageContent || 'Start typing code...' }}</code></pre>
        </div>

        <!-- File Preview -->
        <FilePreview v-else-if="formatMode === 'file' && files.length > 0" :file="files[0]"
          @file-uploaded="handleFileUploaded" @upload-error="handleFileUploadError" @file-removed="handleFileRemoved"
          @trigger-upload="handleTriggerUpload" />
      </div>
    </div>

    <!-- Enhanced Markdown Toolbar (in middle) -->
    <MarkdownToolbar v-if="formatMode === 'markdown'" :show-preview="showPreview" @insert-markdown="insertMarkdown"
      @insert-code-block="insertCodeBlock" @insert-table="insertTable" @toggle-preview="toggleMarkdownPreview"
      @clear-content="clearContent" />

    <!-- Emoji Picker (positioned above input) -->
    <div v-if="showEmojiPicker" class="emoji-picker-overlay" @click="handleEmojiOverlayClick">
      <div ref="emojiPickerRef" class="emoji-picker-container" @click.stop>
        <div class="emoji-picker-header">
          <h4>Select Emoji</h4>
          <button @click="showEmojiPicker = false" class="emoji-close-btn">×</button>
        </div>

        <div class="emoji-picker-content">
          <!-- Search -->
          <div class="emoji-search">
            <input type="text" v-model="emojiSearchQuery" placeholder="Search emojis..." class="emoji-search-input">
          </div>

          <!-- Categories -->
          <div class="emoji-categories">
            <button v-for="(emojis, category) in emojiCategories" :key="category" @click="selectedCategory = category"
              :class="{ active: selectedCategory === category }" class="category-btn">
              {{ getCategoryIcon(category) }} {{ getCategoryName(category) }}
            </button>
          </div>

          <!-- Emoji Grid -->
          <div class="emoji-grid">
            <button v-for="emojiItem in filteredEmojis" :key="emojiItem.name" @click="handleEmojiSelect(emojiItem)"
              class="emoji-item" :title="emojiItem.name">
              {{ emojiItem.emoji }}
            </button>
          </div>

          <!-- Recently Used -->
          <div v-if="recentEmojis.length > 0" class="recent-emojis">
            <h5>Recently Used</h5>
            <div class="emoji-grid">
              <button v-for="emojiItem in recentEmojis" :key="'recent-' + emojiItem.name"
                @click="handleEmojiSelect(emojiItem)" class="emoji-item" :title="emojiItem.name">
                {{ emojiItem.emoji }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Main Input Area (at bottom) -->
    <div class="main-input-area">
      <!-- File Upload Button -->
      <button @click="triggerFileUpload" class="input-btn file-btn" :class="{ active: formatMode === 'file' }"
        title="Attach file">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
        </svg>
      </button>

      <!-- Input Container -->
      <div class="input-container">
        <textarea ref="messageInput" v-model="messageContent" @keydown="handleKeyDown" @input="handleInput"
          @paste="handlePaste" :placeholder="placeholderText" class="message-textarea" rows="1">
        </textarea>
      </div>

      <!-- Mode Button -->
      <button @click="cycleFormatMode" class="input-btn mode-btn" :class="{
        'mode-markdown': formatMode === 'markdown',
        'mode-code': formatMode === 'code'
      }" :title="getFormatModeTooltip()">
        <svg v-if="formatMode === 'text'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor"
          stroke-width="2">
          <path d="M4 7V4h16v3M9 20h6M12 4v16"></path>
        </svg>
        <svg v-else-if="formatMode === 'markdown'" width="20" height="20" viewBox="0 0 24 24" fill="none"
          stroke="currentColor" stroke-width="2">
          <path d="M3 5h18v14H3zM7 15V9l2 2 2-2v6m3-2h4"></path>
        </svg>
        <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="16,18 22,12 16,6"></polyline>
          <polyline points="8,6 2,12 8,18"></polyline>
        </svg>
      </button>

      <!-- Emoji Button -->
      <button @click="toggleEmojiPicker" class="input-btn emoji-btn" :class="{ active: showEmojiPicker }"
        title="Add emoji">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"></circle>
          <path d="M8 14s1.5 2 4 2 4-2 4-2"></path>
          <line x1="9" y1="9" x2="9.01" y2="9"></line>
          <line x1="15" y1="9" x2="15.01" y2="9"></line>
        </svg>
      </button>

      <!-- Enhanced Send Button with Dynamic State -->
      <button @click="sendMessage" :disabled="!canSend" class="input-btn send-btn" :class="{
        'active': canSend,
        'empty': !messageContent.trim(),
        'filled': messageContent.trim(),
        'pulse': canSend && messageContent.trim().length > 5
      }">
        <svg v-if="!isSending" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor"
          stroke-width="2">
          <line x1="22" y1="2" x2="11" y2="13"></line>
          <polygon points="22,2 15,22 11,13 2,9"></polygon>
        </svg>
        <div v-else class="loading-spinner"></div>
      </button>
    </div>

    <!-- Hidden file input -->
    <input ref="fileInput" type="file" multiple @change="handleFileSelect" style="display: none">
  </div>
</template>

<script setup>
import { ref, computed, nextTick, onMounted, onUnmounted, watch } from 'vue';
import MarkdownIt from 'markdown-it';
import DOMPurify from 'dompurify';
import FilePreview from './FilePreview.vue';
import MarkdownToolbar from './MarkdownToolbar.vue';

const props = defineProps({
  chatId: { type: [Number, String], required: true },
  currentUserId: { type: [Number, String], default: null },
  replyToMessage: { type: Object, default: null },
  disabled: { type: Boolean, default: false },
  maxLength: { type: Number, default: 2000 }
});

const emit = defineEmits(['message-sent', 'reply-cancelled', 'preview-state-change']);

// Core state - keeping existing functionality
const messageContent = ref('');
const showPreview = ref(false);
const formatMode = ref('text');
const isSending = ref(false);
const selectedLanguage = ref('javascript');
const files = ref([]);
const uploadedFileUrl = ref(''); // 🎯 新增：存储上传成功的文件URL
const uploadedFileInfo = ref(null); // 🎯 新增：存储上传成功的文件信息
const showEmojiPicker = ref(false);
const emojiSearchQuery = ref('');
const selectedCategory = ref('smileys');
const recentEmojis = ref([]);

// 🎯 专业企业表情数据
const enterpriseEmojis = ref([
  // 常用表情
  { emoji: '😊', name: 'smile', category: 'smileys' },
  { emoji: '😃', name: 'grin', category: 'smileys' },
  { emoji: '😄', name: 'laugh', category: 'smileys' },
  { emoji: '😁', name: 'grinning', category: 'smileys' },
  { emoji: '😅', name: 'sweat_smile', category: 'smileys' },
  { emoji: '😂', name: 'joy', category: 'smileys' },
  { emoji: '🤣', name: 'rofl', category: 'smileys' },
  { emoji: '😭', name: 'sob', category: 'smileys' },

  // 工作表情
  { emoji: '👍', name: 'thumbsup', category: 'people' },
  { emoji: '👎', name: 'thumbsdown', category: 'people' },
  { emoji: '👏', name: 'clap', category: 'people' },
  { emoji: '🙌', name: 'raised_hands', category: 'people' },
  { emoji: '🤝', name: 'handshake', category: 'people' },
  { emoji: '💪', name: 'muscle', category: 'people' },
  { emoji: '🤞', name: 'crossed_fingers', category: 'people' },
  { emoji: '✌️', name: 'victory', category: 'people' },

  // 心情表情
  { emoji: '❤️', name: 'heart', category: 'symbols' },
  { emoji: '💖', name: 'sparkling_heart', category: 'symbols' },
  { emoji: '💯', name: 'hundred', category: 'symbols' },
  { emoji: '🔥', name: 'fire', category: 'symbols' },
  { emoji: '⭐', name: 'star', category: 'symbols' },
  { emoji: '✨', name: 'sparkles', category: 'symbols' },
  { emoji: '⚡', name: 'zap', category: 'symbols' },
  { emoji: '💎', name: 'gem', category: 'symbols' },

  // 状态表情
  { emoji: '✅', name: 'white_check_mark', category: 'symbols' },
  { emoji: '❌', name: 'x', category: 'symbols' },
  { emoji: '⚠️', name: 'warning', category: 'symbols' },
  { emoji: '🚀', name: 'rocket', category: 'travel' },
  { emoji: '🎉', name: 'tada', category: 'objects' },
  { emoji: '🎊', name: 'confetti_ball', category: 'objects' },
  { emoji: '🎯', name: 'dart', category: 'objects' },
  { emoji: '📈', name: 'chart_increasing', category: 'objects' },

  // 技术表情
  { emoji: '💻', name: 'computer', category: 'objects' },
  { emoji: '📱', name: 'mobile_phone', category: 'objects' },
  { emoji: '🖥️', name: 'desktop_computer', category: 'objects' },
  { emoji: '⌨️', name: 'keyboard', category: 'objects' },
  { emoji: '🖱️', name: 'computer_mouse', category: 'objects' },
  { emoji: '💾', name: 'floppy_disk', category: 'objects' },
  { emoji: '🔧', name: 'wrench', category: 'objects' },
  { emoji: '⚙️', name: 'gear', category: 'objects' }
]);

// Refs
const messageInput = ref(null);
const fileInput = ref(null);
const emojiPickerRef = ref(null);

// Computed properties
const canSend = computed(() => {
  // 🎯 严格的发送逻辑：必须先上传文件到远端获取URL
  const hasContent = messageContent.value.trim().length > 0;
  const hasLocalFiles = files.value.length > 0;
  const hasUploadedFileUrl = uploadedFileUrl.value.trim().length > 0;
  const notSending = !isSending.value;

  // 🚨 CRITICAL: 如果有本地文件但没有远端URL，绝对不能发送
  if (hasLocalFiles && !hasUploadedFileUrl) {
    return false;
  }

  // ✅ 只有这两种情况可以发送：
  // 1. 纯文本消息（没有任何文件）
  // 2. 有远端文件URL（无论是否有文本内容）
  const canSendText = hasContent && !hasLocalFiles && !hasUploadedFileUrl;
  const canSendFile = hasUploadedFileUrl;
  
  const result = (canSendText || canSendFile) && notSending;

  // 🔍 详细状态日志
  if (hasLocalFiles && !hasUploadedFileUrl) {
    console.log('🚫 [MessageInput] Send BLOCKED: Local files need remote upload first');
  } else if (canSendFile) {
    console.log('✅ [MessageInput] Send ENABLED: Remote file URL ready');
  } else if (canSendText) {
    console.log('✅ [MessageInput] Send ENABLED: Text-only message');
  } else {
    console.log('⚪ [MessageInput] Send DISABLED: No content or files');
  }

  return result;
});

const placeholderText = computed(() => {
  const hasLocalFiles = files.value.length > 0;
  const hasUploadedFileUrl = uploadedFileUrl.value.trim().length > 0;

  // 🎯 根据文件上传状态显示不同提示
  if (hasUploadedFileUrl) {
    return 'File uploaded! Ready to send...';
  } else if (hasLocalFiles) {
    return 'Files selected - upload to remote first...';
  }

  if (formatMode.value === 'markdown') {
    return 'Type in Markdown... **bold**, *italic*, `code`';
  } else if (formatMode.value === 'code') {
    return 'Type code... (Tab to indent, Shift+Tab to unindent)';
  } else if (formatMode.value === 'file') {
    return 'Add files to share...';
  }
  return 'Type a message...';
});

// 🎯 表情分类
const emojiCategories = computed(() => {
  const categories = {};
  enterpriseEmojis.value.forEach(item => {
    if (!categories[item.category]) {
      categories[item.category] = [];
    }
    categories[item.category].push(item);
  });
  return categories;
});

// 🎯 过滤后的表情
const filteredEmojis = computed(() => {
  let emojis = enterpriseEmojis.value;

  // 按分类过滤
  if (selectedCategory.value && selectedCategory.value !== 'all') {
    emojis = emojis.filter(item => item.category === selectedCategory.value);
  }

  // 按搜索词过滤
  if (emojiSearchQuery.value.trim()) {
    const query = emojiSearchQuery.value.toLowerCase();
    emojis = emojis.filter(item =>
      item.name.toLowerCase().includes(query) ||
      item.emoji.includes(query)
    );
  }

  return emojis;
});

// 🎯 表情分类相关方法
const getCategoryIcon = (category) => {
  const icons = {
    smileys: '😊',
    people: '👍',
    symbols: '❤️',
    objects: '💻',
    travel: '🚀'
  };
  return icons[category] || '📂';
};

const getCategoryName = (category) => {
  const names = {
    smileys: 'Smileys',
    people: 'People',
    symbols: 'Symbols',
    objects: 'Objects',
    travel: 'Travel'
  };
  return names[category] || category;
};

// Initialize markdown-it instance with stable configuration
const md = new MarkdownIt({
  html: true,          // Enable HTML tags in source
  breaks: true,        // Convert '\n' in paragraphs into <br>
  linkify: true,       // Autoconvert URL-like text to links
  typographer: true,   // Enable smartquotes and other typographic features
});

// Enhanced renderer overrides
const defaultRender = md.renderer.rules.link_open || function (tokens, idx, options, env, renderer) {
  return renderer.renderToken(tokens, idx, options);
};

// Enhanced link rendering with external indicators
md.renderer.rules.link_open = function (tokens, idx, options, env, renderer) {
  const token = tokens[idx];
  const href = token.attrGet('href');

  if (href && (href.startsWith('http') || href.startsWith('//'))) {
    token.attrSet('target', '_blank');
    token.attrSet('rel', 'noopener noreferrer');
    token.attrPush(['class', 'markdown-link external-link']);
  } else {
    token.attrPush(['class', 'markdown-link']);
  }

  return defaultRender(tokens, idx, options, env, renderer);
};

// Enhanced code block rendering
md.renderer.rules.fence = function (tokens, idx, options, env, renderer) {
  const token = tokens[idx];
  const info = token.info ? token.info.trim() : '';
  const langName = info ? info.split(/\s+/g)[0] : '';
  const langLabel = langName ? `<div class="code-language">${langName}</div>` : '';

  return `<div class="code-block">${langLabel}<pre><code class="hljs language-${langName}">${token.content}</code></pre></div>`;
};

// Enhanced blockquote rendering
md.renderer.rules.blockquote_open = function () {
  return '<blockquote class="enhanced-quote">\n';
};

// Enhanced heading rendering with anchors
md.renderer.rules.heading_open = function (tokens, idx) {
  const token = tokens[idx];
  const level = token.markup.length;
  const next = tokens[idx + 1];
  const content = next && next.type === 'inline' ? next.content : '';
  const id = content.toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-');

  token.attrPush(['class', 'markdown-heading']);
  token.attrPush(['id', id]);

  return `<h${level} class="markdown-heading" id="${id}">`;
};

// Enhanced table rendering
md.renderer.rules.table_open = function () {
  return '<div class="table-wrapper"><table class="enhanced-table">\n';
};

md.renderer.rules.table_close = function () {
  return '</table></div>\n';
};

// Enhanced image rendering
md.renderer.rules.image = function (tokens, idx) {
  const token = tokens[idx];
  const src = token.attrGet('src');
  const alt = token.attrGet('alt') || '';
  const title = token.attrGet('title') ? ` title="${token.attrGet('title')}"` : '';

  return `<div class="image-wrapper"><img src="${src}" alt="${alt}"${title} class="markdown-image" loading="lazy" /></div>`;
};

const renderedMarkdown = computed(() => {
  if (!messageContent.value.trim()) {
    return `<div class="empty-preview">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 5h18v14H3zM7 15V9l2 2 2-2v6m3-2h4"></path>
      </svg>
      <p>开始输入Markdown，实时预览...</p>
      <div class="preview-examples">
        <span>试试: **粗体**, *斜体*, \`代码\`, > 引用</span>
      </div>
    </div>`;
  }

  try {
    // Render markdown to HTML
    let html = md.render(messageContent.value);

    // Post-process for task lists (GitHub style)
    html = html.replace(
      /\[ \]/g,
      '<input type="checkbox" disabled class="task-checkbox">'
    );
    html = html.replace(
      /\[x\]/gi,
      '<input type="checkbox" checked disabled class="task-checkbox task-checked">'
    );

    // Post-process for highlights ==text==
    html = html.replace(
      /==(.*?)==/g,
      '<mark class="markdown-highlight">$1</mark>'
    );

    // Add external link icons
    html = html.replace(
      /<a([^>]*class="[^"]*external-link[^"]*"[^>]*)>([^<]*)<\/a>/g,
      '<a$1>$2 <svg class="external-link-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15,3 21,3 21,9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg></a>'
    );

    // Add task list item classes
    html = html.replace(
      /<li><input type="checkbox"/g,
      '<li class="task-list-item"><input type="checkbox"'
    );

    // Sanitize the final HTML
    const sanitizedHtml = DOMPurify.sanitize(html, {
      ALLOWED_TAGS: [
        'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
        'p', 'br', 'strong', 'em', 'u', 's', 'del',
        'a', 'img', 'code', 'pre', 'kbd',
        'blockquote', 'ul', 'ol', 'li',
        'table', 'thead', 'tbody', 'tr', 'th', 'td',
        'div', 'span', 'mark', 'input', 'svg', 'path', 'polyline', 'line'
      ],
      ALLOWED_ATTR: [
        'href', 'title', 'target', 'rel', 'src', 'alt', 'loading',
        'id', 'class', 'type', 'checked', 'disabled',
        'width', 'height', 'viewBox', 'fill', 'stroke', 'stroke-width',
        'x1', 'y1', 'x2', 'y2', 'points', 'd'
      ],
      ALLOWED_URI_REGEXP: /^(?:(?:(?:f|ht)tps?|mailto|tel|callto|cid|xmpp|data):|[^a-z]|[a-z+.\-]+(?:[^a-z+.\-:]|$))/i
    });

    return sanitizedHtml;

  } catch (error) {
    console.error('Markdown渲染错误:', error);
    return `<div class="error-preview">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
        <line x1="12" y1="9" x2="12" y2="13"/>
        <line x1="12" y1="17" x2="12.01" y2="17"/>
      </svg>
      <p>Markdown渲染错误</p>
      <small>请检查Markdown语法后重试</small>
    </div>`;
  }
});

// Component methods - keeping core functionality
const getPreviewTitle = () => {
  if (formatMode.value === 'markdown') return 'Markdown Preview';
  if (formatMode.value === 'code') return `Code Preview (${selectedLanguage.value})`;
  if (formatMode.value === 'file') return 'File Preview';
  return 'Preview';
};

const cycleFormatMode = () => {
  if (formatMode.value === 'text') {
    formatMode.value = 'markdown';
    showPreview.value = false;
  } else if (formatMode.value === 'markdown') {
    formatMode.value = 'code';
    showPreview.value = true;
  } else {
    formatMode.value = 'text';
    showPreview.value = false;
  }
  emit('preview-state-change', showPreview.value);
};

const getFormatModeTooltip = () => {
  if (formatMode.value === 'text') return 'Switch to Markdown mode';
  if (formatMode.value === 'markdown') return 'Switch to Code mode';
  return 'Switch to Text mode';
};

const closePreview = () => {
  showPreview.value = false;
  emit('preview-state-change', false);
};

const toggleMarkdownPreview = () => {
  if (formatMode.value === 'markdown') {
    showPreview.value = !showPreview.value;
    emit('preview-state-change', showPreview.value);
  }
};

const insertMarkdown = (before, after = '') => {
  const textarea = messageInput.value;
  if (!textarea) return;

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const selectedText = messageContent.value.substring(start, end);

  const newText = before + selectedText + after;
  messageContent.value = messageContent.value.substring(0, start) + newText + messageContent.value.substring(end);

  nextTick(() => {
    const newCursorPos = start + before.length + selectedText.length;
    textarea.setSelectionRange(newCursorPos, newCursorPos);
    textarea.focus();
  });
};

// Code indentation for Tab key in code mode
const insertCodeIndentation = () => {
  const textarea = messageInput.value;
  if (!textarea) return;

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const indentation = '    '; // 4 spaces for code indentation

  if (start === end) {
    // No selection: insert indentation at cursor
    messageContent.value = messageContent.value.substring(0, start) + indentation + messageContent.value.substring(start);

    nextTick(() => {
      const newCursorPos = start + indentation.length;
      textarea.setSelectionRange(newCursorPos, newCursorPos);
      textarea.focus();
    });
  } else {
    // Has selection: indent all selected lines
    const selectedText = messageContent.value.substring(start, end);
    const lines = selectedText.split('\n');
    const indentedLines = lines.map(line => indentation + line);
    const indentedText = indentedLines.join('\n');

    messageContent.value = messageContent.value.substring(0, start) + indentedText + messageContent.value.substring(end);

    nextTick(() => {
      // Select the indented text
      const newEnd = start + indentedText.length;
      textarea.setSelectionRange(start, newEnd);
      textarea.focus();
    });
  }
};

// Remove code indentation for Shift+Tab in code mode
const removeCodeIndentation = () => {
  const textarea = messageInput.value;
  if (!textarea) return;

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const indentation = '    '; // 4 spaces to remove

  if (start === end) {
    // No selection: remove indentation at cursor position
    const beforeCursor = messageContent.value.substring(0, start);
    const lineStart = beforeCursor.lastIndexOf('\n') + 1;
    const currentLine = beforeCursor.substring(lineStart);

    if (currentLine.startsWith(indentation)) {
      const newStart = lineStart;
      const newEnd = lineStart + indentation.length;
      messageContent.value = messageContent.value.substring(0, newStart) +
        messageContent.value.substring(newEnd);

      nextTick(() => {
        const newCursorPos = start - indentation.length;
        textarea.setSelectionRange(newCursorPos, newCursorPos);
        textarea.focus();
      });
    } else if (currentLine.startsWith('  ')) {
      // If less than 4 spaces, remove 2 spaces
      const newStart = lineStart;
      const newEnd = lineStart + 2;
      messageContent.value = messageContent.value.substring(0, newStart) +
        messageContent.value.substring(newEnd);

      nextTick(() => {
        const newCursorPos = start - 2;
        textarea.setSelectionRange(newCursorPos, newCursorPos);
        textarea.focus();
      });
    }
  } else {
    // Has selection: remove indentation from all selected lines
    const selectedText = messageContent.value.substring(start, end);
    const lines = selectedText.split('\n');
    const unindentedLines = lines.map(line => {
      if (line.startsWith(indentation)) {
        return line.substring(indentation.length);
      } else if (line.startsWith('  ')) {
        return line.substring(2);
      }
      return line;
    });
    const unindentedText = unindentedLines.join('\n');

    messageContent.value = messageContent.value.substring(0, start) + unindentedText + messageContent.value.substring(end);

    nextTick(() => {
      // Select the unindented text
      const newEnd = start + unindentedText.length;
      textarea.setSelectionRange(start, newEnd);
      textarea.focus();
    });
  }
};

const insertCodeBlock = () => {
  const textarea = messageInput.value;
  if (!textarea) return;

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const selectedText = messageContent.value.substring(start, end);

  const codeBlock = selectedText
    ? `\`\`\`\n${selectedText}\n\`\`\``
    : `\`\`\`\n\n\`\`\``;

  messageContent.value = messageContent.value.substring(0, start) + codeBlock + messageContent.value.substring(end);

  nextTick(() => {
    const newCursorPos = selectedText ? start + codeBlock.length : start + 4;
    textarea.setSelectionRange(newCursorPos, newCursorPos);
    textarea.focus();
  });
};

const insertTable = () => {
  const textarea = messageInput.value;
  if (!textarea) return;

  const start = textarea.selectionStart;
  const table = `| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |

`;

  messageContent.value = messageContent.value.substring(0, start) + table + messageContent.value.substring(start);

  nextTick(() => {
    const newCursorPos = start + table.length;
    textarea.setSelectionRange(newCursorPos, newCursorPos);
    textarea.focus();
  });
};

const clearContent = () => {
  messageContent.value = '';
  files.value = [];
  uploadedFileUrl.value = ''; // 🎯 清空上传的文件URL
  uploadedFileInfo.value = null; // 🎯 清空上传的文件信息
  showPreview.value = false;
  emit('preview-state-change', false);
  nextTick(() => {
    if (messageInput.value) {
      messageInput.value.focus();
      messageInput.value.style.height = 'auto';
    }
  });
};

// File handling
const triggerFileUpload = () => {
  if (fileInput.value) {
    fileInput.value.click();
  }
};

const handleFileSelect = (event) => {
  const selectedFiles = Array.from(event.target.files);
  if (selectedFiles.length === 0) return;

  files.value.push(...selectedFiles);
  formatMode.value = 'file';
  showPreview.value = true;
  emit('preview-state-change', true);

  event.target.value = '';
};

const handlePaste = (event) => {
  const items = event.clipboardData?.items;
  if (!items) return;

  const pastedFiles = [];
  for (const item of items) {
    if (item.type.startsWith('image/')) {
      event.preventDefault();
      const file = item.getAsFile();
      if (file) {
        pastedFiles.push(file);
      }
    }
  }

  if (pastedFiles.length > 0) {
    files.value.push(...pastedFiles);
    formatMode.value = 'file';
    showPreview.value = true;
    emit('preview-state-change', true);
  }
};

const removeFile = (index) => {
  files.value.splice(index, 1);
  if (files.value.length === 0) {
    closePreview();
    formatMode.value = 'text';
  }
};

// 🎯 处理FilePreview组件的新事件
const handleFileUploaded = (uploadResult) => {
  console.log('✅ [MessageInput] File uploaded successfully:', uploadResult);

  // 🎯 保存上传成功的文件URL和信息
  if (uploadResult && (uploadResult.url || uploadResult.file_url)) {
    // 支持多种URL字段格式
    const fileUrl = uploadResult.url || uploadResult.file_url;

    // 🎯 强制触发响应式更新
    uploadedFileUrl.value = '';
    uploadedFileInfo.value = null;

    // 使用nextTick确保UI更新
    nextTick(() => {
      uploadedFileUrl.value = fileUrl;
      uploadedFileInfo.value = {
        url: fileUrl,
        filename: uploadResult.filename || uploadResult.file_name || files.value[0]?.name,
        size: uploadResult.size || uploadResult.file_size || files.value[0]?.size,
        mime_type: uploadResult.mime_type || uploadResult.mimetype || files.value[0]?.type,
        id: uploadResult.id
      };

      console.log('📁 [MessageInput] File URL set for sending:', uploadedFileUrl.value);

      // 🎯 强制触发canSend重新计算
      nextTick(() => {
        console.log('🔄 [MessageInput] canSend should now be:', canSend.value);
      });
    });
  } else {
    console.error('❌ [MessageInput] Invalid uploadResult - no URL found:', uploadResult);
  }
};

const handleFileUploadError = (error) => {
  console.error('❌ [MessageInput] File upload error:', error);
  // 可以显示错误消息给用户
};

const handleFileRemoved = () => {
  console.log('🗑️ [MessageInput] File removed - clearing all file-related state');

  // 清空所有文件相关状态
  files.value = []; // 清空文件数组
  uploadedFileUrl.value = ''; // 🎯 清空上传的文件URL
  uploadedFileInfo.value = null; // 🎯 清空上传的文件信息

  // 重置UI状态
  closePreview();
  formatMode.value = 'text';

  console.log('🧹 [MessageInput] All file state cleared - send button should reflect content-only state');
  console.log('📊 [MessageInput] Current state:', {
    hasContent: messageContent.value.trim().length > 0,
    hasFiles: files.value.length > 0,
    hasUploadedFile: uploadedFileUrl.value.trim().length > 0,
    canSend: canSend.value
  });
};

// 🎯 处理FilePreview触发的文件上传事件
const handleTriggerUpload = () => {
  console.log('📁 [MessageInput] Triggering new file upload');
  triggerFileUpload();
};

// Input handling
const handleKeyDown = (event) => {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault();
    sendMessage();
  }

  // Handle Tab key for code indentation in code mode
  if (event.key === 'Tab' && formatMode.value === 'code') {
    event.preventDefault();
    if (event.shiftKey) {
      removeCodeIndentation();
    } else {
      insertCodeIndentation();
    }
    return;
  }

  if ((event.metaKey || event.ctrlKey) && formatMode.value === 'markdown') {
    switch (event.key) {
      case 'b': event.preventDefault(); insertMarkdown('**', '**'); break;
      case 'i': event.preventDefault(); insertMarkdown('*', '*'); break;
      case 'k': event.preventDefault(); insertMarkdown('[', '](url)'); break;
    }
  }

  // Close emoji picker on Escape
  if (event.key === 'Escape' && showEmojiPicker.value) {
    showEmojiPicker.value = false;
  }
};

const handleInput = () => {
  nextTick(() => {
    if (messageInput.value) {
      messageInput.value.style.height = 'auto';
      messageInput.value.style.height = Math.min(messageInput.value.scrollHeight, 120) + 'px';
    }
  });
};

const sendMessage = async () => {
  if (!canSend.value) {
    console.log('⚠️ [MessageInput] Send blocked - requirements not met');
    return;
  }

  // 🚨 严格验证：如果有本地文件但没有远端URL，拒绝发送
  const hasLocalFiles = files.value.length > 0;
  const hasUploadedFileUrl = uploadedFileUrl.value.trim().length > 0;
  
  if (hasLocalFiles && !hasUploadedFileUrl) {
    console.error('❌ [MessageInput] Cannot send: Files selected but not uploaded to remote');
    // 可以显示用户提示
    alert('Please upload files to remote server first before sending');
    return;
  }

  isSending.value = true;

  try {
    // 🎯 构建消息数据
    let content = messageContent.value.trim();

    // 🚀 Code模式下自动包装为代码块
    if (formatMode.value === 'code' && content) {
      const language = selectedLanguage.value || 'plaintext';
      content = `\`\`\`${language}\n${content}\n\`\`\``;
      console.log(`🔧 [MessageInput] Code mode: wrapping content as ${language} code block`);
    }

    const messageData = {
      content: content,
      formatMode: formatMode.value,
      reply_to: props.replyToMessage?.id
    };

    // 🎯 只发送已上传到远端的文件URL
    if (hasUploadedFileUrl) {
      messageData.files = [uploadedFileInfo.value];
      console.log('📨 [MessageInput] Sending message with remote file URL:', uploadedFileUrl.value);
    } else {
      console.log('📨 [MessageInput] Sending text-only message');
    }

    emit('message-sent', messageData);

    // 🎯 清空所有状态
    messageContent.value = '';
    files.value = [];
    uploadedFileUrl.value = '';
    uploadedFileInfo.value = null;
    closePreview();
    formatMode.value = 'text';

    nextTick(() => {
      if (messageInput.value) {
        messageInput.value.style.height = 'auto';
      }
    });

  } catch (error) {
    console.error('❌ [MessageInput] Failed to send message:', error);
  } finally {
    isSending.value = false;
  }
};

// Chat isolation
watch(() => props.chatId, (newChatId, oldChatId) => {
  if (newChatId !== oldChatId && oldChatId !== undefined) {
    messageContent.value = '';
    files.value = [];
    uploadedFileUrl.value = '';
    uploadedFileInfo.value = null;
    showPreview.value = false;
    formatMode.value = 'text';
    showEmojiPicker.value = false; // 🎯 Close emoji picker on chat switch
    emit('preview-state-change', false);
  }
});

// 🎯 Emoji handling functions
const toggleEmojiPicker = () => {
  showEmojiPicker.value = !showEmojiPicker.value;
  console.log('🎭 Toggle emoji picker:', showEmojiPicker.value);
};

const handleEmojiOverlayClick = (event) => {
  if (event.target.classList.contains('emoji-picker-overlay')) {
    showEmojiPicker.value = false;
    console.log('🎭 Emoji picker closed by overlay click');
  }
};

const handleEmojiSelect = (emojiObject) => {
  const textarea = messageInput.value;
  if (!textarea) {
    console.warn('⚠️ Textarea not found for emoji insertion');
    return;
  }

  const emoji = emojiObject.emoji || emojiObject;
  const start = textarea.selectionStart || 0;
  const end = textarea.selectionEnd || 0;

  // Insert emoji at cursor position
  messageContent.value = messageContent.value.substring(0, start) + emoji + messageContent.value.substring(end);

  // Move cursor after emoji
  nextTick(() => {
    const newCursorPos = start + emoji.length;
    textarea.setSelectionRange(newCursorPos, newCursorPos);
    textarea.focus();
  });

  // 🎯 添加到最近使用
  addToRecentEmojis(emojiObject);

  // Close emoji picker after selection
  showEmojiPicker.value = false;
  console.log('🎭 Emoji inserted:', emoji);
};

// 🎯 添加到最近使用的表情
const addToRecentEmojis = (emojiObject) => {
  // 移除已存在的相同表情
  const filtered = recentEmojis.value.filter(item => item.emoji !== emojiObject.emoji);

  // 添加到开头
  recentEmojis.value = [emojiObject, ...filtered];

  // 限制最近使用的数量
  if (recentEmojis.value.length > 16) {
    recentEmojis.value = recentEmojis.value.slice(0, 16);
  }

  // 保存到localStorage
  localStorage.setItem('fechatter_recent_emojis', JSON.stringify(recentEmojis.value));
};

// Click outside handler for emoji picker
const handleClickOutside = (event) => {
  if (showEmojiPicker.value && emojiPickerRef.value && !emojiPickerRef.value.contains(event.target)) {
    const emojiButton = document.querySelector('.emoji-btn');
    if (emojiButton && !emojiButton.contains(event.target)) {
      showEmojiPicker.value = false;
    }
  }
};

onMounted(() => {
  document.addEventListener('click', handleClickOutside);

  // 🎯 加载最近使用的表情
  try {
    const saved = localStorage.getItem('fechatter_recent_emojis');
    if (saved) {
      recentEmojis.value = JSON.parse(saved);
    }
  } catch (error) {
    console.warn('Failed to load recent emojis:', error);
  }

  console.log('🎭 MessageInput with Enterprise Emoji mounted');
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<style scoped src="./styles.css"></style>
