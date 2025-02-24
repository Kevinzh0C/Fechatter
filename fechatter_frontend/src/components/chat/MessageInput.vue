<template>
  <div class="message-input" :class="{
    'has-preview': showPreview,
    'format-active': formatMode !== 'text',
    'has-floating-toolbar': showFloatingToolbar
  }">
    <!-- Unified Upward Preview Container -->
    <Transition name="preview-popup" appear>
      <div v-if="showPreview" class="unified-preview-container" :class="`preview-${formatMode}`">
        <div class="preview-header">
          <div class="preview-title">
            <svg v-if="formatMode === 'markdown'" class="w-4 h-4" viewBox="0 0 16 16" fill="currentColor">
              <path
                d="M14.85 3H1.15C.52 3 0 3.52 0 4.15v7.69C0 12.48.52 13 1.15 13h13.69c.64 0 1.15-.52 1.15-1.15v-7.7C16 3.52 15.48 3 14.85 3z" />
            </svg>
            <svg v-else-if="formatMode === 'code'" class="w-4 h-4" viewBox="0 0 16 16" fill="currentColor">
              <path
                d="M4.72 3.22a.75.75 0 011.06 1.06L2.06 8l3.72 3.72a.75.75 0 11-1.06 1.06L.47 8.53a.75.75 0 010-1.06l4.25-4.25z" />
            </svg>
            <svg v-else class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd"
                d="M15.621 4.379a3 3 0 00-4.242 0l-7 7a3 3 0 004.241 4.243h.001l.497-.5a.75.75 0 011.064 1.057l-.498.501-.002.002a4.5 4.5 0 01-6.364-6.364l7-7a4.5 4.5 0 016.368 6.36l-3.455 3.553A2.625 2.625 0 119.52 9.52l3.45-3.451a.75.75 0 111.061 1.06l-3.45 3.451a1.125 1.125 0 001.587 1.595l3.454-3.553a3 3 0 000-4.242z"
                clip-rule="evenodd" />
            </svg>
            <span>{{ previewTitles[formatMode] }}</span>
          </div>
          <button @click="closePreview" class="preview-close">×</button>
        </div>

        <div class="preview-content" ref="previewContentRef">
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
              </select>
            </div>
            <pre><code :class="`language-${selectedLanguage}`">{{ messageContent }}</code></pre>
          </div>

          <!-- File Preview -->
          <div v-else-if="formatMode === 'file'" class="file-preview">
            <!-- 🔧 FILE SIZE CHECKER: Add 2MB validation with user interaction -->
            <FileSizeChecker :files="files" :auto-show="true" @files-validated="handleFilesValidated"
              @files-compressed="handleFilesCompressed" @invalid-files-removed="handleInvalidFilesRemoved" />

            <div v-for="(file, index) in files" :key="index" class="file-item">
              <img v-if="file.type.startsWith('image/')" :src="getFilePreviewUrl(file)" :alt="file.name"
                class="file-thumbnail">
              <div v-else class="file-icon">📄</div>
              <div class="file-info">
                <span class="file-name">{{ file.name }}</span>
                <span class="file-size">{{ formatFileSize(file.size) }}</span>
              </div>
              <button @click="removeFile(index)" class="file-remove">×</button>
            </div>
            <button @click="triggerFileUpload" class="add-files-btn">+ Add more files</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Smart Floating Toolbar -->
    <Transition name="toolbar-float">
      <div v-if="showFloatingToolbar" ref="floatingToolbar" class="floating-toolbar" :style="floatingToolbarStyle">
        <button @click="applyFormat('bold')" class="floating-tool-btn" title="Bold (⌘B)">
          <strong>B</strong>
        </button>
        <button @click="applyFormat('italic')" class="floating-tool-btn" title="Italic (⌘I)">
          <em>I</em>
        </button>
        <button @click="applyFormat('code')" class="floating-tool-btn" title="Code">
          <code>{}</code>
        </button>
        <div class="floating-toolbar-divider"></div>
        <button @click="applyFormat('link')" class="floating-tool-btn" title="Link (⌘K)">
          🔗
        </button>
      </div>
    </Transition>

    <!-- Reply Context -->
    <div v-if="replyToMessage" class="reply-context">
      <div class="reply-indicator">
        <span>↩️ Replying to {{ replyToMessage.user?.fullname || 'Unknown' }}</span>
      </div>
      <div class="reply-content">{{ truncateText(replyToMessage.content, 100) }}</div>
      <button @click="cancelReply" class="reply-cancel">×</button>
    </div>

    <!-- Mode Label -->
    <div v-if="formatMode !== 'text'" class="mode-label" :class="`mode-${formatMode}`">
      <div class="mode-indicator">
        <span v-if="formatMode === 'markdown'" class="mode-icon">📝</span>
        <span v-if="formatMode === 'code'" class="mode-icon">💻</span>
        <span class="mode-text">{{ formatMode === 'markdown' ? 'Markdown Mode' : 'Code Mode' }}</span>
      </div>
    </div>

    <!-- Main Input Area -->
    <div class="main-input-area">
      <!-- File/Media Button -->
      <button @click="toggleFileMode" class="action-btn" :class="{ active: formatMode === 'file' || files.length > 0 }"
        title="Attach files">
        📎
        <span v-if="files.length > 0" class="file-count-badge">{{ files.length }}</span>
      </button>

      <!-- Input Container -->
      <div class="input-container">
        <textarea ref="messageInput" v-model="messageContent" @keydown="handleKeyDown" @input="handleInput"
          @paste="handlePaste" @select="handleTextSelection" @mouseup="handleTextSelection" @blur="handleBlur"
          :placeholder="placeholderText" class="message-textarea" rows="1"></textarea>

        <!-- Character Counter -->
        <div v-if="showCharacterCount" class="character-counter" :class="{ warning: isNearLimit }">
          {{ messageContent.length }}{{ maxLength ? `/${maxLength}` : '' }}
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="action-buttons">
        <!-- Unified Format Mode Button (State Machine) -->
        <button @click="cycleFormatMode" class="unified-mode-btn" :class="{
          active: formatMode !== 'text',
          'mode-markdown': formatMode === 'markdown',
          'mode-code': formatMode === 'code'
        }" :title="getFormatModeTooltip()">
          <span v-if="formatMode === 'text'" class="mode-icon">A</span>
          <span v-else-if="formatMode === 'markdown'" class="mode-icon">MD</span>
          <span v-else-if="formatMode === 'code'" class="mode-icon">{ }</span>
        </button>

        <!-- 🚀 NEW: Floating Formatting Toolbar Button -->
        <button @click="toggleFloatingFormattingToolbar" class="action-btn floating-toolbar-btn"
          :class="{ active: showFloatingFormattingToolbar }" title="Open Formatting Toolbar (⌘⇧F)">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z" />
            <path d="M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z" />
          </svg>
        </button>

        <!-- Emoji Button -->
        <button @click="toggleEmojiPicker" class="action-btn" title="Add emoji">
          😊
        </button>

        <!-- Send Button -->
        <button @click="sendMessage" :disabled="!canSend" class="send-btn" :class="{ active: canSend }">
          <svg v-if="!isSending" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor"
            stroke-width="2">
            <line x1="22" y1="2" x2="11" y2="13"></line>
            <polygon points="22,2 15,22 11,13 2,9"></polygon>
          </svg>
          <div v-else class="loading-spinner"></div>
        </button>
      </div>
    </div>

    <!-- Production-Grade Emoji Picker -->
    <Transition name="emoji-fade">
      <div v-if="showEmojiPicker" class="emoji-picker" ref="emojiPickerRef">
        <!-- Header with Search -->
        <div class="emoji-header">
          <div class="emoji-search-container">
            <input v-model="emojiSearchQuery" type="text" placeholder="Search emojis..." class="emoji-search"
              @input="selectEmojiCategory('smileys')" />
            <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor"
              stroke-width="2">
              <circle cx="11" cy="11" r="8"></circle>
              <path d="m21 21-4.35-4.35"></path>
            </svg>
          </div>
          <button @click="showEmojiPicker = false" class="emoji-close">×</button>
        </div>

        <!-- Category Tabs -->
        <div class="emoji-categories" v-if="!emojiSearchQuery">
          <!-- Recent Emojis Tab -->
          <button v-if="recentEmojis.length > 0" @click="selectEmojiCategory('recent')" class="category-tab"
            :class="{ active: selectedEmojiCategory === 'recent' }" title="Recently Used">
            🕒
          </button>

          <!-- Category Tabs -->
          <button v-for="(category, key) in emojiCategories" :key="key" @click="selectEmojiCategory(key)"
            class="category-tab" :class="{ active: selectedEmojiCategory === key }" :title="category.name">
            {{ category.icon }}
          </button>
        </div>

        <!-- Emoji Content -->
        <div class="emoji-content">
          <!-- Recent Emojis Section -->
          <div v-if="selectedEmojiCategory === 'recent' && recentEmojis.length > 0" class="emoji-section">
            <div class="section-title">Recently Used</div>
            <div class="emoji-grid">
              <button v-for="emoji in recentEmojis" :key="emoji.char" @click="insertEmoji(emoji)" class="emoji-item"
                :title="`${emoji.name} (${emoji.char})`">
                {{ emoji.char }}
              </button>
            </div>
          </div>

          <!-- Search Results -->
          <div v-else-if="emojiSearchQuery" class="emoji-section">
            <div class="section-title">Search Results ({{ filteredEmojis.length }})</div>
            <div class="emoji-grid" v-if="filteredEmojis.length > 0">
              <button v-for="emoji in filteredEmojis.slice(0, 48)" :key="emoji.char" @click="insertEmoji(emoji)"
                class="emoji-item" :title="`${emoji.name} (${emoji.char})`">
                {{ emoji.char }}
              </button>
            </div>
            <div v-else class="no-results">
              <div class="no-results-icon">🔍</div>
              <div class="no-results-text">No emojis found</div>
              <div class="no-results-hint">Try a different search term</div>
            </div>
          </div>

          <!-- Category Content -->
          <div v-else class="emoji-section">
            <div class="section-title">{{ emojiCategories[selectedEmojiCategory]?.name }}</div>
            <div class="emoji-grid">
              <button v-for="emoji in emojiCategories[selectedEmojiCategory]?.emojis" :key="emoji.char"
                @click="insertEmoji(emoji)" class="emoji-item" :title="`${emoji.name} (${emoji.char})`">
                {{ emoji.char }}
              </button>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="emoji-footer">
          <div class="emoji-count">{{Object.values(emojiCategories).reduce((acc, cat) => acc + cat.emojis.length, 0)}}
            emojis available</div>
        </div>
      </div>
    </Transition>

    <!-- Hidden file input -->
    <input ref="fileInput" type="file" multiple @change="handleFileSelect" style="display: none">

    <!-- 🚀 NEW: Floating Formatting Toolbar -->
    <FloatingFormattingToolbar :visible="showFloatingFormattingToolbar" :position="floatingFormattingPosition"
      :textareaRef="messageInput" :currentContent="messageContent" :disabled="props.disabled"
      @close="handleFloatingFormattingClose" @format-applied="handleFormatApplied"
      @content-changed="handleContentChanged" @position-changed="handlePositionChanged" />
  </div>
