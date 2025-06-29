<template>
  <!-- Advanced Search Interface - Human-Centered Design -->
  <div class="advanced-search-interface" :class="{ 'interface-active': isSearchActive }">
    <!-- Search Input Section - Golden Ratio Layout -->
    <div class="search-input-section">
      <div class="search-container" ref="searchContainer">
        <!-- Main Search Input - Ergonomic Design -->
        <div class="search-input-wrapper">
          <div class="search-icon-wrapper">
            <svg class="search-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>
          
          <input
            ref="searchInput"
            v-model="searchQuery"
            type="text"
            class="search-input"
            :placeholder="intelligentPlaceholder"
            @input="handleSearchInput"
            @focus="handleFocus"
            @blur="handleBlur"
            @keydown="handleKeydown"
            autocomplete="off"
            spellcheck="false"
          />
          
          <!-- Strategy Indicator -->
          <div class="strategy-indicator" v-if="currentStrategy">
            <span class="strategy-badge">{{ strategyLabels[currentStrategy] }}</span>
          </div>
          
          <!-- Clear Button - Fitts' Law Optimized -->
          <button
            v-if="searchQuery"
            @click="clearSearch"
            class="clear-button"
            type="button"
          >
            <svg class="clear-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        
        <!-- Advanced Filters Toggle - Cognitive Load Optimized -->
        <button
          @click="toggleAdvancedFilters"
          class="filters-toggle"
          :class="{ 'active': showAdvancedFilters }"
          type="button"
        >
          <svg class="filter-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
          </svg>
          <span>Filters</span>
        </button>
      </div>
      
      <!-- Search Suggestions - Progressive Disclosure -->
      <transition name="suggestions">
        <div v-if="showSuggestions && suggestions.length > 0" class="suggestions-container">
          <div class="suggestions-header">
            <span>{{ $t('search.suggestions') }}</span>
          </div>
          <ul class="suggestions-list" role="listbox">
            <li
              v-for="(suggestion, index) in suggestions"
              :key="suggestion.id || index"
              @click="selectSuggestion(suggestion)"
              class="suggestion-item"
              :class="{ 'highlighted': index === selectedSuggestionIndex }"
              role="option"
            >
              <div class="suggestion-content">
                <span class="suggestion-text" v-html="highlightSuggestion(suggestion.text)"></span>
                <span class="suggestion-type">{{ suggestion.type }}</span>
              </div>
            </li>
          </ul>
        </div>
      </transition>
    </div>
    
    <!-- Advanced Filters Panel - Information Architecture -->
    <transition name="filters-panel">
      <div v-if="showAdvancedFilters" class="filters-panel">
        <div class="filters-grid">
          <!-- Time Range Filter -->
          <div class="filter-group">
            <label class="filter-label">{{ $t('search.timeRange') }}</label>
            <select v-model="filters.timeRange" class="filter-select">
              <option value="all">{{ $t('search.allTime') }}</option>
              <option value="today">{{ $t('search.today') }}</option>
              <option value="week">{{ $t('search.thisWeek') }}</option>
              <option value="month">{{ $t('search.thisMonth') }}</option>
            </select>
          </div>
          
          <!-- User Filter -->
          <div class="filter-group">
            <label class="filter-label">{{ $t('search.sender') }}</label>
            <input
              v-model="filters.sender"
              type="text"
              class="filter-input"
              :placeholder="$t('search.senderPlaceholder')"
            />
          </div>
          
          <!-- Content Type Filter -->
          <div class="filter-group">
            <label class="filter-label">{{ $t('search.contentType') }}</label>
            <div class="checkbox-group">
              <label class="checkbox-item">
                <input v-model="filters.hasFiles" type="checkbox" />
                <span>{{ $t('search.hasFiles') }}</span>
              </label>
              <label class="checkbox-item">
                <input v-model="filters.hasLinks" type="checkbox" />
                <span>{{ $t('search.hasLinks') }}</span>
              </label>
            </div>
          </div>
          
          <!-- Search Strategy Selection -->
          <div class="filter-group">
            <label class="filter-label">{{ $t('search.strategy') }}</label>
            <div class="strategy-buttons">
              <button
                v-for="strategy in availableStrategies"
                :key="strategy"
                @click="setSearchStrategy(strategy)"
                class="strategy-button"
                :class="{ 'active': selectedStrategies.includes(strategy) }"
                type="button"
              >
                {{ strategyLabels[strategy] }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </transition>
    
    <!-- Search Results Area - Visual Hierarchy -->
    <div class="results-area" v-if="isSearchActive">
      <!-- Results Header - Information Scent -->
      <div class="results-header">
        <div class="results-meta">
          <span class="results-count">
            {{ formatResultsCount(searchResults.total) }}
          </span>
          <span class="search-time" v-if="searchResults.took_ms">
            {{ searchResults.took_ms }}ms
          </span>
          <div class="strategies-used" v-if="searchResults.strategies_used">
            <span class="strategies-label">Strategies:</span>
            <span
              v-for="strategy in searchResults.strategies_used"
              :key="strategy"
              class="strategy-tag"
            >
              {{ strategyLabels[strategy] }}
            </span>
          </div>
        </div>
        
        <!-- Sort Options - Cognitive Mapping -->
        <div class="sort-options">
          <select v-model="sortBy" @change="handleSortChange" class="sort-select">
            <option value="relevance">{{ $t('search.sortByRelevance') }}</option>
            <option value="date">{{ $t('search.sortByDate') }}</option>
            <option value="sender">{{ $t('search.sortBySender') }}</option>
          </select>
        </div>
      </div>
      
      <!-- Search Results List - Scan Patterns Optimized -->
      <div class="results-container" ref="resultsContainer">
        <transition-group name="result-item" tag="div" class="results-list">
          <SearchResultItem
            v-for="result in searchResults.hits"
            :key="result.id"
            :result="result"
            :query="searchQuery"
            @navigate="handleResultNavigation"
            @preview="handleResultPreview"
          />
        </transition-group>
        
        <!-- Load More Button - Fitts' Law Optimized -->
        <div v-if="hasMoreResults" class="load-more-section">
          <button
            @click="loadMoreResults"
            class="load-more-button"
            :disabled="isLoadingMore"
            type="button"
          >
            <LoadingIcon v-if="isLoadingMore" class="loading-icon" />
            <span>{{ $t('search.loadMore') }}</span>
          </button>
        </div>
        
        <!-- Empty State - Emotional Design -->
        <div v-if="showEmptyState" class="empty-state">
          <div class="empty-icon">
            <svg class="empty-search-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>
          <h3 class="empty-title">No results found</h3>
          <p class="empty-description">Try a different search term</p>
        </div>
      </div>
    </div>
    
    <!-- Search Performance Monitor - Progressive Enhancement -->
    <div v-if="showPerformanceMonitor && isDev" class="performance-monitor">
      <div class="performance-stats">
        <div class="stat-item">
          <span class="stat-label">{{ $t('search.cacheHitRate') }}</span>
          <span class="stat-value">{{ formatPercentage(performanceStats.cacheHitRate) }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">{{ $t('search.avgResponseTime') }}</span>
          <span class="stat-value">{{ formatTime(performanceStats.avgResponseTime) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import { useDebounceFn } from '@vueuse/core';

// Icons
import SearchIcon from '@/components/icons/SearchIcon.vue';
import FilterIcon from '@/components/icons/FilterIcon.vue';
import XIcon from '@/components/icons/XIcon.vue';
import LoadingIcon from '@/components/icons/LoadingIcon.vue';

// Components
import SearchResultItem from './SearchResultItem.vue';

// Services
import searchService from '@/services/searchService';

// Composables
import { useKeyboardNavigation } from '@/composables/useKeyboardNavigation';
import { useSearchHistory } from '@/composables/useSearchHistory';

const { t } = useI18n();
const router = useRouter();

// Props
const props = defineProps({
  chatId: {
    type: [String, Number],
    default: null
  },
  initialQuery: {
    type: String,
    default: ''
  },
  autoFocus: {
    type: Boolean,
    default: true
  }
});

// Emits
const emit = defineEmits(['search-complete', 'result-selected', 'close']);

// Refs
const searchInput = ref(null);
const searchContainer = ref(null);
const resultsContainer = ref(null);

// Reactive State
const searchQuery = ref(props.initialQuery);
const isSearchActive = ref(false);
const showAdvancedFilters = ref(false);
const showSuggestions = ref(false);
const selectedSuggestionIndex = ref(-1);
const currentStrategy = ref(null);
const isLoadingMore = ref(false);
const showPerformanceMonitor = ref(false);

// Search Results
const searchResults = ref({
  hits: [],
  total: 0,
  took_ms: 0,
  strategies_used: []
});

// Filters
const filters = ref({
  timeRange: 'all',
  sender: '',
  hasFiles: false,
  hasLinks: false
});

// Search Configuration
const selectedStrategies = ref([]);
const sortBy = ref('relevance');
const suggestions = ref([]);

// Strategy Labels - Internationalization
const strategyLabels = computed(() => ({
  full_text: t('search.strategies.fullText'),
  semantic: t('search.strategies.semantic'),
  fuzzy: t('search.strategies.fuzzy'),
  exact: t('search.strategies.exact'),
  boolean: t('search.strategies.boolean'),
  temporal: t('search.strategies.temporal'),
  user_scoped: t('search.strategies.userScoped'),
  file_content: t('search.strategies.fileContent'),
  fallback_local: t('search.strategies.fallback')
}));

// Available Strategies
const availableStrategies = ref([
  'full_text', 'semantic', 'fuzzy', 'exact', 'boolean', 'temporal'
]);

// Computed Properties
const intelligentPlaceholder = computed(() => {
  const hour = new Date().getHours();
  if (hour < 12) return t('search.placeholders.morning');
  if (hour < 18) return t('search.placeholders.afternoon');
  return t('search.placeholders.evening');
});

const hasMoreResults = computed(() => {
  return searchResults.value.total > searchResults.value.hits.length;
});

const showEmptyState = computed(() => {
  return isSearchActive.value && 
         searchQuery.value.length > 0 && 
         searchResults.value.hits.length === 0;
});

const emptySuggestions = computed(() => {
  return ['messages', 'files', 'today', 'last week'];
});

const performanceStats = computed(() => {
  return searchService.getMetrics();
});

const isDev = computed(() => {
  return import.meta.env.DEV;
});

// Debounced Search Function - Cognitive Load Optimization
const debouncedSearch = useDebounceFn(async () => {
  if (!searchQuery.value.trim()) {
    resetSearch();
    return;
  }

  isSearchActive.value = true;
  
  try {
    const searchParams = {
      query: searchQuery.value,
      chatId: props.chatId,
      strategies: selectedStrategies.value.length > 0 ? selectedStrategies.value : null,
      filters: filters.value,
      limit: 20,
      offset: 0
    };

    const results = await searchService.intelligentSearch(searchParams);
    
    searchResults.value = results;
    currentStrategy.value = results.strategies_used?.[0] || null;
    
    emit('search-complete', results);
    
    // Add to search history
    if (results.hits.length > 0) {
      useSearchHistory().addSearch(searchQuery.value, results.total);
    }
    
  } catch (error) {
    console.error('Search failed:', error);
    searchResults.value = { hits: [], total: 0, took_ms: 0, strategies_used: [] };
  }
}, 300); // Optimal debounce time based on human perception

// Event Handlers
const handleSearchInput = () => {
  if (searchQuery.value.length >= 2) {
    debouncedSearch();
    loadSuggestions();
  } else {
    resetSearch();
  }
};

const handleFocus = () => {
  showSuggestions.value = true;
  if (searchQuery.value.length >= 2) {
    loadSuggestions();
  }
};

const handleBlur = () => {
  // Delay hiding suggestions to allow clicks
  setTimeout(() => {
    showSuggestions.value = false;
  }, 200);
};

const handleKeydown = (event) => {
  if (event.key === 'Escape') {
    clearSearch();
    emit('close');
  } else if (event.key === 'Enter') {
    if (selectedSuggestionIndex.value >= 0) {
      selectSuggestion(suggestions.value[selectedSuggestionIndex.value]);
    }
  } else if (event.key === 'ArrowDown') {
    event.preventDefault();
    navigateSuggestions(1);
  } else if (event.key === 'ArrowUp') {
    event.preventDefault();
    navigateSuggestions(-1);
  }
};

const navigateSuggestions = (direction) => {
  const maxIndex = suggestions.value.length - 1;
  selectedSuggestionIndex.value = Math.max(-1, 
    Math.min(maxIndex, selectedSuggestionIndex.value + direction)
  );
};

const selectSuggestion = (suggestion) => {
  searchQuery.value = suggestion.text;
  showSuggestions.value = false;
  selectedSuggestionIndex.value = -1;
  debouncedSearch();
};

const clearSearch = () => {
  searchQuery.value = '';
  resetSearch();
  searchInput.value?.focus();
};

const resetSearch = () => {
  isSearchActive.value = false;
  searchResults.value = { hits: [], total: 0, took_ms: 0, strategies_used: [] };
  suggestions.value = [];
  currentStrategy.value = null;
};

const toggleAdvancedFilters = () => {
  showAdvancedFilters.value = !showAdvancedFilters.value;
};

const setSearchStrategy = (strategy) => {
  const index = selectedStrategies.value.indexOf(strategy);
  if (index >= 0) {
    selectedStrategies.value.splice(index, 1);
  } else {
    selectedStrategies.value.push(strategy);
  }
  
  if (searchQuery.value.trim()) {
    debouncedSearch();
  }
};

const handleSortChange = () => {
  if (searchResults.value.hits.length > 0) {
    debouncedSearch();
  }
};

const loadMoreResults = async () => {
  if (isLoadingMore.value) return;
  
  isLoadingMore.value = true;
  
  try {
    const searchParams = {
      query: searchQuery.value,
      chatId: props.chatId,
      strategies: selectedStrategies.value.length > 0 ? selectedStrategies.value : null,
      filters: filters.value,
      limit: 20,
      offset: searchResults.value.hits.length
    };

    const results = await searchService.intelligentSearch(searchParams);
    
    searchResults.value.hits.push(...results.hits);
    
  } catch (error) {
    console.error('Load more failed:', error);
  } finally {
    isLoadingMore.value = false;
  }
};

const loadSuggestions = async () => {
  // Simple local suggestions for now
  if (searchQuery.value.length >= 2) {
    suggestions.value = [
      { text: searchQuery.value, type: 'exact' },
      { text: `"${searchQuery.value}"`, type: 'phrase' },
      { text: `${searchQuery.value}*`, type: 'wildcard' }
    ];
  }
};

const highlightSuggestion = (text) => {
  if (!searchQuery.value) return text;
  const regex = new RegExp(`(${searchQuery.value})`, 'gi');
  return text.replace(regex, '<strong>$1</strong>');
};

const handleResultNavigation = (result) => {
  emit('result-selected', result);
  
  // Navigate to chat if not already there
  if (props.chatId !== result.chat_id) {
    router.push(`/chat/${result.chat_id}?messageId=${result.id}`);
  }
};

const handleResultPreview = (result) => {
  // Show result preview modal
  console.log('Preview result:', result);
};

const applySuggestion = (suggestion) => {
  searchQuery.value = suggestion;
  debouncedSearch();
};

// Utility Functions
const formatResultsCount = (count) => {
  if (count === 0) return t('search.noResults');
  if (count === 1) return t('search.oneResult');
  return t('search.multipleResults', { count });
};

const formatPercentage = (value) => {
  return `${Math.round(value * 100)}%`;
};

const formatTime = (ms) => {
  return `${Math.round(ms)}ms`;
};

// Lifecycle
onMounted(() => {
  if (props.autoFocus) {
    nextTick(() => {
      searchInput.value?.focus();
    });
  }
  
  // Initialize with initial query if provided
  if (props.initialQuery) {
    debouncedSearch();
  }
});

// Watchers
watch(() => props.chatId, () => {
  if (searchQuery.value.trim()) {
    debouncedSearch();
  }
});

// Expose methods for parent components
defineExpose({
  focus: () => searchInput.value?.focus(),
  clear: clearSearch,
  search: (query) => {
    searchQuery.value = query;
    debouncedSearch();
  }
});
</script>

<style scoped>
/* Advanced Search Interface Styles - Human-Centered Design */

.advanced-search-interface {
  /* Golden Ratio Based Layout */
  --golden-ratio: 1.618;
  --base-unit: 16px;
  --spacing-xs: calc(var(--base-unit) / var(--golden-ratio) / var(--golden-ratio)); /* ~6px */
  --spacing-sm: calc(var(--base-unit) / var(--golden-ratio)); /* ~10px */
  --spacing-md: var(--base-unit); /* 16px */
  --spacing-lg: calc(var(--base-unit) * var(--golden-ratio)); /* ~26px */
  --spacing-xl: calc(var(--base-unit) * var(--golden-ratio) * var(--golden-ratio)); /* ~42px */
  
  /* Ergonomic Touch Targets - Fitts' Law */
  --touch-target-min: 44px; /* iOS/Android minimum */
  --touch-target-optimal: 48px; /* Optimal size */
  
  /* Color Psychology */
  --color-primary: #3b82f6; /* Trust, stability */
  --color-success: #10b981; /* Success, growth */
  --color-warning: #f59e0b; /* Attention, caution */
  --color-error: #ef4444; /* Error, urgency */
  --color-neutral-50: #f9fafb;
  --color-neutral-100: #f3f4f6;
  --color-neutral-200: #e5e7eb;
  --color-neutral-300: #d1d5db;
  --color-neutral-400: #9ca3af;
  --color-neutral-500: #6b7280;
  --color-neutral-600: #4b5563;
  --color-neutral-700: #374151;
  --color-neutral-800: #1f2937;
  --color-neutral-900: #111827;
  
  width: 100%;
  max-width: 1200px; /* Golden ratio applied to max width */
  margin: 0 auto;
  background: var(--color-neutral-50);
  border-radius: calc(var(--spacing-md) / 2);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.interface-active {
  background: white;
  box-shadow: 
    0 4px 6px -1px rgba(0, 0, 0, 0.1),
    0 2px 4px -1px rgba(0, 0, 0, 0.06);
}

/* Search Input Section - Ergonomic Design */
.search-input-section {
  padding: var(--spacing-lg);
}

.search-container {
  display: flex;
  gap: var(--spacing-sm);
  align-items: center;
}

.search-input-wrapper {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
  background: white;
  border: 2px solid var(--color-neutral-200);
  border-radius: calc(var(--spacing-xl) / 2); /* Golden ratio based radius */
  transition: all 0.2s ease;
  min-height: var(--touch-target-optimal);
}

.search-input-wrapper:focus-within {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.search-icon-wrapper {
  padding: 0 var(--spacing-md);
  display: flex;
  align-items: center;
}

.search-icon {
  width: 20px;
  height: 20px;
  color: var(--color-neutral-400);
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  font-size: 16px; /* Prevents zoom on iOS */
  line-height: 1.5;
  padding: var(--spacing-sm) 0;
  background: transparent;
  color: var(--color-neutral-900);
}

.search-input::placeholder {
  color: var(--color-neutral-400);
}

.strategy-indicator {
  padding: 0 var(--spacing-sm);
}

.strategy-badge {
  display: inline-flex;
  align-items: center;
  padding: calc(var(--spacing-xs)) calc(var(--spacing-sm));
  background: var(--color-primary);
  color: white;
  border-radius: var(--spacing-xs);
  font-size: 12px;
  font-weight: 500;
}

.clear-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--touch-target-min);
  height: var(--touch-target-min);
  border: none;
  background: transparent;
  border-radius: 50%;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.clear-button:hover {
  background: var(--color-neutral-100);
}

.clear-icon {
  width: 16px;
  height: 16px;
  color: var(--color-neutral-400);
}

.filters-toggle {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-sm) var(--spacing-md);
  border: 2px solid var(--color-neutral-200);
  background: white;
  border-radius: calc(var(--spacing-lg) / 2);
  cursor: pointer;
  transition: all 0.2s ease;
  min-height: var(--touch-target-optimal);
  font-weight: 500;
  color: var(--color-neutral-700);
}

.filters-toggle:hover {
  border-color: var(--color-neutral-300);
  background: var(--color-neutral-50);
}

.filters-toggle.active {
  border-color: var(--color-primary);
  background: var(--color-primary);
  color: white;
}

.filter-icon {
  width: 16px;
  height: 16px;
}

/* Suggestions - Progressive Disclosure */
.suggestions {
  transform: translateY(-8px);
  opacity: 0;
}

.suggestions-enter-active,
.suggestions-leave-active {
  transition: all 0.2s ease;
}

.suggestions-enter-to {
  transform: translateY(0);
  opacity: 1;
}

.suggestions-container {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  z-index: 50;
  background: white;
  border: 1px solid var(--color-neutral-200);
  border-radius: calc(var(--spacing-md) / 2);
  box-shadow: 
    0 10px 15px -3px rgba(0, 0, 0, 0.1),
    0 4px 6px -2px rgba(0, 0, 0, 0.05);
  margin-top: var(--spacing-xs);
}

.suggestions-header {
  padding: var(--spacing-sm) var(--spacing-md);
  border-bottom: 1px solid var(--color-neutral-100);
  font-size: 12px;
  font-weight: 600;
  color: var(--color-neutral-600);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.suggestions-list {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 200px;
  overflow-y: auto;
}

.suggestion-item {
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
  transition: background-color 0.15s ease;
  border-bottom: 1px solid var(--color-neutral-50);
}

.suggestion-item:hover,
.suggestion-item.highlighted {
  background: var(--color-neutral-50);
}

.suggestion-item:last-child {
  border-bottom: none;
}

.suggestion-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.suggestion-text {
  font-size: 14px;
  color: var(--color-neutral-900);
}

.suggestion-type {
  font-size: 12px;
  color: var(--color-neutral-500);
  background: var(--color-neutral-100);
  padding: 2px 6px;
  border-radius: 4px;
}

/* Advanced Filters Panel - Information Architecture */
.filters-panel-enter-active,
.filters-panel-leave-active {
  transition: all 0.3s ease;
}

.filters-panel-enter-from,
.filters-panel-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

.filters-panel {
  padding: var(--spacing-lg);
  border-top: 1px solid var(--color-neutral-200);
  background: var(--color-neutral-50);
}

.filters-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: var(--spacing-lg);
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.filter-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-neutral-700);
}

.filter-select,
.filter-input {
  padding: var(--spacing-sm);
  border: 1px solid var(--color-neutral-300);
  border-radius: calc(var(--spacing-sm) / 2);
  font-size: 14px;
  background: white;
  transition: border-color 0.2s ease;
  min-height: var(--touch-target-min);
}

.filter-select:focus,
.filter-input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.checkbox-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.checkbox-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  cursor: pointer;
  font-size: 14px;
  color: var(--color-neutral-700);
  min-height: var(--touch-target-min);
}

.checkbox-item input[type="checkbox"] {
  width: 16px;
  height: 16px;
  cursor: pointer;
}

.strategy-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-xs);
}

