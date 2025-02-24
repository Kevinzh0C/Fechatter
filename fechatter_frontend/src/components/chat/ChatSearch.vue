<template>
  <div class="chat-search">
    <!-- Search Bar -->
    <div class="search-container">
      <div class="search-input-wrapper">
        <svg class="search-icon" width="20" height="20" viewBox="0 0 24 24" fill="none">
          <path d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round" />
        </svg>

        <input ref="searchInput" v-model="searchQuery" @input="handleSearchInput" @keydown="handleKeydown" type="text"
          :placeholder="`Search in ${chatName || 'this channel'}...`" class="search-input" />

        <div class="search-actions">
          <!-- Clear button -->
          <button v-if="searchQuery" @click="clearSearch" class="action-btn" title="Clear search">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path
                d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
            </svg>
          </button>

          <!-- Advanced search toggle -->
          <button @click="toggleAdvanced" class="action-btn" :class="{ active: showAdvanced }" title="Advanced search">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path
                d="M3 17v2h6v-2H3zM3 5v2h10V5H3zm10 16v-2h8v-2h-8v-2h-2v6h2zM7 9v2H3v2h4v2h2V9H7zm14 4v-2H11v2h10zm-6-4h2V7h4V5h-4V3h-2v6z" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Search Stats -->
      <div v-if="searchActive" class="search-stats">
        <span v-if="loading" class="stats-loading">
          <svg class="animate-spin" width="14" height="14" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" stroke-dasharray="40 20" />
          </svg>
          Searching...
        </span>
        <span v-else-if="results.length > 0" class="stats-count">
          {{ currentResult + 1 }} of {{ results.length }} results
        </span>
        <span v-else-if="searchPerformed" class="stats-empty">
          No results found
        </span>

        <!-- Navigation buttons -->
        <div v-if="results.length > 1" class="search-nav">
          <button @click="navigatePrevious" :disabled="currentResult === 0" class="nav-btn"
            title="Previous result (Shift+Enter)">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z" />
            </svg>
          </button>
          <button @click="navigateNext" :disabled="currentResult === results.length - 1" class="nav-btn"
            title="Next result (Enter)">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M8.59 16.59L10 18l6-6-6-6-1.41 1.41L14.17 12z" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Advanced Search Panel -->
    <Transition name="slide-down">
      <div v-if="showAdvanced" class="advanced-panel">
        <div class="advanced-grid">
          <!-- From User -->
          <div class="filter-group">
            <label class="filter-label">From:</label>
            <select v-model="filters.sender_id" @change="performSearch" class="filter-select">
              <option value="">Anyone</option>
              <option v-for="member in chatMembers" :key="member.id" :value="member.id">
                {{ member.fullname || member.username || 'Unknown' }}
              </option>
            </select>
          </div>

          <!-- Date Range -->
          <div class="filter-group">
            <label class="filter-label">When:</label>
            <select v-model="datePreset" @change="handleDatePresetChange" class="filter-select">
              <option value="">Any time</option>
              <option value="today">Today</option>
              <option value="yesterday">Yesterday</option>
              <option value="week">Past 7 days</option>
              <option value="month">Past 30 days</option>
              <option value="custom">Custom range</option>
            </select>
          </div>

          <!-- Custom Date Range -->
          <div v-if="datePreset === 'custom'" class="filter-group date-range">
            <input type="date" v-model="filters.start_date" @change="performSearch" class="filter-input" max="today" />
            <span class="date-separator">to</span>
            <input type="date" v-model="filters.end_date" @change="performSearch" class="filter-input" max="today" />
          </div>

          <!-- Has Attachments -->
          <div class="filter-group">
            <label class="filter-checkbox">
              <input type="checkbox" v-model="filters.has_files" @change="performSearch" />
              <span>Has files</span>
            </label>
          </div>

          <!-- Sort Order -->
          <div class="filter-group">
            <label class="filter-label">Sort:</label>
            <select v-model="filters.sort" @change="performSearch" class="filter-select">
              <option value="relevance">Most relevant</option>
              <option value="date_desc">Newest first</option>
              <option value="date_asc">Oldest first</option>
            </select>
          </div>
        </div>

        <!-- Quick Actions -->
        <div class="advanced-actions">
          <button @click="resetFilters" class="reset-btn">
            Reset filters
          </button>
          <button @click="exportResults" v-if="results.length > 0" class="export-btn">
            Export results
          </button>
        </div>
      </div>
    </Transition>

    <!-- Highlighted Results in Messages -->
    <div v-if="searchActive && results.length > 0" class="search-highlights">
      <!-- This is handled by the parent component -->
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import { useChatStore } from '@/stores/chat';
import { useNotifications } from '@/composables/useNotifications';
import api from '@/services/api';