</template>

<script setup>
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue';
import { marked } from 'marked';
import DOMPurify from 'dompurify';
import FileSizeChecker from '@/components/ui/FileSizeChecker.vue';
import FloatingFormattingToolbar from './FloatingFormattingToolbar.vue';
// TODO: 临时注释掉新组件导入，待修复后恢复
// import EnhancedFormattingToolbar from './EnhancedFormattingToolbar.vue';
// import LiveMarkdownPreview from './LiveMarkdownPreview.vue';
// import { unifiedMarkdownEngine } from '@/services/UnifiedMarkdownEngine.js';

const props = defineProps({
  chatId: { type: [Number, String], required: true },
  replyToMessage: { type: Object, default: null },
  disabled: { type: Boolean, default: false },
  maxLength: { type: Number, default: 2000 }
});

const emit = defineEmits(['message-sent', 'reply-cancelled', 'preview-state-change']);

// State
const messageContent = ref('');
const files = ref([]);
const showPreview = ref(false);
const formatMode = ref('text'); // 'text', 'markdown', 'code', 'file'
const selectedLanguage = ref('javascript');
const isSending = ref(false);
const showEmojiPicker = ref(false);

// Production-grade emoji picker state
const selectedEmojiCategory = ref('smileys');
const emojiSearchQuery = ref('');
const recentEmojis = ref([]);
const maxRecentEmojis = 16;

