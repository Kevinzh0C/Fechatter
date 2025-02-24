<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="isOpen" class="modal-backdrop" @click="handleBackdropClick">
        <div class="modal-container" @click.stop>
          <!-- Header -->
          <div class="modal-header">
            <h2 class="modal-title">Search Messages</h2>
            <button @click="close" class="close-button">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 6L6 18M6 6l12 12"/>
              </svg>
            </button>
          </div>

          <!-- Search Input -->
          <div class="search-input-container">
            <div class="search-icon">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8"/>
                <path d="M21 21l-4.35-4.35"/>
              </svg>
            </div>
            <input
              ref="searchInput"
              v-model="searchQuery"
              @input="debouncedSearch"
              @keydown.enter="performSearch"
              @keydown.escape="close"
              type="text"
              placeholder="Search messages..."
              class="search-input"
            />
            <div v-if="searchQuery" @click="clearSearch" class="clear-button">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <path d="M15 9l-6 6M9 9l6 6"/>
              </svg>
            </div>
          </div>

          <!-- Search Options -->
          <div class="search-options">
            <label class="option-item">
              <input 
                type="checkbox" 
                v-model="searchCurrentChatOnly"
                @change="performSearch"
              />
              <span>Search in current chat only</span>
            </label>
          </div>

          <!-- Search Results -->
          <div class="search-results">
            <!-- Loading State -->
            <div v-if="loading" class="loading-state">
              <div class="spinner"></div>
              <p>Searching...</p>
            </div>

            <!-- No Results -->
            <div v-else-if="searchPerformed && results.length === 0" class="empty-state">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="11" cy="11" r="8"/>
                <path d="M21 21l-4.35-4.35"/>
                <path d="M8 11h6"/>
              </svg>
              <p>No messages found</p>
              <span class="empty-hint">Try different keywords or search options</span>
            </div>

            <!-- Results List -->
            <div v-else-if="results.length > 0" class="results-list">
              <div class="results-header">
                <span>{{ results.length }} results found</span>
              </div>
              <div
                v-for="result in results"
                :key="result.id"
                @click="jumpToMessage(result)"
                class="result-item"
              >
                <div class="result-header">
                  <span class="result-sender">{{ result.sender_fullname || result.sender_name || result.sender?.fullname || 'Unknown' }}</span>
                  <span class="result-time">{{ formatTime(result.created_at) }}</span>
                </div>
                <div class="result-chat">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 11.5a8.38 8.38 0 01-.9 3.8 8.5 8.5 0 01-7.6 4.7 8.38 8.38 0 01-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 01-.9-3.8 8.5 8.5 0 014.7-7.6 8.38 8.38 0 013.8-.9h.5a8.48 8.48 0 018 8v.5z"/>
                  </svg>
                  {{ result.chat_name }}
                </div>
                <div class="result-content" v-html="highlightMatch(result.content, searchQuery)"></div>
              </div>
            </div>

            <!-- Initial State -->
            <div v-else class="initial-state">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="11" cy="11" r="8"/>
                <path d="M21 21l-4.35-4.35"/>
              </svg>
              <p>Type to search messages</p>
              <div class="search-tips">
                <p class="tip-title">Search tips:</p>
                <ul>
                  <li>Use quotes for exact phrases: "hello world"</li>
                  <li>Search by sender: from:username</li>
                  <li>Search by date: after:2024-01-01</li>
                </ul>
              </div>
            </div>
          </div>

          <!-- Footer -->
          <div class="modal-footer">
            <div class="footer-hint">
              Press <kbd>ESC</kbd> to close • <kbd>ENTER</kbd> to search
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref, watch, nextTick } from 'vue';
import { useChatStore } from '@/stores/chat';
import { useRouter } from 'vue-router';
import { SearchService } from '@/services/api';

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  },
  currentChatId: {
    type: [Number, String],
    default: null
  }
});

const emit = defineEmits(['update:modelValue', 'message-selected']);

const chatStore = useChatStore();
const router = useRouter();

// State
const isOpen = ref(props.modelValue);
const searchQuery = ref('');
const searchCurrentChatOnly = ref(false);
const loading = ref(false);
const results = ref([]);
const searchPerformed = ref(false);
const searchInput = ref(null);

// Debounce timer
let debounceTimer = null;

