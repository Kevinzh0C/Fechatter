<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="modelValue" class="search-modal-overlay" @click="handleOverlayClick" @keydown.esc="close">
        <div class="search-modal" @click.stop role="dialog" aria-modal="true" aria-label="Search messages">
          <!-- 搜索头部 -->
          <div class="search-header">

            <!-- 搜索输入框 -->
            <div class="search-input-wrapper">
              <svg class="search-icon" width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
                <path
                  d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" />
              </svg>

              <input ref="searchInput" v-model="localQuery" @input="handleSearchInput" @keydown="handleKeydown"
                type="text" class="search-input" placeholder="Search messages... (Ctrl+K)" aria-label="Search query"
                autocomplete="off" spellcheck="false" />

              <button v-if="localQuery" @click="clearSearch" class="clear-button" aria-label="Clear search">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                  <path
                    d="M4.646 4.646a.5.5 0 01.708 0L8 7.293l2.646-2.647a.5.5 0 01.708.708L8.707 8l2.647 2.646a.5.5 0 01-.708.708L8 8.707l-2.646 2.647a.5.5 0 01-.708-.708L7.293 8 4.646 5.354a.5.5 0 010-.708z" />
                </svg>
              </button>
            </div>

            <!-- 快捷过滤器 -->
            <div class="search-filters">
              <button v-for="preset in searchPresets" :key="preset.filter" @click="applyPreset(preset)"
                class="filter-preset" :class="{ active: localQuery.includes(preset.filter) }">
                {{ preset.label }}
              </button>
            </div>

            <!-- Advanced Filters Toggle (Hidden for now - backend doesn't support yet) -->
            <!--
            <button
              @click="showFilters = !showFilters"
              class="filters-toggle"
              :class="{ active: showFilters }"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M3 17v2h6v-2H3zM3 5v2h10V5H3zm10 16v-2h8v-2h-8v-2h-2v6h2zM7 9v2H3v2h4v2h2V9H7zm14 4v-2H11v2h10zm-6-4h2V7h4V5h-4V3h-2v6z"/>
              </svg>
              <span>Filters</span>
            </button>
            -->
          </div>

          <!-- Advanced Filters -->
          <Transition name="slide-down">
            <div v-if="showFilters" class="filters-section">
              <div class="filters-grid">
                <div class="filter-group">
                  <label>From:</label>
                  <select v-model="filters.sender_id" @change="performSearch">
                    <option value="">Anyone</option>
                    <option v-for="member in chatMembers" :key="member.id" :value="member.id">
                      {{ member.fullname || member.username || 'Unknown' }}
                    </option>
                  </select>
                </div>

                <div class="filter-group">
                  <label>When:</label>
                  <select v-model="datePreset" @change="handleDatePresetChange">
                    <option value="">Any time</option>
                    <option value="today">Today</option>
                    <option value="yesterday">Yesterday</option>
                    <option value="week">Past 7 days</option>
                    <option value="month">Past 30 days</option>
                  </select>
                </div>

                <div class="filter-group checkbox-group">
                  <label>
                    <input type="checkbox" v-model="filters.has_files" @change="performSearch" />
                    <span>Has files</span>
                  </label>
                </div>

                <div class="filter-group">
                  <label>Sort:</label>
                  <select v-model="filters.sort_by" @change="performSearch">
                    <option value="relevance">Most relevant</option>
                    <option value="date_desc">Newest first</option>
                    <option value="date_asc">Oldest first</option>
                  </select>
                </div>
              </div>
            </div>
          </Transition>

          <!-- Results Section -->
          <div class="results-section">
            <!-- Loading State -->
            <div v-if="loading" class="loading-state">
              <div class="loading-spinner"></div>
              <p>Searching...</p>
            </div>

            <!-- No Results -->
            <div v-else-if="searchPerformed && results.length === 0" class="no-results">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8" />
                <path d="m21 21-4.35-4.35M11 8v3M11 14h.01" />
              </svg>
              <h3>No messages found</h3>
              <p>Try adjusting your search terms or filters</p>
            </div>

            <!-- Results List -->
            <div v-else-if="results.length > 0" class="results-list">
              <div class="results-header">
                <span>{{ results.length }} results</span>
              </div>

              <div class="messages-list">
                <div v-for="message in results" :key="message.id" class="search-message-item"
                  @click="selectMessage(message)">
                  <!-- Avatar -->
                  <div class="message-avatar">
                    {{ getInitials(message.sender?.fullname || message.sender_name || 'U') }}
                  </div>

                  <!-- Content -->
                  <div class="message-content">
                    <div class="message-header">
                      <span class="sender-name">
                        {{ message.sender?.fullname || message.sender_name || 'Unknown User' }}
                      </span>
                      <span class="message-time">
                        {{ formatTime(message.created_at) }}
                      </span>
                    </div>

                    <div class="message-text" v-html="highlightQuery(message.content || '')"></div>

                    <!-- Files indicator -->
                    <div v-if="message.files && message.files.length > 0" class="message-files">
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                        <path
                          d="M16.5 6v11.5c0 2.21-1.79 4-4 4s-4-1.79-4-4V5c0-1.38 1.12-2.5 2.5-2.5s2.5 1.12 2.5 2.5v10.5c0 .55-.45 1-1 1s-1-.45-1-1V6H10v9.5c0 1.38 1.12 2.5 2.5 2.5s2.5-1.12 2.5-2.5V5c0-2.21-1.79-4-4-4S7 2.79 7 5v12.5c0 3.04 2.46 5.5 5.5 5.5s5.5-2.46 5.5-5.5V6h-1.5z" />
                      </svg>
                      <span>{{ message.files.length }} file(s)</span>
                    </div>
                  </div>

                  <!-- Action -->
                  <div class="message-action">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                      <path d="M8.59 16.59L10 18l6-6-6-6-1.41 1.41L14.17 12z" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>

            <!-- Empty State (initial) -->
            <div v-else class="empty-state">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8" />
                <path d="m21 21-4.35-4.35" />
              </svg>
              <h3>Search messages</h3>
              <p>Find messages in this channel</p>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import { SearchService } from '@/services/api';
import { useNotifications } from '@/composables/useNotifications';
import { useAuthStore } from '@/stores/auth';
import { useChatStore } from '@/stores/chat';

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  },
  chatId: {
    type: [Number, String],
    required: true
  },
  chatName: {
    type: String,
    default: ''
  },
  chatMembers: {
    type: Array,
    default: () => []
  }
});