// Floating toolbar state (original small toolbar)
const showFloatingToolbar = ref(false);
const floatingToolbarStyle = ref({});

// 🚀 NEW: Floating Formatting Toolbar State
const showFloatingFormattingToolbar = ref(false);
const floatingFormattingPosition = ref({ x: 100, y: 100 });

// Refs
const messageInput = ref(null);
const fileInput = ref(null);
const emojiPickerRef = ref(null);
const floatingToolbar = ref(null);
const previewContentRef = ref(null);

// Constants
const previewTitles = {
  markdown: 'Markdown Preview',
  code: 'Code Preview',
  file: 'File Preview'
};

// Production-grade emoji data organized by categories
const emojiCategories = {
  smileys: {
    name: 'Smileys & People',
    icon: '😀',
    emojis: [
      { char: '😀', name: 'grinning', keywords: ['happy', 'smile'] },
      { char: '😃', name: 'smiley', keywords: ['happy', 'joy'] },
      { char: '😄', name: 'smile', keywords: ['happy', 'laugh'] },
      { char: '😁', name: 'grin', keywords: ['happy', 'teeth'] },
      { char: '😅', name: 'sweat_smile', keywords: ['happy', 'sweat'] },
      { char: '🤣', name: 'rofl', keywords: ['laugh', 'rolling'] },
      { char: '😂', name: 'joy', keywords: ['laugh', 'tears'] },
      { char: '🙂', name: 'slightly_smiling', keywords: ['smile', 'slight'] },
      { char: '😉', name: 'wink', keywords: ['flirt', 'wink'] },
      { char: '😊', name: 'blush', keywords: ['happy', 'blush'] },
      { char: '🥰', name: 'smiling_face_with_hearts', keywords: ['love', 'hearts'] },
      { char: '😍', name: 'heart_eyes', keywords: ['love', 'heart'] },
      { char: '🤩', name: 'star_struck', keywords: ['star', 'eyes'] },
      { char: '😘', name: 'kissing_heart', keywords: ['kiss', 'love'] },
      { char: '😋', name: 'yum', keywords: ['tongue', 'tasty'] },
      { char: '😛', name: 'stuck_out_tongue', keywords: ['tongue', 'playful'] },
      { char: '🤪', name: 'zany_face', keywords: ['crazy', 'wild'] },
      { char: '🤔', name: 'thinking', keywords: ['think', 'hmm'] },
      { char: '🤗', name: 'hugs', keywords: ['hug', 'embrace'] },
      { char: '🤭', name: 'hand_over_mouth', keywords: ['oops', 'quiet'] },
      { char: '🤫', name: 'shushing_face', keywords: ['quiet', 'shh'] },
      { char: '😴', name: 'sleeping', keywords: ['sleep', 'tired'] },
      { char: '🥱', name: 'yawning', keywords: ['tired', 'yawn'] },
      { char: '😷', name: 'mask', keywords: ['sick', 'mask'] }
    ]
  },
  gestures: {
    name: 'Gestures',
    icon: '👍',
    emojis: [
      { char: '👍', name: 'thumbs_up', keywords: ['good', 'like', 'yes'] },
      { char: '👎', name: 'thumbs_down', keywords: ['bad', 'dislike', 'no'] },
      { char: '👏', name: 'clap', keywords: ['applause', 'clap'] },
      { char: '🙌', name: 'raised_hands', keywords: ['celebrate', 'hooray'] },
      { char: '👐', name: 'open_hands', keywords: ['open', 'hands'] },
      { char: '🤝', name: 'handshake', keywords: ['deal', 'agreement'] },
      { char: '🙏', name: 'pray', keywords: ['please', 'thanks'] },
      { char: '✋', name: 'raised_hand', keywords: ['stop', 'high_five'] },
      { char: '🤚', name: 'raised_back_of_hand', keywords: ['stop', 'back'] },
      { char: '👋', name: 'wave', keywords: ['hello', 'goodbye'] },
      { char: '🤟', name: 'love_you_gesture', keywords: ['love', 'rock'] },
      { char: '✌️', name: 'victory_hand', keywords: ['peace', 'victory'] },
      { char: '🤞', name: 'crossed_fingers', keywords: ['luck', 'hope'] },
      { char: '🤘', name: 'sign_of_the_horns', keywords: ['rock', 'metal'] },
      { char: '🤙', name: 'call_me_hand', keywords: ['call', 'hang_loose'] },
      { char: '👌', name: 'ok_hand', keywords: ['ok', 'perfect'] }
    ]
  },
  hearts: {
    name: 'Hearts',
    icon: '❤️',
    emojis: [
      { char: '❤️', name: 'red_heart', keywords: ['love', 'heart'] },
      { char: '🧡', name: 'orange_heart', keywords: ['love', 'orange'] },
      { char: '💛', name: 'yellow_heart', keywords: ['love', 'yellow'] },
      { char: '💚', name: 'green_heart', keywords: ['love', 'green'] },
      { char: '💙', name: 'blue_heart', keywords: ['love', 'blue'] },
      { char: '💜', name: 'purple_heart', keywords: ['love', 'purple'] },
      { char: '🖤', name: 'black_heart', keywords: ['love', 'black'] },
      { char: '🤍', name: 'white_heart', keywords: ['love', 'white'] },
      { char: '🤎', name: 'brown_heart', keywords: ['love', 'brown'] },
      { char: '💔', name: 'broken_heart', keywords: ['sad', 'break'] },
      { char: '💕', name: 'two_hearts', keywords: ['love', 'hearts'] },
      { char: '💖', name: 'sparkling_heart', keywords: ['love', 'sparkle'] },
      { char: '💗', name: 'growing_heart', keywords: ['love', 'growing'] },
      { char: '💘', name: 'heart_with_arrow', keywords: ['love', 'cupid'] },
      { char: '💝', name: 'heart_with_ribbon', keywords: ['love', 'gift'] }
    ]
  },
  celebration: {
    name: 'Celebration',
    icon: '🎉',
    emojis: [
      { char: '🎉', name: 'party_popper', keywords: ['party', 'celebrate'] },
      { char: '🎊', name: 'confetti_ball', keywords: ['party', 'confetti'] },
      { char: '🥳', name: 'partying_face', keywords: ['party', 'hat'] },
      { char: '🎈', name: 'balloon', keywords: ['party', 'balloon'] },
      { char: '🎂', name: 'birthday_cake', keywords: ['birthday', 'cake'] },
      { char: '🍰', name: 'shortcake', keywords: ['cake', 'dessert'] },
      { char: '🎁', name: 'gift', keywords: ['present', 'gift'] },
      { char: '🏆', name: 'trophy', keywords: ['win', 'award'] },
      { char: '🥇', name: 'first_place_medal', keywords: ['gold', 'first'] },
      { char: '🥈', name: 'second_place_medal', keywords: ['silver', 'second'] },
      { char: '🥉', name: 'third_place_medal', keywords: ['bronze', 'third'] },
      { char: '⭐', name: 'star', keywords: ['star', 'awesome'] },
      { char: '🌟', name: 'glowing_star', keywords: ['star', 'shine'] },
      { char: '✨', name: 'sparkles', keywords: ['sparkle', 'magic'] },
      { char: '🔥', name: 'fire', keywords: ['hot', 'fire'] },
      { char: '💯', name: 'hundred_points', keywords: ['perfect', '100'] }
    ]
  }
};