.strategy-button {
  padding: var(--spacing-xs) var(--spacing-sm);
  border: 1px solid var(--color-neutral-300);
  background: white;
  border-radius: calc(var(--spacing-sm) / 2);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  min-height: 32px; /* Smaller touch target for secondary actions */
}

.strategy-button:hover {
  border-color: var(--color-primary);
  background: var(--color-primary);
  color: white;
}

.strategy-button.active {
  border-color: var(--color-primary);
  background: var(--color-primary);
  color: white;
}

/* Results Area - Visual Hierarchy */
.results-area {
  border-top: 1px solid var(--color-neutral-200);
}

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-md) var(--spacing-lg);
  background: var(--color-neutral-50);
  border-bottom: 1px solid var(--color-neutral-200);
}

.results-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  flex-wrap: wrap;
}

.results-count {
  font-weight: 600;
  color: var(--color-neutral-900);
}

.search-time {
  font-size: 14px;
  color: var(--color-neutral-500);
}

.strategies-used {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  font-size: 12px;
}

.strategies-label {
  color: var(--color-neutral-600);
}

.strategy-tag {
  background: var(--color-neutral-200);
  color: var(--color-neutral-700);
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: 500;
}

.sort-select {
  padding: var(--spacing-xs) var(--spacing-sm);
  border: 1px solid var(--color-neutral-300);
  border-radius: calc(var(--spacing-sm) / 2);
  font-size: 14px;
  background: white;
  min-height: 32px;
}