const props = defineProps({
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

const emit = defineEmits([
  'search-results',
  'highlight-message',
  'clear-highlights',
  'navigate-to-message'
]);

// Refs
const searchInput = ref(null);
const searchQuery = ref('');
const showAdvanced = ref(false);
const loading = ref(false);
const searchPerformed = ref(false);
const results = ref([]);
const currentResult = ref(-1);
const searchActive = ref(false);
const datePreset = ref('');

// Filters
const filters = ref({
  sender_id: '',
  start_date: '',
  end_date: '',
  has_files: false,
  sort: 'relevance'
});

// Debounce timer
let searchTimer = null;

// Methods
const handleSearchInput = () => {
  clearTimeout(searchTimer);

  if (!searchQuery.value.trim()) {
    clearSearchResults();
    return;
  }

  searchTimer = setTimeout(() => {
    performSearch();
  }, 300); // 300ms debounce
};

const performSearch = async () => {
  const query = searchQuery.value.trim();
  if (!query && !hasActiveFilters.value) {
    clearSearchResults();
    return;
  }

  loading.value = true;
  searchPerformed.value = true;
  searchActive.value = true;

  try {
    // Build search parameters
    const searchParams = {
      query: query || '*', // Use wildcard if only filters
      chatId: props.chatId,
      limit: 100, // Get more results for in-channel search
      ...filters.value
    };

    // Clean up empty values
    Object.keys(searchParams).forEach(key => {
      if (searchParams[key] === '' || searchParams[key] === null) {
        delete searchParams[key];
      }
    });

    const response = await SearchService.search(searchParams);

    // Handle response data format
    results.value = response.results || [];
    currentResult.value = results.value.length > 0 ? 0 : -1;

    // Emit results to parent
    emit('search-results', {
      query,
      results: results.value,
      filters: filters.value
    });

    // Highlight first result
    if (results.value.length > 0) {
      highlightCurrentResult();
    }

  } catch (error) {
    console.error('Search failed:', error);
    const { notifyError } = useNotifications();
    notifyError('Search failed. Please try again.');
    results.value = [];
  } finally {
    loading.value = false;
  }
};

const highlightCurrentResult = () => {
  if (currentResult.value >= 0 && currentResult.value < results.value.length) {
    const result = results.value[currentResult.value];
    emit('navigate-to-message', {
      messageId: result.id,
      highlight: true
    });
  }
};

const navigateNext = () => {
  if (currentResult.value < results.value.length - 1) {
    currentResult.value++;
    highlightCurrentResult();
  }
};

const navigatePrevious = () => {
  if (currentResult.value > 0) {
    currentResult.value--;
    highlightCurrentResult();
  }
};

const handleKeydown = (event) => {
  if (event.key === 'Enter') {
    if (event.shiftKey) {
      event.preventDefault();
      navigatePrevious();
    } else {
      event.preventDefault();
      navigateNext();
    }
  } else if (event.key === 'Escape') {
    clearSearch();
  }
};

const clearSearch = () => {
  searchQuery.value = '';
  clearSearchResults();
  searchInput.value?.focus();
};

const clearSearchResults = () => {
  results.value = [];
  currentResult.value = -1;
  searchPerformed.value = false;
  searchActive.value = false;
  emit('clear-highlights');
};

const toggleAdvanced = () => {
  showAdvanced.value = !showAdvanced.value;
  if (!showAdvanced.value) {
    // Optionally reset filters when closing
    // resetFilters();
  }
};

const resetFilters = () => {
  filters.value = {
    sender_id: '',
    start_date: '',
    end_date: '',
    has_files: false,
    sort: 'relevance'
  };
  datePreset.value = '';
  if (searchQuery.value) {
    performSearch();
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
    case 'custom':
      // Don't change dates, let user pick
      return;
    default:
      filters.value.start_date = '';
      filters.value.end_date = '';
  }

  performSearch();
};

const exportResults = async () => {
  if (results.value.length === 0) return;

  try {
    const { notifyInfo } = useNotifications();
    notifyInfo('Preparing export...');

    // Format results for export
    const exportData = results.value.map(r => ({
      date: new Date(r.created_at).toLocaleString(),
      sender: r.sender_name || r.sender?.fullname || 'Unknown',
      message: r.content,
      hasFiles: r.files?.length > 0 ? 'Yes' : 'No'
    }));

    // Convert to CSV
    const csv = [
      ['Date', 'Sender', 'Message', 'Has Files'],
      ...exportData.map(row => [
        row.date,
        row.sender,
        `"${row.message.replace(/"/g, '""')}"`,
        row.hasFiles
      ])
    ].map(row => row.join(',')).join('\n');

    // Download
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `search-results-${props.chatName}-${new Date().toISOString().split('T')[0]}.csv`;
    a.click();
    URL.revokeObjectURL(url);

    const { notifySuccess } = useNotifications();
    notifySuccess('Search results exported successfully');
  } catch (error) {
    console.error('Export failed:', error);
    const { notifyError } = useNotifications();
    notifyError('Failed to export results');
  }
};

// Computed
const hasActiveFilters = computed(() => {
  return filters.value.sender_id ||
    filters.value.start_date ||
    filters.value.end_date ||
    filters.value.has_files ||
    filters.value.sort !== 'relevance';
});

// Focus management
const focusSearch = () => {
  nextTick(() => {
    searchInput.value?.focus();
  });
};

// Keyboard shortcut
const handleGlobalKeydown = (event) => {
  // Ctrl/Cmd + F to focus search
  if ((event.ctrlKey || event.metaKey) && event.key === 'f') {
    event.preventDefault();
    focusSearch();
  }
};

// Lifecycle
onMounted(() => {
  document.addEventListener('keydown', handleGlobalKeydown);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleGlobalKeydown);
  clearTimeout(searchTimer);
});

// Expose methods for parent component
defineExpose({
  focusSearch,
  clearSearch,
  performSearch
});

// Simple SearchService implementation
const SearchService = {
  async search(params) {
    try {
      const response = await api.get('/search/messages', { params });
      return response.data || { results: [], total: 0 };
    } catch (error) {
      console.error('Search error:', error);
      return { results: [], total: 0 };
    }
  }
};
</script>

<style scoped>
.chat-search {
  position: relative;
  border-bottom: 1px solid #e1e5e9;
  background: #f8f9fa;
}

.search-container {
  padding: 12px 16px;
}

.search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  background: white;
  border: 1px solid #d1d9e0;
  border-radius: 6px;
  transition: all 0.2s;
}