// Frequently used emojis (user customizable)
// (variables already defined above)

// Computed
const canSend = computed(() => (messageContent.value.trim() || files.value.length > 0) && !isSending.value);

const showCharacterCount = computed(() =>
  messageContent.value.length > 0 || (props.maxLength && messageContent.value.length > props.maxLength * 0.8)
);

const isNearLimit = computed(() =>
  props.maxLength && messageContent.value.length > props.maxLength * 0.9
);

const placeholderText = computed(() => {
  if (props.replyToMessage) {
    return `Replying to ${props.replyToMessage.user?.fullname || 'Unknown'}...`;
  }

  switch (formatMode.value) {
    case 'markdown': return 'Type in Markdown... **bold**, *italic*, `code`';
    case 'code': return `Type ${selectedLanguage.value} code...`;
    case 'file': return 'Add files and type a message...';
    default: return 'Type a message...';
  }
});

const renderedMarkdown = computed(() => {
  if (!messageContent.value.trim()) {
    return '<p class="empty-preview">Start typing to see preview...</p>';
  }
  try {
    return DOMPurify.sanitize(marked(messageContent.value, { breaks: true, gfm: true }));
  } catch (error) {
    return '<p class="error-preview">Markdown syntax error</p>';
  }
});

// Methods - Format Mode State Machine
const cycleFormatMode = () => {
  const modes = ['text', 'markdown', 'code'];
  const currentIndex = modes.indexOf(formatMode.value);
  const nextIndex = (currentIndex + 1) % modes.length;
  const nextMode = modes[nextIndex];

  if (nextMode === 'text') {
    closePreview();
  } else {
    formatMode.value = nextMode;
    showPreview.value = true;
    emit('preview-state-change', true);
  }
};

const getFormatModeTooltip = () => {
  switch (formatMode.value) {
    case 'text': return 'Switch to Markdown mode';
    case 'markdown': return 'Switch to Code mode';
    case 'code': return 'Switch to Text mode';
    default: return 'Toggle format mode';
  }
};

const toggleFileMode = () => {
  if (files.value.length > 0) {
    if (formatMode.value === 'file' && showPreview.value) {
      closePreview();
    } else {
      formatMode.value = 'file';
      showPreview.value = true;
      emit('preview-state-change', true);
    }
  } else {
    fileInput.value?.click();
  }
};

const closePreview = () => {
  showPreview.value = false;
  formatMode.value = 'text';
  emit('preview-state-change', false);
};

const handleTextSelection = () => {
  if (formatMode.value !== 'markdown') return;

  const textarea = messageInput.value;
  if (!textarea) return;

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;

  if (start === end) {
    showFloatingToolbar.value = false;
    return;
  }

  nextTick(() => {
    const rect = textarea.getBoundingClientRect();
    const textHeight = 20;
    const lineNumber = Math.floor(start / 50); // Approximate

    floatingToolbarStyle.value = {
      position: 'fixed',
      left: `${rect.left + 20}px`,
      top: `${rect.top - 50 + lineNumber * textHeight}px`,
      zIndex: 1001
    };

    showFloatingToolbar.value = true;
  });
};

const handleBlur = () => {
  setTimeout(() => {
    if (!floatingToolbar.value?.contains(document.activeElement)) {
      showFloatingToolbar.value = false;
    }
  }, 200);
};