/* Results Container - Scan Patterns Optimized */
.results-container {
  max-height: 600px; /* Golden ratio applied */
  overflow-y: auto;
}

.results-list {
  padding: var(--spacing-md);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.result-item-enter-active,
.result-item-leave-active {
  transition: all 0.3s ease;
}

.result-item-enter-from,
.result-item-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}

.result-item-move {
  transition: transform 0.3s ease;
}

/* Load More Section - Fitts' Law Optimized */
.load-more-section {
  display: flex;
  justify-content: center;
  padding: var(--spacing-lg);
}

.load-more-button {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-sm) var(--spacing-lg);
  border: 2px solid var(--color-primary);
  background: white;
  color: var(--color-primary);
  border-radius: calc(var(--spacing-lg) / 2);
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  min-height: var(--touch-target-optimal);
  min-width: 120px;
}

.load-more-button:hover:not(:disabled) {
  background: var(--color-primary);
  color: white;
}

.load-more-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.loading-icon {
  width: 16px;
  height: 16px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Empty State - Emotional Design */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--spacing-xl);
  text-align: center;
}

.empty-icon {
  width: 64px;
  height: 64px;
  background: var(--color-neutral-100);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: var(--spacing-lg);
}

.empty-search-icon {
  width: 32px;
  height: 32px;
  color: var(--color-neutral-400);
}

