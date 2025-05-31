<template>
  <div class="bg-white/90 backdrop-blur-sm border-t border-gray-200">
    <!-- Drag Upload Overlay -->
    <div v-if="isDragging" 
         class="absolute inset-0 bg-blue-50/90 backdrop-blur-sm border-2 border-dashed border-blue-300 rounded-lg flex items-center justify-center z-50"
         @drop="handleDrop"
         @dragover.prevent
         @dragleave="isDragging = false">
      <div class="text-center">
        <div class="w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center mx-auto mb-4">
          <svg class="w-8 h-8 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"></path>
          </svg>
        </div>
        <h3 class="text-lg font-semibold text-blue-700 mb-2">Drop files to upload</h3>
        <p class="text-blue-600">Images, documents, and more</p>
      </div>
    </div>

    <!-- File Upload Progress -->
    <div v-if="uploading" class="px-4 py-2 bg-blue-50 border-b border-blue-100">
      <div class="flex items-center space-x-3">
        <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600"></div>
        <span class="text-sm text-blue-700">Uploading files...</span>
        <div class="flex-1 bg-blue-200 rounded-full h-2">
          <div class="bg-blue-600 h-2 rounded-full transition-all duration-300" 
               :style="{ width: uploadProgress + '%' }"></div>
        </div>
        <span class="text-xs text-blue-600">{{ uploadProgress }}%</span>
      </div>
    </div>

    <!-- Selected Files Preview -->
    <div v-if="selectedFiles.length > 0" class="px-4 py-3 bg-gray-50 border-b border-gray-100">
      <div class="flex items-center justify-between mb-2">
        <span class="text-sm font-medium text-gray-700">Selected Files ({{ selectedFiles.length }})</span>
        <button @click="clearFiles" class="text-xs text-gray-500 hover:text-red-500">Clear All</button>
      </div>
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-2">
        <div v-for="(file, index) in selectedFiles" :key="index" 
             class="relative group bg-white rounded-lg border border-gray-200 p-2">
          <div class="flex flex-col items-center space-y-1">
            <div class="w-8 h-8 bg-gray-100 rounded flex items-center justify-center">
              <svg v-if="isImageFile(file)" class="w-5 h-5 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path>
              </svg>
              <svg v-else class="w-5 h-5 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
              </svg>
            </div>
            <span class="text-xs text-gray-600 truncate w-full text-center">{{ file.name }}</span>
            <span class="text-xs text-gray-400">{{ formatFileSize(file.size) }}</span>
          </div>
          <button @click="removeFile(index)" 
                  class="absolute -top-1 -right-1 w-5 h-5 bg-red-500 text-white rounded-full flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Main Input Area -->
    <div class="p-4">
      <div class="flex items-end space-x-3">
        <!-- File Upload Button -->
        <div class="flex-shrink-0">
          <input ref="fileInput" 
                 type="file" 
                 multiple 
                 class="hidden" 
                 @change="handleFileSelect"
                 accept="image/*,application/pdf,.doc,.docx,.txt,.csv,.xlsx,.ppt,.pptx">
          <button @click="$refs.fileInput.click()" 
                  class="w-10 h-10 bg-gray-100 hover:bg-gray-200 rounded-full flex items-center justify-center transition-colors group"
                  :disabled="uploading">
            <svg class="w-5 h-5 text-gray-500 group-hover:text-gray-700" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"></path>
            </svg>
          </button>
        </div>

        <!-- Message Input -->
        <div class="flex-1 relative">
          <textarea
            ref="messageInput"
            v-model="message"
            @keydown="handleKeyDown"
            @input="adjustHeight"
            @dragover.prevent="handleDragOver"
            @drop.prevent="handleDrop"
            placeholder="Type your message..."
            class="w-full px-4 py-3 bg-white border border-gray-200 rounded-2xl resize-none focus:outline-none focus:ring-2 focus:ring-blue-400/50 focus:border-blue-400 transition-all placeholder-gray-400"
            :class="{ 'pr-12': message.trim() }"
            rows="1"
            :disabled="uploading"
            style="min-height: 44px; max-height: 120px;"
          ></textarea>
          
          <!-- Character Count -->
          <div v-if="message.length > 1500" 
               class="absolute bottom-2 right-4 text-xs"
               :class="message.length > 1800 ? 'text-red-400' : 'text-gray-400'">
            {{ message.length }}/2000
          </div>
        </div>

        <!-- Emoji Picker -->
        <div class="flex-shrink-0 relative">
          <button @click="showEmojiPicker = !showEmojiPicker" 
                  class="w-10 h-10 bg-gray-100 hover:bg-yellow-100 rounded-full flex items-center justify-center transition-colors group"
                  :disabled="uploading">
            <svg class="w-5 h-5 text-gray-500 group-hover:text-yellow-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
            </svg>
          </button>
          
          <!-- Enterprise Emoji Panel -->
          <div v-if="showEmojiPicker" 
               class="absolute bottom-14 right-0 bg-white border border-gray-200 rounded-xl shadow-lg p-4 z-20 w-80">
            
            <!-- Emoji Categories -->
            <div class="flex space-x-1 mb-3 border-b border-gray-100 pb-2">
              <button v-for="category in emojiCategories" :key="category.name"
                      @click="activeEmojiCategory = category.name"
                      class="px-3 py-1 text-sm rounded-lg transition-colors"
                      :class="activeEmojiCategory === category.name 
                        ? 'bg-blue-100 text-blue-700' 
                        : 'text-gray-600 hover:bg-gray-100'">
                {{ category.icon }} {{ category.name }}
              </button>
            </div>
            
            <!-- Emoji Grid -->
            <div class="grid grid-cols-8 gap-1 max-h-48 overflow-y-auto">
              <button v-for="emoji in getCurrentEmojis()" :key="emoji.char"
                      @click="insertEmoji(emoji.char)"
                      class="w-8 h-8 flex items-center justify-center rounded hover:bg-gray-100 transition-colors text-lg"
                      :title="emoji.name">
                {{ emoji.char }}
              </button>
            </div>
            
            <!-- Quick Gestures Bar -->
            <div class="mt-3 pt-3 border-t border-gray-100">
              <div class="text-xs text-gray-500 mb-2">Quick Actions</div>
              <div class="flex space-x-2">
                <button v-for="gesture in quickGestures" :key="gesture.char"
                        @click="insertEmoji(gesture.char)"
                        class="px-2 py-1 bg-gray-50 hover:bg-gray-100 rounded text-sm transition-colors"
                        :title="gesture.name">
                  {{ gesture.char }} {{ gesture.name }}
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- Send Button -->
        <div class="flex-shrink-0">
          <button @click="sendMessage" 
                  :disabled="!canSend || uploading"
                  class="w-10 h-10 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-300 rounded-full flex items-center justify-center transition-colors">
            <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"></path>
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue';
import { useChatStore } from '@/stores/chat';