const applyFormat = (format) => {
  let before = '', after = '';

  switch (format) {
    case 'bold': before = '**'; after = '**'; break;
    case 'italic': before = '*'; after = '*'; break;
    case 'code': before = '`'; after = '`'; break;
    case 'link': before = '['; after = '](url)'; break;
  }

  insertMarkdown(before, after);
  showFloatingToolbar.value = false;
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

const handleKeyDown = (event) => {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault();
    sendMessage();
    return;
  }

  // Format shortcuts
  if ((event.metaKey || event.ctrlKey) && formatMode.value === 'markdown') {
    switch (event.key) {
      case 'b': event.preventDefault(); insertMarkdown('**', '**'); break;
      case 'i': event.preventDefault(); insertMarkdown('*', '*'); break;
      case 'k': event.preventDefault(); insertMarkdown('[', '](url)'); break;
    }
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

  if (pastedFiles.length === 0) return;

  // 🔧 FILE SIZE CHECKER: Validate pasted files
  const validFiles = [];
  const invalidFiles = [];

  for (const file of pastedFiles) {
    const validation = validateFileSize(file);
    if (validation.isValid) {
      validFiles.push(file);
    } else {
      invalidFiles.push({ file, errors: validation.errors });
    }
  }

  // Add valid files
  if (validFiles.length > 0) {
    files.value.push(...validFiles);
    if (!showPreview.value) {
      formatMode.value = 'file';
      showPreview.value = true;
      emit('preview-state-change', true);
    }
  }

  // Show notification for invalid files
  if (invalidFiles.length > 0) {
    const errorMsg = `${invalidFiles.length} pasted file(s) exceed 2MB limit`;
    showFileNotification(errorMsg, 'error');
  }

  // Show success message for valid files
  if (validFiles.length > 0) {
    const successMsg = `Added ${validFiles.length} pasted file(s)`;
    showFileNotification(successMsg, 'success');
  }
};

const triggerFileUpload = () => fileInput.value?.click();

const handleFileSelect = async (event) => {
  const selectedFiles = Array.from(event.target.files);

  if (selectedFiles.length === 0) return;

  // Validate files before adding
  const { useFileUploadStore } = await import('@/stores/fileUploadStore');
  const fileUploadStore = useFileUploadStore();

  const validFiles = [];
  const invalidFiles = [];

  for (const file of selectedFiles) {
    const validation = validateFileSize(file);
    if (validation.isValid) {
      validFiles.push(file);
    } else {
      invalidFiles.push({ file, errors: validation.errors });
    }
  }

  // Add valid files
  if (validFiles.length > 0) {
    files.value.push(...validFiles);
    formatMode.value = 'file';
    showPreview.value = true;
    emit('preview-state-change', true);
  }

  // Show notification for invalid files
  if (invalidFiles.length > 0) {
    const errorMsg = `${invalidFiles.length} file(s) exceed 2MB limit: ${invalidFiles.map(f => f.file.name).join(', ')}`;
    showFileNotification(errorMsg, 'error');
  }

  event.target.value = '';
};

const validateFileSize = (file) => {
  const maxSize = 2 * 1024 * 1024; // 2MB
  const errors = [];

  if (file.size > maxSize) {
    const fileSize = formatFileSize(file.size);
    const maxSizeFormatted = formatFileSize(maxSize);
    errors.push(`File "${file.name}" (${fileSize}) exceeds 2MB limit (${maxSizeFormatted})`);
  }

  if (file.size === 0) {
    errors.push(`File "${file.name}" is empty`);
  }

  return {
    isValid: errors.length === 0,
    errors
  };
};

const showFileNotification = (message, type = 'info') => {
  // Simple notification implementation - you can enhance this with a proper toast system
  const notificationStyle = {
    error: 'background: #fee; color: #c53030; border: 1px solid #fed7d7;',
    success: 'background: #f0fff4; color: #38a169; border: 1px solid #c6f6d5;',
    warning: 'background: #fffbeb; color: #d69e2e; border: 1px solid #feebc8;',
    info: 'background: #ebf8ff; color: #3182ce; border: 1px solid #bee3f8;'
  };

  const notification = document.createElement('div');
  notification.innerHTML = `
    <div style="
      position: fixed; 
      top: 20px; 
      right: 20px; 
      padding: 12px 16px; 
      border-radius: 8px; 
      font-size: 14px; 
      font-weight: 500;
      z-index: 10000;
      max-width: 400px;
      ${notificationStyle[type]}
      box-shadow: 0 4px 12px rgba(0,0,0,0.1);
    ">
      ${message}
    </div>
  `;

  document.body.appendChild(notification);

  // Auto remove after 5 seconds
  setTimeout(() => {
    if (document.body.contains(notification)) {
      document.body.removeChild(notification);
    }
  }, 5000);

  // Click to dismiss
  notification.addEventListener('click', () => {
    if (document.body.contains(notification)) {
      document.body.removeChild(notification);
    }
  });
};

const removeFile = (index) => {
  files.value.splice(index, 1);
  if (files.value.length === 0) {
    closePreview();
  }
};

const getFilePreviewUrl = (file) => URL.createObjectURL(file);
const formatFileSize = (bytes) => {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

// 🔧 FILE SIZE CHECKER: Event handlers for file validation and user interaction
const handleFilesValidated = (validationData) => {
  const { valid, invalid, total } = validationData;

  if (import.meta.env.DEV) {
    console.log(`📏 File validation: ${valid.length}/${total} valid, ${invalid.length} invalid`);
  }

  if (invalid.length > 0) {
    // Show user-friendly notification
    const errorMsg = `${invalid.length} file(s) exceed 2MB limit or are invalid. Please compress or remove them.`;

    // Create a temporary notification (you can implement a proper toast system)
    showFileNotification(errorMsg, 'error');
  }
};

const handleFilesCompressed = (compressionData) => {
  const { originalFiles, compressedFiles, compressionResults } = compressionData;

  // Replace original files with compressed ones
  files.value = compressedFiles;

  const successMsg = `Successfully compressed ${compressionResults.successful} image(s). Files are now ready for upload.`;
  showFileNotification(successMsg, 'success');

  if (import.meta.env.DEV) {
    console.log(`🗜️ Compression completed:`, compressionResults);
  }
};

const handleInvalidFilesRemoved = (removalData) => {
  const { validFiles, removedCount } = removalData;

  // Update files array with only valid files
  files.value = validFiles;

  const successMsg = `Removed ${removedCount} invalid file(s). ${validFiles.length} valid file(s) remaining.`;
  showFileNotification(successMsg, 'warning');

  if (import.meta.env.DEV) {
    console.log(`🗑️ Removed ${removedCount} invalid files`);
  }

  // Close preview if no files remain
  if (validFiles.length === 0) {
    closePreview();
  }
};

// Production-grade emoji picker methods
const toggleEmojiPicker = () => {
  showEmojiPicker.value = !showEmojiPicker.value;
  if (showEmojiPicker.value) {
    emojiSearchQuery.value = '';
  }
};

const selectEmojiCategory = (categoryKey) => {
  selectedEmojiCategory.value = categoryKey;
};

const insertEmoji = (emoji) => {
  const textarea = messageInput.value;
  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;

  messageContent.value = messageContent.value.substring(0, start) + emoji.char + messageContent.value.substring(end);

  // Add to recent emojis
  addToRecentEmojis(emoji);

  nextTick(() => {
    const newPos = start + emoji.char.length;
    textarea.setSelectionRange(newPos, newPos);
    textarea.focus();
  });

  showEmojiPicker.value = false;
};

const addToRecentEmojis = (emoji) => {
  // Remove if already exists
  const existingIndex = recentEmojis.value.findIndex(e => e.char === emoji.char);
  if (existingIndex !== -1) {
    recentEmojis.value.splice(existingIndex, 1);
  }

  // Add to beginning
  recentEmojis.value.unshift(emoji);

  // Keep only max number
  if (recentEmojis.value.length > maxRecentEmojis) {
    recentEmojis.value = recentEmojis.value.slice(0, maxRecentEmojis);
  }

  // Save to localStorage
  localStorage.setItem('fechatter-recent-emojis', JSON.stringify(recentEmojis.value));
};

const loadRecentEmojis = () => {
  try {
    const stored = localStorage.getItem('fechatter-recent-emojis');
    if (stored) {
      recentEmojis.value = JSON.parse(stored);
    }
  } catch (error) {
    console.warn('Failed to load recent emojis:', error);
  }
};

const filteredEmojis = computed(() => {
  if (!emojiSearchQuery.value.trim()) {
    return emojiCategories[selectedEmojiCategory.value]?.emojis || [];
  }

  const query = emojiSearchQuery.value.toLowerCase();
  const allEmojis = Object.values(emojiCategories).flatMap(cat => cat.emojis);

  return allEmojis.filter(emoji =>
    emoji.name.toLowerCase().includes(query) ||
    emoji.keywords.some(keyword => keyword.toLowerCase().includes(query)) ||
    emoji.char.includes(query)
  );
});

const sendMessage = async () => {
  if (!canSend.value) return;

  isSending.value = true;

  try {
    // 1. 先处理文件上传（如果有文件）
    let uploadedFileUrls = [];
    if (files.value.length > 0) {
      // 使用fileUploadStore处理文件
      const { useFileUploadStore } = await import('@/stores/fileUploadStore');
      const fileUploadStore = useFileUploadStore();

      // 添加文件到store
      await fileUploadStore.addFiles(files.value);

      // 上传所有文件 - 这里会返回正确的文件URL数组
      uploadedFileUrls = await fileUploadStore.uploadAll();

      if (uploadedFileUrls.length !== files.value.length) {
        throw new Error('Some files failed to upload. Please try again.');
      }

      if (import.meta.env.DEV) {
        console.log('✅ [MessageInput] Files uploaded successfully:', uploadedFileUrls);
      }
    }

    // 2. 构建符合后端API要求的消息数据
    const messageData = {
      content: messageContent.value.trim(),
      files: uploadedFileUrls, // 文件URL数组，格式: ["/files/xxx.jpg", "/files/yyy.pdf"]
      formatMode: formatMode.value,
      reply_to: props.replyToMessage?.id,
      mentions: [], // TODO: 从内容中提取@mentions
      idempotency_key: Date.now().toString() + '-' + Math.random().toString(36).substr(2, 9)
    };

    if (import.meta.env.DEV) {
      console.log('📤 [MessageInput] Sending message data:', messageData);
    }

    // 3. 发送消息事件给Chat.vue
    emit('message-sent', messageData);

    // 4. 立即重置状态（乐观更新 - 假设发送成功）
    messageContent.value = '';
    files.value = [];
    closePreview();

    // 5. 重置textarea高度
    nextTick(() => {
      if (messageInput.value) {
        messageInput.value.style.height = 'auto';
      }
    });

  } catch (error) {
    console.error('❌ [MessageInput] Failed to send message:', error);

    // 显示用户友好的错误信息
    const errorMessage = error.message || 'Failed to send message. Please try again.';
    if (typeof window !== 'undefined' && window.showNotification) {
      window.showNotification(errorMessage, 'error');
    } else {
      alert(errorMessage);
    }
  } finally {
    isSending.value = false;
  }
};

const cancelReply = () => emit('reply-cancelled');
const truncateText = (text, maxLength) => text?.length > maxLength ? text.substring(0, maxLength) + '...' : text || '';

// Click outside handlers
const handleClickOutside = (event) => {
  if (showEmojiPicker.value && emojiPickerRef.value && !emojiPickerRef.value.contains(event.target)) {
    showEmojiPicker.value = false;
  }
};

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
  loadRecentEmojis();
});
onUnmounted(() => document.removeEventListener('click', handleClickOutside));