.empty-title {
  margin: 0 0 var(--spacing-sm) 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-neutral-900);
}

.empty-description {
  margin: 0 0 var(--spacing-lg) 0;
  color: var(--color-neutral-600);
  line-height: 1.5;
}

/* Performance Monitor - Progressive Enhancement */
.performance-monitor {
  padding: var(--spacing-md) var(--spacing-lg);
  border-top: 1px solid var(--color-neutral-200);
  background: var(--color-neutral-50);
  font-size: 12px;
}

.performance-stats {
  display: flex;
  gap: var(--spacing-lg);
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.stat-label {
  color: var(--color-neutral-600);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.stat-value {
  font-weight: 600;
  color: var(--color-neutral-900);
}

/* Responsive Design - Mobile First */
@media (max-width: 768px) {
  .advanced-search-interface {
    --base-unit: 14px; /* Slightly smaller on mobile */
  }
  
  .search-input-section {
    padding: var(--spacing-md);
  }
  
  .search-container {
    flex-direction: column;
    gap: var(--spacing-sm);
  }
  
  .filters-grid {
    grid-template-columns: 1fr;
  }
  
  .results-header {
    flex-direction: column;
    align-items: flex-start;
    gap: var(--spacing-sm);
  }
  
  .results-meta {
    flex-direction: column;
    align-items: flex-start;
    gap: var(--spacing-xs);
  }
  
  .performance-stats {
    flex-direction: column;
    gap: var(--spacing-sm);
  }
}

/* Dark Mode Support */
@media (prefers-color-scheme: dark) {
  .advanced-search-interface {
    --color-neutral-50: #1f2937;
    --color-neutral-100: #374151;
    --color-neutral-200: #4b5563;
    --color-neutral-300: #6b7280;
    --color-neutral-400: #9ca3af;
    --color-neutral-500: #d1d5db;
    --color-neutral-600: #e5e7eb;
    --color-neutral-700: #f3f4f6;
    --color-neutral-800: #f9fafb;
    --color-neutral-900: #ffffff;
  }
}

/* Reduced Motion Support */
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}

/* High Contrast Support */
@media (prefers-contrast: high) {
  .advanced-search-interface {
    --color-primary: #0066cc;
    border: 2px solid;
  }
  
  .search-input-wrapper:focus-within {
    border-width: 3px;
  }
}
</style> 