const emit = defineEmits(['update:modelValue', 'select-message']);

// Refs
const searchInput = ref(null);
const localQuery = ref('');
const showFilters = ref(false);
const loading = ref(false);
const searchPerformed = ref(false);
const results = ref([]);
const datePreset = ref('');

// Search presets
const searchPresets = ref([
  { label: 'From me', filter: 'from:me' },
  { label: 'Has files', filter: 'has:file' },
  { label: 'Has links', filter: 'has:link' },
  { label: 'Mentions me', filter: 'mentions:me' }
]);

// Filters
const filters = ref({
  sender_id: '',
  start_date: '',
  end_date: '',
  has_files: false,
  sort_by: 'relevance'
});

// Debounce timer
let searchTimer = null;

// Methods
const handleSearchInput = () => {
  clearTimeout(searchTimer);

  if (!localQuery.value.trim()) {
    clearSearchResults();
    return;
  }

  searchTimer = setTimeout(() => {
    performSearch();
  }, 300);
};

const performSearch = async () => {
  const query = localQuery.value.trim();
  if (!query && !hasActiveFilters.value) {
    clearSearchResults();
    return;
  }

  // Check if we have auth token before searching
  const authStore = useAuthStore();
  const chatStore = useChatStore();
  const tokenManager = window.tokenManager || {};
  const hasToken = authStore.token || tokenManager.getAccessToken?.() || localStorage.getItem('auth_token') || sessionStorage.getItem('auth_token');

  console.log('🔍 [Search] Auth check before search:', {
    hasAuthStoreToken: !!authStore.token,
    hasTokenManager: !!tokenManager.getAccessToken?.(),
    hasLocalStorage: !!localStorage.getItem('auth_token'),
    hasSessionStorage: !!sessionStorage.getItem('auth_token'),
    isAuthenticated: authStore.isAuthenticated,
    user: authStore.user
  });

  if (!hasToken || !authStore.isAuthenticated) {
    console.error('🔐 No auth token found - cannot perform search');
    const { notifyError } = useNotifications();
    notifyError('Please log in to search messages.');
    return;
  }

  // Check if user is member of the chat
  const currentChat = chatStore.getChatById(props.chatId);
  if (!currentChat) {
    console.error('🔍 Chat not found:', props.chatId);
    const { notifyError } = useNotifications();
    notifyError('Chat not found. Please refresh the page.');
    return;
  }

  // Ensure we have chat members loaded
  if (!chatStore.chatMembers[props.chatId]) {
    try {
      await chatStore.fetchChatMembers(props.chatId);
    } catch (error) {
      console.error('🔍 Failed to fetch chat members:', error);
    }
  }

  loading.value = true;
  searchPerformed.value = true;

  try {
    // Backend currently only supports basic search parameters
    const searchParams = {
      query: query,
      chatId: props.chatId,
      limit: 50,
      sortBy: filters.value.sort_by || 'relevance'
    };

    console.log('🔍 Performing search with params:', searchParams);

    const response = await SearchService.search(searchParams);
    results.value = response.results || [];
  } catch (error) {
    console.error('Search failed:', error);
    console.error('🔍 Search error details:', {
      status: error.response?.status,
      statusText: error.response?.statusText,
      data: error.response?.data,
      headers: error.response?.headers,
      config: {
        url: error.config?.url,
        method: error.config?.method,
        headers: error.config?.headers
      }
    });

    const { notifyError } = useNotifications();

    if (error.response?.status === 401) {
      notifyError('Session expired. Please log in again.');
    } else {
      notifyError('Search failed. Please try again.');
    }

    results.value = [];
  } finally {
    loading.value = false;
  }
};