// 🚀 NEW: Floating Formatting Toolbar Methods
const toggleFloatingFormattingToolbar = () => {
  showFloatingFormattingToolbar.value = !showFloatingFormattingToolbar.value;

  if (showFloatingFormattingToolbar.value) {
    // Smart positioning based on input area
    const inputRect = messageInput.value?.getBoundingClientRect();
    if (inputRect) {
      // Position the toolbar to the right of the input area, or center it if not enough space
      const x = Math.min(inputRect.right + 20, window.innerWidth - 450);
      const y = Math.max(inputRect.top - 50, 50);
      floatingFormattingPosition.value = { x, y };
    }
  }
};

const handleFloatingFormattingClose = () => {
  showFloatingFormattingToolbar.value = false;
};

const handleFormatApplied = ({ format, beforeText, afterText, cursorPos }) => {
  // Apply the format using existing logic
  insertMarkdown(beforeText, afterText);

  // Focus back to textarea and set cursor position if provided
  nextTick(() => {
    if (messageInput.value && cursorPos !== undefined) {
      messageInput.value.focus();
      messageInput.value.setSelectionRange(cursorPos, cursorPos);
    }
  });
};

const handleContentChanged = (newContent) => {
  messageContent.value = newContent;

  // Update textarea height after content change
  nextTick(() => {
    handleInput();
  });
};

const handlePositionChanged = (position) => {
  floatingFormattingPosition.value = position;
};
</script>

<style scoped>
.message-input {
  position: relative;
  background: #ffffff;
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  overflow: visible;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.message-input:focus-within {
  border-color: #6366f1;
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

.message-input.format-active {
  border-color: #8b5cf6;
}

/* Unified Preview Container */
.unified-preview-container {
  position: absolute;
  bottom: 100%;
  left: 0;
  right: 0;
  background: #f8fafc;
  border: 1px solid #e5e7eb;
  border-bottom: none;
  border-radius: 12px 12px 0 0;
  max-height: 300px;
  overflow: hidden;
  z-index: 100;
  box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.1);
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #e5e7eb;
  background: #ffffff;
}

.preview-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  color: #374151;
  font-size: 14px;
}