.search-input-wrapper:focus-within {
  border-color: #4a154b;
  box-shadow: 0 0 0 1px #4a154b;
}

.search-icon {
  position: absolute;
  left: 12px;
  color: #616061;
  pointer-events: none;
}

.search-input {
  flex: 1;
  padding: 8px 12px 8px 40px;
  border: none;
  background: none;
  font-size: 14px;
  color: #1d1c1d;
  outline: none;
}

.search-input::placeholder {
  color: #616061;
}

.search-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  padding-right: 8px;
}

.action-btn {
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

.action-btn:hover {
  background: #f8f9fa;
  color: #1d1c1d;
}

.action-btn.active {
  background: #4a154b;
  color: white;
}

/* Search Stats */
.search-stats {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 8px;
  padding: 4px 8px;
  background: rgba(74, 21, 75, 0.05);
  border-radius: 4px;
  font-size: 13px;
  color: #616061;
}

.stats-loading {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #4a154b;
}

.stats-count {
  color: #1d1c1d;
  font-weight: 500;
}

.stats-empty {
  color: #616061;
}

.search-nav {
  display: flex;
  align-items: center;
  gap: 4px;
}

.nav-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: white;
  color: #616061;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.nav-btn:hover:not(:disabled) {
  background: #f8f9fa;
  color: #1d1c1d;
}

.nav-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Advanced Panel */
.advanced-panel {
  padding: 16px;
  background: white;
  border-top: 1px solid #e1e5e9;
}

.advanced-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
  margin-bottom: 16px;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.filter-label {
  font-size: 13px;
  font-weight: 600;
  color: #1d1c1d;
}

.filter-select,
.filter-input {
  padding: 6px 10px;
  border: 1px solid #d1d9e0;
  border-radius: 4px;
  font-size: 13px;
  color: #1d1c1d;
  background: white;
  transition: all 0.2s;
}

.filter-select:focus,
.filter-input:focus {
  outline: none;
  border-color: #4a154b;
  box-shadow: 0 0 0 1px #4a154b;
}

.date-range {
  grid-column: span 2;
  flex-direction: row;
  align-items: center;
  gap: 8px;
}

.date-separator {
  font-size: 13px;
  color: #616061;
}

.filter-checkbox {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: #1d1c1d;
  cursor: pointer;
  user-select: none;
}

.filter-checkbox input {
  width: 16px;
  height: 16px;
  cursor: pointer;
}

.advanced-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 12px;
  border-top: 1px solid #e1e5e9;
}

.reset-btn,
.export-btn {
  padding: 6px 12px;
  border: 1px solid #d1d9e0;
  border-radius: 4px;
  background: white;
  font-size: 13px;
  font-weight: 500;
  color: #1d1c1d;
  cursor: pointer;
  transition: all 0.2s;
}

.reset-btn:hover {
  background: #f8f9fa;
  border-color: #616061;
}

.export-btn {
  background: #4a154b;
  color: white;
  border-color: #4a154b;
}

.export-btn:hover {
  background: #3f0f40;
}

/* Animations */
.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.2s ease;
}

.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

/* Loading animation */
@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

.animate-spin {
  animation: spin 1s linear infinite;
}

/* Responsive */
@media (max-width: 768px) {
  .advanced-grid {
    grid-template-columns: 1fr;
  }

  .date-range {
    grid-column: span 1;
    flex-direction: column;
    align-items: stretch;
  }

  .date-separator {
    text-align: center;
  }
}
</style>