const selectMessage = (message) => {
  emit('select-message', {
    messageId: message.id,
    chatId: message.chat_id || props.chatId
  });
  close();
};

const clearSearch = () => {
  localQuery.value = '';
  clearSearchResults();
  searchInput.value?.focus();
};

const clearSearchResults = () => {
  results.value = [];
  searchPerformed.value = false;
};

const applyPreset = (preset) => {
  // Toggle preset filter in search query
  if (localQuery.value.includes(preset.filter)) {
    localQuery.value = localQuery.value.replace(preset.filter, '').trim();
  } else {
    localQuery.value = localQuery.value ? `${localQuery.value} ${preset.filter}` : preset.filter;
  }
  performSearch();
};

const close = () => {
  emit('update:modelValue', false);
  // Clean up on close
  setTimeout(() => {
    clearSearch();
    showFilters.value = false;
    filters.value = {
      sender_id: '',
      start_date: '',
      end_date: '',
      has_files: false,
      sort_by: 'relevance'
    };
    datePreset.value = '';
  }, 300);
};

const handleOverlayClick = (event) => {
  if (event.target === event.currentTarget) {
    close();
  }
};

const handleDatePresetChange = () => {
  const today = new Date();
  const formatDate = (date) => date.toISOString().split('T')[0];

  switch (datePreset.value) {
    case 'today':
      filters.value.start_date = formatDate(today);
      filters.value.end_date = formatDate(today);
      break;
    case 'yesterday':
      const yesterday = new Date(today);
      yesterday.setDate(yesterday.getDate() - 1);
      filters.value.start_date = formatDate(yesterday);
      filters.value.end_date = formatDate(yesterday);
      break;
    case 'week':
      const weekAgo = new Date(today);
      weekAgo.setDate(weekAgo.getDate() - 7);
      filters.value.start_date = formatDate(weekAgo);
      filters.value.end_date = formatDate(today);
      break;
    case 'month':
      const monthAgo = new Date(today);
      monthAgo.setDate(monthAgo.getDate() - 30);
      filters.value.start_date = formatDate(monthAgo);
      filters.value.end_date = formatDate(today);
      break;
    default:
      filters.value.start_date = '';
      filters.value.end_date = '';
  }

  performSearch();
};