.preview-close {
  background: none;
  border: none;
  color: #6b7280;
  cursor: pointer;
  font-size: 18px;
  padding: 4px 8px;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.preview-close:hover {
  background: #f3f4f6;
}

.preview-content {
  padding: 16px;
  max-height: 240px;
  overflow-y: auto;
}

/* Preview Content Styles */
.markdown-preview {
  line-height: 1.6;
  font-size: 14px;
}

.markdown-preview h1,
.markdown-preview h2,
.markdown-preview h3 {
  margin: 16px 0 8px 0;
  color: #1f2937;
}

.markdown-preview p {
  margin: 8px 0;
}

.markdown-preview code {
  background: #f3f4f6;
  padding: 2px 4px;
  border-radius: 3px;
  font-size: 13px;
}

.markdown-preview pre {
  background: #1f2937;
  color: #e5e7eb;
  padding: 12px;
  border-radius: 6px;
  overflow-x: auto;
  margin: 12px 0;
}

.code-preview .code-header {
  margin-bottom: 12px;
}

.language-selector {
  background: #ffffff;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  padding: 4px 8px;
  font-size: 12px;
}

.code-preview pre {
  background: #1f2937;
  color: #e5e7eb;
  padding: 16px;
  border-radius: 8px;
  overflow-x: auto;
  font-size: 13px;
}

.file-preview {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px;
  background: #ffffff;
  border-radius: 8px;
  border: 1px solid #e5e7eb;
}

.file-thumbnail {
  width: 48px;
  height: 48px;
  object-fit: cover;
  border-radius: 6px;
}

.file-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  background: #f3f4f6;
  border-radius: 6px;
}

.file-info {
  flex: 1;
}

.file-name {
  font-weight: 500;
  color: #374151;
  font-size: 14px;
}

.file-size {
  font-size: 12px;
  color: #6b7280;
}

.file-remove {
  background: none;
  border: none;
  color: #ef4444;
  cursor: pointer;
  font-size: 16px;
  padding: 4px 8px;
  border-radius: 4px;
}

.add-files-btn {
  background: #f9fafb;
  border: 1px solid #e5e7eb;
  color: #6b7280;
  cursor: pointer;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 12px;
  align-self: flex-start;
}

.add-files-btn:hover {
  background: #f3f4f6;
}

/* Floating Toolbar */
.floating-toolbar {
  background: linear-gradient(135deg, #1f2937 0%, #374151 100%);
  border-radius: 8px;
  padding: 4px;
  display: flex;
  align-items: center;
  gap: 2px;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
  z-index: 1001;
}

.floating-tool-btn {
  background: transparent;
  border: none;
  color: #e5e7eb;
  cursor: pointer;
  padding: 6px 8px;
  border-radius: 4px;
  font-size: 12px;
  transition: background-color 0.2s;
}

.floating-tool-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.floating-toolbar-divider {
  width: 1px;
  height: 20px;
  background: rgba(255, 255, 255, 0.2);
  margin: 0 4px;
}

/* Reply Context */
.reply-context {
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-radius: 12px 12px 0 0;
}

.reply-indicator {
  font-size: 12px;
  color: #6366f1;
  font-weight: 500;
}

.reply-content {
  flex: 1;
  margin: 0 12px;
  color: #6b7280;
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.reply-cancel {
  background: none;
  border: none;
  color: #6b7280;
  cursor: pointer;
  font-size: 16px;
  padding: 4px 8px;
  border-radius: 4px;
}

/* Main Input Area */
.main-input-area {
  display: flex;
  align-items: flex-end;
  gap: 12px;
  padding: 16px;
}

.action-btn {
  background: #f9fafb;
  border: 1px solid #e5e7eb;
  color: #6b7280;
  cursor: pointer;
  padding: 8px;
  border-radius: 8px;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  position: relative;
  transition: all 0.2s;
}

.action-btn:hover,
.action-btn.active {
  background: #6366f1;
  color: white;
  border-color: #6366f1;
}

/* 🚀 NEW: Floating Formatting Toolbar Button */
.floating-toolbar-btn {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%) !important;
  color: white !important;
  border: none !important;
  position: relative;
  overflow: hidden;
}

.floating-toolbar-btn::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.2), transparent);
  transition: left 0.6s;
}

.floating-toolbar-btn:hover::before {
  left: 100%;
}

.floating-toolbar-btn:hover {
  background: linear-gradient(135deg, #764ba2 0%, #667eea 100%) !important;
  transform: translateY(-2px) scale(1.05);
  box-shadow: 0 8px 16px rgba(102, 126, 234, 0.3);
}

.floating-toolbar-btn.active {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%) !important;
  box-shadow: 0 0 0 3px rgba(240, 147, 251, 0.3);
  animation: pulse-glow 2s infinite;
}

@keyframes pulse-glow {

  0%,
  100% {
    box-shadow: 0 0 0 3px rgba(240, 147, 251, 0.3);
  }

  50% {
    box-shadow: 0 0 0 6px rgba(240, 147, 251, 0.2);
  }
}

.file-count-badge {
  position: absolute;
  top: -6px;
  right: -6px;
  background: #ef4444;
  color: white;
  border-radius: 50%;
  width: 18px;
  height: 18px;
  font-size: 10px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
}

.input-container {
  flex: 1;
  position: relative;
}