const emit = defineEmits(['send']);

// State Management
const message = ref('');
const selectedFiles = ref([]);
const uploading = ref(false);
const uploadProgress = ref(0);
const error = ref('');
const showEmojiPicker = ref(false);
const activeEmojiCategory = ref('Smileys');
const isDragging = ref(false);

// References
const messageInput = ref(null);
const fileInput = ref(null);

// Chat store
const chatStore = useChatStore();

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
  return (message.value.trim() || selectedFiles.value.length > 0) && !uploading.value;
});

// File Handling Functions
function handleKeyDown(event) {
  if (event.key === 'Enter' && !event.shiftKey) {
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
}

// Drag Upload Handling
function handleDragOver(event) {
  event.preventDefault();
  if (!isDragging.value) {
    isDragging.value = true;
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

function addFiles(files) {
  const maxFileSize = 50 * 1024 * 1024; // 50MB
  const maxFiles = 10;
  
  const validFiles = files.filter(file => {
    if (file.size > maxFileSize) {
      error.value = `File "${file.name}" is too large. Maximum size is 50MB.`;
      return false;
    }
    return true;
  });
  
  if (selectedFiles.value.length + validFiles.length > maxFiles) {
    error.value = `Maximum ${maxFiles} files allowed.`;
    return;
  }
  
  selectedFiles.value.push(...validFiles);
  error.value = '';
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

// Send Message
async function sendMessage() {
  if (!canSend.value) return;
  
  try {
    uploading.value = true;
    error.value = '';
    uploadProgress.value = 0;
    
    // Upload files first if any
    let uploadedFiles = [];
    if (selectedFiles.value.length > 0) {
      uploadedFiles = await chatStore.uploadFiles(selectedFiles.value);
    }
    
    // Emit send event
    emit('send', {
      content: message.value.trim(),
      files: uploadedFiles
    });
    
    // Reset form
    message.value = '';
    selectedFiles.value = [];
    adjustHeight();
    
  } catch (err) {
    error.value = err.message || 'Failed to send message';
  } finally {
    uploading.value = false;
    uploadProgress.value = 0;
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

// File Upload
async function uploadFiles() {
  if (selectedFiles.value.length === 0) return [];
  
  uploading.value = true;
  uploadProgress.value = 0;
  
  try {
    const formData = new FormData();
    selectedFiles.value.forEach(file => {
      formData.append('files', file);
    });
    
    const response = await fetch('/api/upload', {
      method: 'POST',
      body: formData,
      headers: {
        'Authorization': `Bearer ${authStore.token}`
      }
    });
    
    if (!response.ok) {
      throw new Error('Upload failed');
    }
    
    const result = await response.json();
    uploadProgress.value = 100;
    
    return result.files || [];
  } catch (err) {
    error.value = 'Failed to upload files. Please try again.';
    throw err;
  } finally {
    uploading.value = false;
    uploadProgress.value = 0;
  }
}
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

/* Smooth Font Rendering */
* {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>