const highlightQuery = (text) => {
  if (!localQuery.value) return text;

  const escapedQuery = localQuery.value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(`(${escapedQuery})`, 'gi');
  return text.replace(regex, '<mark>$1</mark>');
};

const getInitials = (name) => {
  if (!name) return '?';
  return name.split(' ').map(n => n[0]).join('').toUpperCase().slice(0, 2);
};

const formatTime = (timestamp) => {
  // Backend sends timestamp as Unix timestamp (seconds since epoch)
  const date = typeof timestamp === 'number'
    ? new Date(timestamp * 1000)
    : new Date(timestamp);

  if (isNaN(date.getTime())) {
    return 'Unknown time';
  }

  const now = new Date();
  const diffInHours = (now - date) / (1000 * 60 * 60);

  if (diffInHours < 24) {
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      hour12: true
    });
  } else {
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined
    });
  }
};

// Computed
const hasActiveFilters = computed(() => {
  return filters.value.sender_id ||
    filters.value.start_date ||
    filters.value.end_date ||
    filters.value.has_files ||
    filters.value.sort_by !== 'relevance';
});

// Watch for modal open
watch(() => props.modelValue, (modelValue) => {
  if (modelValue) {
    nextTick(() => {
      searchInput.value?.focus();
    });
  }
});

// Keyboard shortcut
const handleKeyDown = (event) => {
  if (event.key === 'Escape' && props.modelValue) {
    close();
  }
};

onMounted(async () => {
  // Ensure auth is initialized
  const authStore = useAuthStore();
  if (!authStore.isInitialized && authStore.isAuthenticated) {
    try {
      await authStore.initialize();
    } catch (error) {
      console.error('Failed to initialize auth:', error);
    }
  }

  document.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown);
  clearTimeout(searchTimer);
});
</script>

<style scoped>
/* Modal Overlay */
.search-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

/* Modal Container */
.search-modal {
  background: white;
  border-radius: 12px;
  width: 100%;
  max-width: 800px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
}

/* Header */
.search-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid #e1e5e9;
}

.search-header h2 {
  font-size: 20px;
  font-weight: 700;
  color: #1d1c1d;
  margin: 0;
}

.close-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  color: #616061;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
}

.close-btn:hover {
  background: #f8f9fa;
  color: #1d1c1d;
}

/* Search Input Section */
.search-input-section {
  padding: 16px 24px;
  display: flex;
  gap: 12px;
  align-items: center;
}

.search-input-wrapper {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  color: #616061;
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 10px 40px 10px 40px;
  border: 1px solid #d1d9e0;
  border-radius: 6px;
  font-size: 15px;
  color: #1d1c1d;
  transition: all 0.2s;
}

.search-input:focus {
  outline: none;
  border-color: #4a154b;
  box-shadow: 0 0 0 1px #4a154b;
}

.clear-btn {
  position: absolute;
  right: 8px;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  color: #616061;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.clear-btn:hover {
  background: #f8f9fa;
  color: #1d1c1d;
}

.filters-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border: 1px solid #d1d9e0;
  border-radius: 6px;
  background: white;
  color: #616061;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.filters-toggle:hover {
  background: #f8f9fa;
  border-color: #616061;
  color: #1d1c1d;
}