.message-textarea {
  width: 100%;
  border: none;
  background: transparent;
  font-size: 15px;
  line-height: 1.5;
  resize: none;
  outline: none;
  padding: 8px 12px;
  min-height: 24px;
  max-height: 120px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

.message-textarea::placeholder {
  color: #9ca3af;
}

.character-counter {
  position: absolute;
  bottom: 4px;
  right: 8px;
  font-size: 11px;
  color: #9ca3af;
  background: rgba(255, 255, 255, 0.9);
  padding: 2px 6px;
  border-radius: 4px;
}

.character-counter.warning {
  color: #ef4444;
  font-weight: 600;
}

.action-buttons {
  display: flex;
  gap: 8px;
  align-items: center;
}

.mode-btn {
  background: #f9fafb;
  border: 1px solid #e5e7eb;
  color: #6b7280;
  cursor: pointer;
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
  transition: all 0.2s;
}

.mode-btn:hover,
.mode-btn.active {
  background: #6366f1;
  color: white;
  border-color: #6366f1;
}

.send-btn {
  background: #d1d5db;
  border: none;
  color: #9ca3af;
  cursor: not-allowed;
  padding: 8px 12px;
  border-radius: 8px;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.send-btn.active {
  background: #6366f1;
  color: white;
  cursor: pointer;
}

.send-btn.active:hover {
  background: #4f46e5;
  transform: translateY(-1px);
}

.loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid transparent;
  border-top: 2px solid currentColor;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* Emoji Picker */
.emoji-picker {
  position: absolute;
  bottom: 100%;
  right: 0;
  background: #ffffff;
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  width: 360px;
  max-height: 400px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
  z-index: 200;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* Emoji Header */
.emoji-header {
  padding: 12px 16px;
  border-bottom: 1px solid #e5e7eb;
  display: flex;
  align-items: center;
  gap: 12px;
  background: #f8fafc;
}

.emoji-search-container {
  flex: 1;
  position: relative;
}

.emoji-search {
  width: 100%;
  padding: 8px 12px 8px 36px;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  font-size: 14px;
  background: #ffffff;
  transition: border-color 0.2s;
}

.emoji-search:focus {
  outline: none;
  border-color: #6366f1;
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

.search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: #9ca3af;
  pointer-events: none;
}

.emoji-close {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 18px;
  color: #6b7280;
  padding: 4px 8px;
  border-radius: 6px;
  transition: background-color 0.2s;
}

.emoji-close:hover {
  background: #f3f4f6;
}

/* Category Tabs */
.emoji-categories {
  display: flex;
  padding: 8px 12px;
  gap: 4px;
  border-bottom: 1px solid #e5e7eb;
  background: #ffffff;
  overflow-x: auto;
}

.category-tab {
  background: none;
  border: none;
  cursor: pointer;
  padding: 8px 12px;
  border-radius: 8px;
  font-size: 16px;
  transition: all 0.2s;
  white-space: nowrap;
  min-width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.category-tab:hover {
  background: #f3f4f6;
  transform: scale(1.05);
}

.category-tab.active {
  background: #6366f1;
  color: white;
  box-shadow: 0 2px 4px rgba(99, 102, 241, 0.3);
}

/* Emoji Content */
.emoji-content {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

.emoji-section {
  margin-bottom: 16px;
}

.section-title {
  font-size: 12px;
  font-weight: 600;
  color: #6b7280;
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* Emoji Grid */
.emoji-grid {
  display: grid;
  grid-template-columns: repeat(8, 1fr);
  gap: 4px;
}

.emoji-item {
  background: none;
  border: none;
  cursor: pointer;
  padding: 8px;
  border-radius: 6px;
  font-size: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  aspect-ratio: 1;
  position: relative;
}

.emoji-item:hover {
  background: #f3f4f6;
  transform: scale(1.2);
  z-index: 1;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.emoji-item:active {
  transform: scale(1.1);
}

/* No Results */
.no-results {
  text-align: center;
  padding: 24px;
  color: #6b7280;
}

.no-results-icon {
  font-size: 32px;
  margin-bottom: 12px;
  opacity: 0.5;
}

.no-results-text {
  font-size: 16px;
  font-weight: 500;
  margin-bottom: 4px;
}

.no-results-hint {
  font-size: 14px;
}

/* Emoji Footer */
.emoji-footer {
  border-top: 1px solid #e5e7eb;
  padding: 8px 16px;
  background: #f8fafc;
}

.emoji-count {
  font-size: 11px;
  color: #9ca3af;
  text-align: center;
}

/* Responsive Design */
@media (max-width: 640px) {
  .emoji-picker {
    width: 320px;
    max-height: 350px;
  }

  .emoji-grid {
    grid-template-columns: repeat(6, 1fr);
  }

  .emoji-item {
    font-size: 16px;
    padding: 6px;
  }
}

/* Mode Label */
.mode-label {
  position: absolute;
  top: -40px;
  left: 0;
  right: 0;
  z-index: 50;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-radius: 8px 8px 0 0;
  padding: 8px 16px;
  box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.1);
  animation: slideDown 0.3s ease-out;
}

.mode-label.mode-markdown {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

.mode-label.mode-code {
  background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
}

.mode-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
}

.mode-icon {
  font-size: 16px;
}

.mode-text {
  font-size: 14px;
  font-weight: 500;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
}

@keyframes slideDown {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Unified Format Mode Button */
.unified-mode-btn {
  background: #f9fafb;
  border: 1px solid #e5e7eb;
  color: #6b7280;
  cursor: pointer;
  padding: 8px 12px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 600;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  min-width: 44px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.unified-mode-btn:hover {
  background: #f3f4f6;
  border-color: #d1d5db;
  transform: translateY(-1px);
}

.unified-mode-btn.active {
  border-color: #6366f1;
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.1);
}

.unified-mode-btn.mode-markdown {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  color: white;
  border-color: #f5576c;
}

.unified-mode-btn.mode-code {
  background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
  color: white;
  border-color: #00f2fe;
}

.mode-icon {
  font-size: 14px;
  font-weight: 700;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

/* Animations */
.preview-popup-enter-active,
.preview-popup-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.preview-popup-enter-from {
  opacity: 0;
  transform: translateY(20px);
}

.preview-popup-leave-to {
  opacity: 0;
  transform: translateY(20px);
}

.toolbar-float-enter-active,
.toolbar-float-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.toolbar-float-enter-from,
.toolbar-float-leave-to {
  opacity: 0;
  transform: scale(0.9);
}

.emoji-fade-enter-active,
.emoji-fade-leave-active {
  transition: all 0.2s ease;
}

.emoji-fade-enter-from,
.emoji-fade-leave-to {
  opacity: 0;
  transform: translateY(10px);
}
</style>