// Watch for prop changes
watch(() => props.modelValue, (newVal) => {
  isOpen.value = newVal;
  if (newVal) {
    // Reset state when opening
    searchQuery.value = '';
    results.value = [];
    searchPerformed.value = false;
    searchCurrentChatOnly.value = !!props.currentChatId;
    
    // Focus input after DOM update
    nextTick(() => {
      searchInput.value?.focus();
    });
});

// Methods
function close() {
  isOpen.value = false;
  emit('update:modelValue', false);
}

function handleBackdropClick(event) {
  if (event.target === event.currentTarget) {
    close();
  }

function clearSearch() {
  searchQuery.value = '';
  results.value = [];
  searchPerformed.value = false;
  searchInput.value?.focus();
}

async function performSearch() {
  if (!searchQuery.value.trim()) {
    results.value = [];
    searchPerformed.value = false;
    return;
  }

  loading.value = true;
  searchPerformed.value = true;

  try {
    const searchParams = {
      query: searchQuery.value.trim(),
      limit: 50
    };

    if (searchCurrentChatOnly.value && props.currentChatId) {
      searchParams.chatId = props.currentChatId;
    }

    const response = await SearchService.search(searchParams);
    results.value = response.results || [];
  } catch (error) {
    console.error('Search failed:', error);
    results.value = [];
  } finally {
    loading.value = false;
  }

function debouncedSearch() {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
  
  debounceTimer = setTimeout(() => {
    performSearch();
  }, 300);
}

function highlightMatch(text, query) {
  if (!query) return text;
  
  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(`(${escapedQuery})`, 'gi');
  return text.replace(regex, '<mark>$1</mark>');
}

function formatTime(timestamp) {
  const date = new Date(timestamp);
  const now = new Date();
  const diffInHours = (now - date) / (1000 * 60 * 60);
  
  if (diffInHours < 24) {
    return date.toLocaleTimeString('zh-CN', { 
      hour: '2-digit', 
      minute: '2-digit',
      hour12: false 
    });
  } else if (diffInHours < 168) { // 7 days
    return date.toLocaleDateString('zh-CN', { 
      weekday: 'short',
      hour: '2-digit', 
      minute: '2-digit',
      hour12: false 
    });
  } else {
    return date.toLocaleDateString('zh-CN', { 
      month: 'short', 
      day: 'numeric',
      year: 'numeric'
    });

async function jumpToMessage(result) {
  // Close modal
  close();
  
  // Navigate to chat if needed
  if (result.chat_id !== chatStore.currentChatId) {
    await router.push(`/chat/${result.chat_id}`);
    // Wait for chat to load
    await new Promise(resolve => setTimeout(resolve, 300));
  }
  
  // Emit event to scroll to message
  emit('message-selected', {
    messageId: result.id,
    chatId: result.chat_id
  });
</script>

<style scoped>
/* Modal Backdrop */
.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 1rem;
}

/* Modal Container */
.modal-container {
  background: white;
  border-radius: 12px;
  width: 100%;
  max-width: 600px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  overflow: hidden;
}

/* Modal Header */
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.5rem;
  border-bottom: 1px solid #e5e7eb;
}

.modal-title {
  font-size: 1.25rem;
  font-weight: 600;
  color: #111827;
  margin: 0;
}

.close-button {
  background: none;
  border: none;
  color: #6b7280;
  cursor: pointer;
  padding: 0.5rem;
  border-radius: 6px;
  transition: all 0.2s;
}

.close-button:hover {
  background-color: #f3f4f6;
  color: #374151;
}

/* Search Input */
.search-input-container {
  position: relative;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid #e5e7eb;
}

.search-icon {
  position: absolute;
  left: 2.5rem;
  top: 50%;
  transform: translateY(-50%);
  color: #9ca3af;
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 0.75rem 1rem 0.75rem 3rem;
  font-size: 1rem;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  transition: all 0.2s;
  background-color: #f9fafb;
}

.search-input:focus {
  outline: none;
  border-color: #3b82f6;
  background-color: white;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.clear-button {
  position: absolute;
  right: 2.5rem;
  top: 50%;
  transform: translateY(-50%);
  color: #9ca3af;
  cursor: pointer;
  padding: 0.25rem;
  border-radius: 4px;
  transition: all 0.2s;
}

.clear-button:hover {
  color: #6b7280;
  background-color: #f3f4f6;
}

/* Search Options */
.search-options {
  padding: 1rem 1.5rem;
  border-bottom: 1px solid #e5e7eb;
  background-color: #f9fafb;
}

.option-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  color: #374151;
  cursor: pointer;
  user-select: none;
}

.option-item input[type="checkbox"] {
  cursor: pointer;
}

/* Search Results */
.search-results {
  flex: 1;
  overflow-y: auto;
  min-height: 300px;
}

/* Loading State */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3rem;
  color: #6b7280;
}

.spinner {
  width: 2rem;
  height: 2rem;
  border: 2px solid #e5e7eb;
  border-top-color: #3b82f6;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 1rem;
}

@keyframes spin {
  to { transform: rotate(360deg); }

/* Empty State */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3rem;
  text-align: center;
  color: #6b7280;
}

.empty-state svg {
  color: #d1d5db;
  margin-bottom: 1rem;
}

.empty-state p {
  font-weight: 500;
  margin-bottom: 0.5rem;
}

.empty-hint {
  font-size: 0.875rem;
  color: #9ca3af;
}

/* Initial State */
.initial-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 3rem;
  text-align: center;
  color: #6b7280;
}

.initial-state svg {
  color: #d1d5db;
  margin-bottom: 1rem;
}

.initial-state p {
  font-weight: 500;
  margin-bottom: 2rem;
}

.search-tips {
  text-align: left;
  background-color: #f9fafb;
  border-radius: 8px;
  padding: 1rem;
  font-size: 0.875rem;
}

.tip-title {
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: #374151;
}

.search-tips ul {
  list-style: none;
  margin: 0;
  padding: 0;
}

.search-tips li {
  padding: 0.25rem 0;
  color: #6b7280;
}

.search-tips li::before {
  content: "•";
  color: #9ca3af;
  margin-right: 0.5rem;
}

/* Results List */
.results-header {
  padding: 1rem 1.5rem;
  font-size: 0.875rem;
  color: #6b7280;
  border-bottom: 1px solid #e5e7eb;
}

.result-item {
  padding: 1rem 1.5rem;
  border-bottom: 1px solid #e5e7eb;
  cursor: pointer;
  transition: background-color 0.2s;
}

.result-item:hover {
  background-color: #f9fafb;
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.25rem;
}

.result-sender {
  font-weight: 600;
  color: #111827;
  font-size: 0.875rem;
}

.result-time {
  font-size: 0.75rem;
  color: #9ca3af;
}

.result-chat {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.75rem;
  color: #6b7280;
  margin-bottom: 0.5rem;
}

.result-chat svg {
  color: #9ca3af;
}

.result-content {
  font-size: 0.875rem;
  color: #374151;
  line-height: 1.5;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.result-content mark {
  background-color: #fef3c7;
  color: inherit;
  padding: 0.125rem;
  border-radius: 2px;
}

/* Modal Footer */
.modal-footer {
  padding: 1rem 1.5rem;
  border-top: 1px solid #e5e7eb;
  background-color: #f9fafb;
}

.footer-hint {
  font-size: 0.75rem;
  color: #6b7280;
  text-align: center;
}

.footer-hint kbd {
  display: inline-block;
  padding: 0.125rem 0.375rem;
  font-size: 0.75rem;
  font-family: monospace;
  background-color: white;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
}

/* Transitions */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .modal-container,
.modal-leave-active .modal-container {
  transition: transform 0.3s ease;
}

.modal-enter-from .modal-container {
  transform: scale(0.95);
}

.modal-leave-to .modal-container {
  transform: scale(0.95);
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .modal-container {
    background: #1f2937;
    color: #f9fafb;
  }
  
  .modal-header,
  .search-input-container,
  .search-options,
  .results-header,
  .result-item {
    border-color: #374151;
  }
  
  .modal-title,
  .result-sender {
    color: #f9fafb;
  }
  
  .search-input {
    background-color: #111827;
    border-color: #374151;
    color: #f9fafb;
  }
  
  .search-input:focus {
    background-color: #1f2937;
    border-color: #3b82f6;
  }
  
  .search-options {
    background-color: #111827;
  }
  
  .close-button:hover,
  .clear-button:hover {
    background-color: #374151;
    color: #f9fafb;
  }
  
  .result-item:hover {
    background-color: #111827;
  }
  
  .result-content {
    color: #e5e7eb;
  }
  
  .result-content mark {
    background-color: #854d0e;
    color: #fef3c7;
  }
  
  .search-tips {
    background-color: #111827;
  }
  
  .modal-footer {
    background-color: #111827;
  }
  
  .footer-hint kbd {
    background-color: #374151;
    border-color: #4b5563;
    color: #f9fafb;
  }
</style>