.filters-toggle.active {
  background: #4a154b;
  border-color: #4a154b;
  color: white;
}

/* Filters Section */
.filters-section {
  padding: 16px 24px;
  background: #f8f9fa;
  border-bottom: 1px solid #e1e5e9;
}

.filters-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 16px;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.filter-group label {
  font-size: 13px;
  font-weight: 600;
  color: #1d1c1d;
}

.filter-group select {
  padding: 6px 10px;
  border: 1px solid #d1d9e0;
  border-radius: 4px;
  font-size: 13px;
  color: #1d1c1d;
  background: white;
}

.checkbox-group {
  flex-direction: row;
  align-items: center;
}

.checkbox-group label {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-weight: normal;
}

/* Results Section */
.results-section {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.results-header {
  padding: 12px 24px;
  background: #f8f9fa;
  border-bottom: 1px solid #e1e5e9;
  font-size: 13px;
  font-weight: 600;
  color: #616061;
}

.messages-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

/* Message Item */
.search-message-item {
  display: flex;
  gap: 12px;
  padding: 12px 16px;
  margin-bottom: 4px;
  background: white;
  border: 1px solid transparent;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.search-message-item:hover {
  background: #f8f9fa;
  border-color: #e1e5e9;
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.message-avatar {
  width: 36px;
  height: 36px;
  background: linear-gradient(135deg, #4a154b, #611f69);
  color: white;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 600;
  flex-shrink: 0;
}

.message-content {
  flex: 1;
  min-width: 0;
}

.message-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.sender-name {
  font-size: 14px;
  font-weight: 700;
  color: #1d1c1d;
}

.message-time {
  font-size: 12px;
  color: #616061;
}

.message-text {
  font-size: 14px;
  color: #1d1c1d;
  line-height: 1.4;
  margin-bottom: 4px;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.message-text mark {
  background: #fff3cd;
  color: inherit;
  padding: 0 2px;
  border-radius: 2px;
}

.message-files {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #616061;
}

.message-action {
  display: flex;
  align-items: center;
  color: #616061;
  opacity: 0;
  transition: opacity 0.2s;
}

.search-message-item:hover .message-action {
  opacity: 1;
}

/* States */
.loading-state,
.no-results,
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  text-align: center;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid #f1f2f6;
  border-top-color: #4a154b;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 16px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.loading-state p,
.no-results p,
.empty-state p {
  color: #616061;
  font-size: 14px;
  margin-top: 8px;
}

.no-results h3,
.empty-state h3 {
  font-size: 18px;
  font-weight: 700;
  color: #1d1c1d;
  margin: 16px 0 8px;
}

.no-results svg,
.empty-state svg {
  color: #d1d9e0;
}

/* Scrollbar */
.messages-list::-webkit-scrollbar {
  width: 8px;
}

.messages-list::-webkit-scrollbar-track {
  background: #f8f9fa;
}

.messages-list::-webkit-scrollbar-thumb {
  background: #d1d9e0;
  border-radius: 4px;
}

.messages-list::-webkit-scrollbar-thumb:hover {
  background: #616061;
}

/* Animations */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s;
}

.modal-enter-active .search-modal,
.modal-leave-active .search-modal {
  transition: transform 0.3s, opacity 0.3s;
}

.modal-enter-from {
  opacity: 0;
}

.modal-enter-from .search-modal {
  transform: scale(0.9);
  opacity: 0;
}

.modal-leave-to {
  opacity: 0;
}

.modal-leave-to .search-modal {
  transform: scale(0.9);
  opacity: 0;
}

.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.2s ease;
}

.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

/* Responsive */
@media (max-width: 768px) {
  .search-modal {
    max-width: 100%;
    margin: 10px;
  }

  .filters-grid {
    grid-template-columns: 1fr 1fr;
  }

  .search-input-section {
    flex-direction: column;
    align-items: stretch;
  }

  .filters-toggle {
    width: 100%;
    justify-content: center;
  }
}
</style>