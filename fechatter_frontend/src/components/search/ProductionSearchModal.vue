<template>
  <!-- 🔍 Production Search Modal - Human-Centered Design -->
  <teleport to="body">
    <transition name="modal-backdrop">
      <div 
        v-if="isModalOpen"
        class="search-modal-backdrop"
        @click="handleBackdropClick"
        @keydown.esc="handleClose"
        role="dialog"
        aria-modal="true"
        aria-label="Search Messages"
      >
        <transition name="modal-content">
          <div 
            class="search-modal-container"
            @click.stop
            ref="modalContainer"
          >
            <!-- Modal Header - Golden Ratio Optimized -->
            <header class="search-modal-header">
              <div class="header-content">
                <h2 class="modal-title">
                  <svg class="title-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                  </svg>
                  Search Messages
                </h2>
                
                <button 
                  @click="handleClose"
                  class="close-button"
                  type="button"
                  aria-label="Close"
                >
                  <svg class="close-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
            </header>

            <!-- Search Input Section - Ergonomic Design -->
            <section class="search-input-section">
              <div class="search-container">
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
                    @keydown="handleKeydown"
                    autocomplete="off"
                    spellcheck="false"
                  />
                  
                  <!-- Strategy Indicator -->
                  <div class="strategy-indicator" v-if="currentStrategy">
                    <span class="strategy-badge">{{ strategyLabels[currentStrategy] }}</span>
                  </div>
                  
                  <!-- Clear Button -->
                  <button
                    v-if="searchQuery"
                    @click="clearSearch"
                    class="clear-button"
                    type="button"
                    aria-label="Clear search"
                  >
                    <svg class="clear-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                  
                  <!-- 🔧 NEW: Search Action Button on the right -->
                  <button
                    v-if="searchQuery.trim() && searchQuery.length >= 2"
                    @click="handleSearch"
                    class="search-action-button"
                    :disabled="isSearching"
                    type="button"
                    aria-label="Search"
                  >
                    <svg v-if="!isSearching" class="search-button-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                    </svg>
                    <svg v-else class="search-button-icon animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" fill="none" stroke="currentColor" stroke-width="4" r="10"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12A8 8 0 018 4V2.5"></path>
                    </svg>
                  </button>
                </div>
                
                <!-- Quick Filters - Cognitive Load Optimized -->
                <div class="quick-filters" v-if="showQuickFilters">
                  <button 
                    v-for="filter in quickFilters"
                    :key="filter.value"
                    @click="applyQuickFilter(filter)"
                    class="quick-filter-button"
                    :class="{ 'active': activeFilter === filter.value }"
                    type="button"
                  >
                    {{ filter.label }}
                  </button>
                </div>
              </div>
            </section>

            <!-- Search Results - F-Pattern Layout -->
            <section class="search-results-section" v-if="isSearchActive">
              <!-- Results Header -->
              <header class="results-header">
                <div class="results-meta">
                  <span class="results-count">{{ formatResultsCount(searchResults.total) }}</span>
                  <span class="search-time" v-if="searchResults.took_ms">{{ searchResults.took_ms }}ms</span>
                </div>
                
                <div class="results-actions">
                  <button 
                    @click="toggleAdvancedFilters"
                    class="filters-toggle"
                    :class="{ 'active': showAdvancedFilters }"
                    type="button"
                  >
                    <svg class="filter-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
                    </svg>
                    Filters
                  </button>
                </div>
              </header>

              <!-- Advanced Filters Panel -->
              <transition name="filters-slide">
                <div v-if="showAdvancedFilters" class="advanced-filters-panel">
                  <div class="filters-grid">
                    <div class="filter-group">
                      <label class="filter-label">Time Range</label>
                      <select v-model="filters.timeRange" class="filter-select">
                        <option value="all">All Time</option>
                        <option value="today">Today</option>
                        <option value="week">This Week</option>
                        <option value="month">This Month</option>
                      </select>
                    </div>
                    
                    <div class="filter-group">
                      <label class="filter-label">Sender</label>
                      <input
                        v-model="filters.sender"
                        type="text"
                        class="filter-input"
                        placeholder="Filter by sender..."
                      />
                    </div>
                  </div>
                </div>
              </transition>

              <!-- Results Container -->
              <div class="results-container" ref="resultsContainer">
                <div class="results-list">
                  <SearchResultItem
                    v-for="result in searchResults.hits"
                    :key="result.id"
                    :result="result"
                    :query="searchQuery"
                    @navigate="handleResultNavigation"
                    @preview="handleResultPreview"
                  />
                </div>
                
                <!-- Load More -->
                <div v-if="hasMoreResults" class="load-more-section">
                  <button
                    @click="loadMoreResults"
                    class="load-more-button"
                    :disabled="isLoadingMore"
                    type="button"
                  >
                    <svg v-if="isLoadingMore" class="loading-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                    </svg>
                    <span>{{ isLoadingMore ? 'Loading...' : 'Load More' }}</span>
                  </button>
                </div>
                
                <!-- Empty State -->
                <div v-if="showEmptyState" class="empty-state">
                  <div class="empty-icon">
                    <svg class="empty-search-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                    </svg>
                  </div>
                  <h3 class="empty-title">No results found</h3>
                  <p class="empty-description">Try different keywords or adjust your filters</p>
                </div>
              </div>
            </section>

            <!-- Initial State - Cognitive Guidance -->
            <section v-else class="search-welcome-section">
              <div class="welcome-content">
                <div class="welcome-icon">
                  <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                  </svg>
                </div>
                <h3 class="welcome-title">Search Messages</h3>
                <p class="welcome-description">Start typing to search through messages in this chat</p>
                
                <!-- Quick Search Suggestions -->
                <div class="search-suggestions">
                  <span class="suggestions-label">Try searching for:</span>
                  <div class="suggestion-chips">
                    <button
                      v-for="suggestion in quickSuggestions"
                      :key="suggestion"
                      @click="applySuggestion(suggestion)"
                      class="suggestion-chip"
                      type="button"
                    >
                      {{ suggestion }}
                    </button>
                  </div>
                </div>
              </div>
            </section>

            <!-- Modal Footer - Keyboard Shortcuts -->
            <footer class="search-modal-footer">
              <div class="keyboard-shortcuts">
                <span class="shortcut-hint">
                  <kbd>Esc</kbd> to close • <kbd>↑↓</kbd> to navigate • <kbd>Enter</kbd> to select
                </span>
              </div>
            </footer>
          </div>
        </transition>
      </div>
    </transition>
  </teleport>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue';
import { useDebounceFn } from '@vueuse/core';
import SearchResultItem from './SearchResultItem.vue';
import searchService from '@/services/searchService';

// Props
const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  },
  isOpen: {
    type: Boolean,
    default: false
  },
  chatId: {
    type: [String, Number],
    default: null
  },
  initialQuery: {
    type: String,
    default: ''
  }
});

// Emits
const emit = defineEmits(['close', 'navigate-to-message', 'search-complete', 'update:modelValue']);

// Refs
const searchInput = ref(null);
const modalContainer = ref(null);
const resultsContainer = ref(null);

// Reactive State
const searchQuery = ref(props.initialQuery);
const isSearchActive = ref(false);
const isSearching = ref(false); // 🔧 NEW: Track search loading state
const showAdvancedFilters = ref(false);
const showQuickFilters = ref(true);
const currentStrategy = ref(null);
const activeFilter = ref('all');
const isLoadingMore = ref(false);

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
  sender: ''
});

// Quick Filters - Based on actual backend API capabilities
const quickFilters = ref([
  { label: 'All Messages', value: 'all', strategy: 'full_text' },
  { label: 'Semantic', value: 'semantic', strategy: 'semantic' },
  { label: 'Exact Match', value: 'exact', strategy: 'exact' },
  { label: 'Recent', value: 'recent', strategy: 'temporal' }
]);

// Quick Suggestions
const quickSuggestions = ref(['project', 'meeting', 'task', 'deadline']);

// Strategy Labels
const strategyLabels = {
  full_text: 'Full Text',
  semantic: 'Semantic',
  fuzzy: 'Fuzzy',
  exact: 'Exact',
  boolean: 'Boolean',
  temporal: 'Temporal',
  user_scoped: 'User Scoped',
  file_content: 'File Content',
  fallback_local: 'Local Search'
};

// Computed Properties
const intelligentPlaceholder = computed(() => {
  const hour = new Date().getHours();
  if (hour < 12) return 'Search messages... Good morning!';
  if (hour < 18) return 'Search messages... Good afternoon!';
  return 'Search messages... Good evening!';
});

const hasMoreResults = computed(() => {
  return searchResults.value.total > searchResults.value.hits.length;
});

const showEmptyState = computed(() => {
  return isSearchActive.value && 
         searchQuery.value.length > 0 && 
         searchResults.value.hits.length === 0;
});

// Main Search Function
const performSearch = async (forceStrategy = null) => {
  if (!searchQuery.value.trim()) {
    resetSearch();
    return;
  }

  isSearchActive.value = true;
  isSearching.value = true;
  
  try {
    const searchParams = {
      query: searchQuery.value,
      chatId: props.chatId,
      filters: filters.value,
      limit: 20,
      offset: 0,
      strategies: forceStrategy ? [forceStrategy] : null
    };

    const results = await searchService.intelligentSearch(searchParams);
    
    searchResults.value = results;
    currentStrategy.value = results.strategies_used?.[0] || null;
    
    emit('search-complete', results);
    
  } catch (error) {
    console.error('Search failed:', error);
    searchResults.value = { hits: [], total: 0, took_ms: 0, strategies_used: [] };
  } finally {
    isSearching.value = false;
  }
};

// Debounced Search Function
const debouncedSearch = useDebounceFn(() => performSearch(), 300);

// Event Handlers
const handleSearchInput = () => {
  if (searchQuery.value.length >= 2) {
    debouncedSearch();
  } else {
    resetSearch();
  }
};

const handleSearch = () => {
  // Direct search trigger from button click
  const activeFilterObj = quickFilters.value.find(f => f.value === activeFilter.value);
  const strategy = activeFilterObj?.strategy || null;
  performSearch(strategy);
};

const handleKeydown = (event) => {
  if (event.key === 'Escape') {
    handleClose();
  } else if (event.key === 'Enter' && searchQuery.value.trim()) {
    handleSearch();
  }
  // Add more keyboard navigation here
};

const handleClose = () => {
  emit('close');
  emit('update:modelValue', false);
};

const handleBackdropClick = () => {
  handleClose();
};

const clearSearch = () => {
  searchQuery.value = '';
  resetSearch();
  searchInput.value?.focus();
};

const resetSearch = () => {
  isSearchActive.value = false;
  searchResults.value = { hits: [], total: 0, took_ms: 0, strategies_used: [] };
  currentStrategy.value = null;
};

const toggleAdvancedFilters = () => {
  showAdvancedFilters.value = !showAdvancedFilters.value;
};

const applyQuickFilter = (filter) => {
  activeFilter.value = filter.value;
  currentStrategy.value = filter.strategy;
  
  // Apply filter with specific strategy
  if (searchQuery.value.trim()) {
    performSearch(filter.strategy);
  }
};

const applySuggestion = (suggestion) => {
  searchQuery.value = suggestion;
  debouncedSearch();
};

const loadMoreResults = async () => {
  if (isLoadingMore.value) return;
  
  isLoadingMore.value = true;
  
  try {
    const searchParams = {
      query: searchQuery.value,
      chatId: props.chatId,
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

const handleResultNavigation = (result) => {
  emit('navigate-to-message', result);
  handleClose();
};

const handleResultPreview = (result) => {
  console.log('Preview result:', result);
};

// Utility Functions
const formatResultsCount = (count) => {
  if (count === 0) return 'No results';
  if (count === 1) return '1 result';
  return `${count} results`;
};

// Lifecycle
onMounted(() => {
  // Focus search input when modal opens
  if (props.isOpen) {
    nextTick(() => {
      searchInput.value?.focus();
    });
  }
});

// Handle focus when modal opens
watch(() => isModalOpen.value, (isOpen) => {
  if (isOpen) {
    nextTick(() => {
      searchInput.value?.focus();
    });
    
    // Reset state when opening
    if (props.initialQuery !== searchQuery.value) {
      searchQuery.value = props.initialQuery;
      if (props.initialQuery) {
        debouncedSearch();
      }
    }
  } else {
    // Clear search when closing
    resetSearch();
  }
});

// Keyboard event handling
onMounted(() => {
  document.addEventListener('keydown', (e) => {
    if (isModalOpen.value && e.key === 'Escape') {
      handleClose();
    }
  });
});

// 计算实际的打开状态 - 修复逻辑优先级
const isModalOpen = computed(() => {
  // 🔧 FIX: 优先使用 isOpen (父组件传递的值)，modelValue 为空时才使用默认值
  if (props.isOpen !== undefined && props.isOpen !== false) {
    return props.isOpen;
  }
  // 兜底使用 modelValue (v-model 支持)
  return props.modelValue || false;
});
</script>

<style scoped>
/* 🎨 Modern Search Modal - Premium Design System inspired by Notion, Linear & Slack */

/* Modern Design Variables - Human-Centered & Accessible */
.search-modal-backdrop {
  /* Spacing System - 8pt grid with optical adjustments */
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --space-8: 32px;
  --space-10: 40px;
  --space-12: 48px;
  --space-16: 64px;
  
  /* Touch Targets - Apple HIG & Google Material compliant */
  --touch-min: 44px;
  --touch-comfortable: 48px;
  --touch-spacious: 56px;
  
  /* Modern Color System - Designed for accessibility */
  --color-primary: #6366f1; /* Indigo 500 - modern primary */
  --color-primary-hover: #4f46e5; /* Indigo 600 */
  --color-primary-light: #e0e7ff; /* Indigo 100 */
  
  --color-success: #059669; /* Emerald 600 */
  --color-warning: #d97706; /* Amber 600 */
  --color-danger: #dc2626; /* Red 600 */
  
  /* Semantic Neutrals */
  --color-background: #ffffff;
  --color-surface: #f8fafc; /* Slate 50 */
  --color-surface-hover: #f1f5f9; /* Slate 100 */
  --color-border: #e2e8f0; /* Slate 200 */
  --color-border-strong: #cbd5e1; /* Slate 300 */
  
  --color-text-primary: #0f172a; /* Slate 900 */
  --color-text-secondary: #475569; /* Slate 600 */
  --color-text-tertiary: #94a3b8; /* Slate 400 */
  --color-text-inverse: #ffffff;
  
  /* Shadow System - Elevation-based */
  --shadow-xs: 0 1px 2px 0 rgb(0 0 0 / 0.05);
  --shadow-sm: 0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1);
  --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
  --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1);
  --shadow-xl: 0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1);
  --shadow-2xl: 0 25px 50px -12px rgb(0 0 0 / 0.25);
  
  /* Border Radius System */
  --radius-sm: 6px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-xl: 16px;
  --radius-2xl: 20px;
  
  /* Legacy spacing compatibility */
  --spacing-xs: var(--space-1);
  --spacing-sm: var(--space-2);
  --spacing-md: var(--space-4);
  --spacing-lg: var(--space-6);
  
  /* Legacy border compatibility */
  --color-border-light: var(--color-border);
  --color-border-medium: var(--color-border-strong);
  --color-bg-secondary: var(--color-surface);
  --color-bg-tertiary: var(--color-surface-hover);
  
  /* Animation System */
  --duration-fast: 150ms;
  --duration-normal: 200ms;
  --duration-slow: 300ms;
  --easing-standard: cubic-bezier(0.4, 0.0, 0.2, 1);
  --easing-enter: cubic-bezier(0.0, 0.0, 0.2, 1);
  --easing-exit: cubic-bezier(0.4, 0.0, 1, 1);
  
  /* Modal Container */
  position: fixed;
  inset: 0;
  background: linear-gradient(to bottom right, 
    rgba(0, 0, 0, 0.4), 
    rgba(0, 0, 0, 0.6)
  );
  backdrop-filter: blur(8px) saturate(150%);
  -webkit-backdrop-filter: blur(8px) saturate(150%);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: var(--space-6) var(--space-4);
  z-index: 50; /* Higher z-index for modern stacking */
  overflow-y: auto;
  
  /* Smooth scroll behavior */
  scroll-behavior: smooth;
}

.search-modal-container {
  /* Modern Container Design */
  background: var(--color-background);
  border-radius: var(--radius-2xl);
  box-shadow: 
    var(--shadow-2xl),
    0 0 0 1px rgba(255, 255, 255, 0.05); /* Subtle border glow */
  
  /* Responsive Sizing */
  width: 100%;
  min-width: 320px; /* Mobile minimum */
  max-width: min(680px, calc(100vw - var(--space-8))); /* Adaptive width */
  max-height: min(720px, calc(100vh - var(--space-12))); /* Adaptive height */
  
  /* Positioning */
  margin-top: var(--space-16);
  
  /* Layout */
  display: flex;
  flex-direction: column;
  overflow: hidden;
  
  /* Enhanced visuals */
  border: 1px solid var(--color-border);
  
  /* Smooth transitions */
  transition: all var(--duration-normal) var(--easing-standard);
  
  /* Glass morphism effect */
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
}

/* Modal Header - Modern & Elegant */
.search-modal-header {
  padding: var(--space-6) var(--space-6) var(--space-4);
  border-bottom: 1px solid var(--color-border);
  background: linear-gradient(to bottom, 
    var(--color-background), 
    var(--color-surface)
  );
  /* Subtle top border radius */
  border-radius: var(--radius-2xl) var(--radius-2xl) 0 0;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.modal-title {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin: 0;
  font-size: 20px; /* Optimal reading size */
  font-weight: 600; /* Modern weight - not too bold */
  line-height: 1.2;
  color: var(--color-text-primary);
  letter-spacing: -0.01em; /* Subtle optical spacing */
}

.title-icon {
  width: 20px; /* Balanced with text */
  height: 20px;
  color: var(--color-primary);
  flex-shrink: 0;
}

.close-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--touch-comfortable);
  height: var(--touch-comfortable);
  border: none;
  background: transparent;
  border-radius: var(--radius-lg);
  cursor: pointer;
  color: var(--color-text-tertiary);
  
  /* Modern transition */
  transition: all var(--duration-fast) var(--easing-standard);
  
  /* Focus styles */
  &:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }
}

.close-button:hover {
  background: var(--color-surface-hover);
  color: var(--color-text-secondary);
  transform: scale(1.05); /* Subtle hover feedback */
}

.close-button:active {
  transform: scale(0.95); /* Satisfying click feedback */
}

.close-icon {
  width: 18px; /* Slightly smaller for elegance */
  height: 18px;
}

/* Search Input Section - Modern & Accessible */
.search-input-section {
  padding: var(--space-4) var(--space-6) var(--space-6);
  border-bottom: 1px solid var(--color-border);
  background: var(--color-background);
}

.search-container {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  background: var(--color-surface);
  border: 1.5px solid var(--color-border);
  border-radius: var(--radius-xl);
  padding: 0 var(--space-4);
  min-height: var(--touch-spacious); /* Spacious for comfortable typing */
  
  /* Modern focus states */
  transition: all var(--duration-normal) var(--easing-standard);
  
  /* Subtle inner shadow for depth */
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.04);
}

.search-input-wrapper:focus-within {
  border-color: var(--color-primary);
  background: var(--color-background);
  box-shadow: 
    inset 0 1px 2px rgba(0, 0, 0, 0.04),
    0 0 0 3px var(--color-primary-light),
    var(--shadow-sm);
  transform: translateY(-1px); /* Subtle lift effect */
}

.search-icon-wrapper {
  display: flex;
  align-items: center;
  margin-right: var(--space-3);
  flex-shrink: 0;
}

.search-icon {
  width: 18px; /* Balanced size */
  height: 18px;
  color: var(--color-text-tertiary);
  transition: color var(--duration-fast) var(--easing-standard);
}

.search-input-wrapper:focus-within .search-icon {
  color: var(--color-primary);
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  
  /* Typography */
  font-size: 16px; /* Prevents zoom on iOS */
  font-weight: 400;
  line-height: 1.5;
  color: var(--color-text-primary);
  
  /* Spacing */
  padding: var(--space-4) 0;
  
  /* Remove autofill styles */
  &:-webkit-autofill {
    -webkit-box-shadow: 0 0 0 1000px var(--color-background) inset;
    -webkit-text-fill-color: var(--color-text-primary);
  }
}

.search-input::placeholder {
  color: var(--color-text-tertiary);
  font-weight: 400;
}

/* 🔧 NEW: Search Action Button - Modern & Prominent */
.search-action-button {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  margin-left: var(--space-2);
  
  /* Modern button design */
  background: linear-gradient(135deg, var(--color-primary), var(--color-primary-hover));
  border: none;
  border-radius: var(--radius-lg);
  color: var(--color-text-inverse);
  
  /* Typography */
  font-size: 14px;
  font-weight: 600;
  white-space: nowrap;
  
  /* Interaction */
  cursor: pointer;
  user-select: none;
  
  /* Smooth transitions */
  transition: all var(--duration-fast) var(--easing-standard);
  
  /* Hover effects */
  &:hover:not(:disabled) {
    background: linear-gradient(135deg, var(--color-primary-hover), var(--color-primary));
    transform: translateY(-1px);
    box-shadow: var(--shadow-md);
  }
  
  /* Active effect */
  &:active:not(:disabled) {
    transform: translateY(0);
    box-shadow: var(--shadow-sm);
  }
  
  /* Disabled state */
  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    transform: none;
  }
  
  /* Focus styles */
  &:focus-visible {
    outline: 2px solid var(--color-primary-light);
    outline-offset: 2px;
  }
}

.search-button-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

/* Spin animation for loading state */
.animate-spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.strategy-indicator {
  margin: 0 var(--space-sm);
}

.strategy-badge {
  display: inline-flex;
  align-items: center;
  padding: calc(var(--space-xs)) var(--space-sm);
  background: var(--color-primary);
  color: white;
  border-radius: var(--space-xs);
  font-size: 12px;
  font-weight: 600;
}

.clear-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--touch-min);
  height: var(--touch-min);
  border: none;
  background: transparent;
  border-radius: 50%;
  cursor: pointer;
  transition: background-color 0.2s ease;
  color: var(--color-neutral-400);
}

.clear-button:hover {
  background: var(--color-neutral-100);
  color: var(--color-neutral-600);
}

.clear-icon {
  width: 16px;
  height: 16px;
}

/* Quick Filters - Modern Pills Design */
.quick-filters {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
  align-items: center;
}

.quick-filter-button {
  /* Modern pill design */
  padding: var(--space-2) var(--space-4);
  border: 1px solid var(--color-border);
  background: var(--color-background);
  border-radius: var(--radius-2xl); /* Full pill shape */
  
  /* Typography */
  font-size: 14px;
  font-weight: 500;
  line-height: 1.4;
  color: var(--color-text-secondary);
  
  /* Interaction */
  cursor: pointer;
  user-select: none;
  min-height: var(--touch-min); /* Minimum touch target */
  
  /* Smooth transitions */
  transition: all var(--duration-fast) var(--easing-standard);
  
  /* Focus styles */
  &:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }
}

.quick-filter-button:hover {
  border-color: var(--color-primary);
  background: var(--color-primary-light);
  color: var(--color-primary);
  transform: translateY(-1px); /* Subtle lift */
  box-shadow: var(--shadow-sm);
}

.quick-filter-button.active {
  border-color: var(--color-primary);
  background: var(--color-primary);
  color: var(--color-text-inverse);
  box-shadow: var(--shadow-md);
  
  /* Active state should not lift on hover */
  &:hover {
    transform: none;
  }
}

/* Search Results Section */
.search-results-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--color-neutral-200);
  background: var(--color-neutral-50);
}

.results-meta {
  display: flex;
  align-items: center;
  gap: var(--space-md);
}

.results-count {
  font-weight: 600;
  color: var(--color-neutral-900);
}

.search-time {
  font-size: 14px;
  color: var(--color-neutral-500);
}

.filters-toggle {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-sm) var(--space-md);
  border: 1px solid var(--color-neutral-300);
  background: white;
  border-radius: var(--space-sm);
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  min-height: 36px;
}

.filters-toggle:hover {
  border-color: var(--color-primary);
  background: rgba(59, 130, 246, 0.05);
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

/* Advanced Filters Panel */
.advanced-filters-panel {
  padding: var(--space-lg);
  background: var(--color-neutral-50);
  border-bottom: 1px solid var(--color-neutral-200);
}

.filters-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: var(--space-lg);
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.filter-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-neutral-700);
}

.filter-select,
.filter-input {
  padding: var(--space-sm);
  border: 1px solid var(--color-neutral-300);
  border-radius: var(--space-sm);
  font-size: 14px;
  background: white;
  min-height: var(--touch-min);
}

.filter-select:focus,
.filter-input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

/* Results Container */
.results-container {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-lg);
}

.results-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

/* Load More Section */
.load-more-section {
  display: flex;
  justify-content: center;
  padding: var(--space-lg) 0;
}

.load-more-button {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-sm) var(--space-lg);
  border: 2px solid var(--color-primary);
  background: white;
  color: var(--color-primary);
  border-radius: calc(var(--space-md) * var(--golden-ratio));
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  min-height: var(--touch-optimal);
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

/* Empty State */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--space-xl);
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
  margin-bottom: var(--space-lg);
}

.empty-search-icon {
  width: 32px;
  height: 32px;
  color: var(--color-neutral-400);
}

.empty-title {
  margin: 0 0 var(--space-sm) 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-neutral-900);
}

.empty-description {
  margin: 0;
  color: var(--color-neutral-600);
  line-height: 1.5;
}

/* Welcome Section */
.search-welcome-section {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-xl);
}

.welcome-content {
  text-align: center;
  max-width: 400px;
}

.welcome-icon {
  width: 80px;
  height: 80px;
  background: var(--color-neutral-100);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto var(--space-lg);
}

.welcome-icon svg {
  width: 40px;
  height: 40px;
  color: var(--color-neutral-400);
}

.welcome-title {
  margin: 0 0 var(--space-sm) 0;
  font-size: 24px;
  font-weight: 700;
  color: var(--color-neutral-900);
}

.welcome-description {
  margin: 0 0 var(--space-lg) 0;
  color: var(--color-neutral-600);
  line-height: 1.5;
}

.search-suggestions {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-sm);
}

.suggestions-label {
  font-size: 14px;
  color: var(--color-neutral-600);
}

.suggestion-chips {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-xs);
  justify-content: center;
}

.suggestion-chip {
  padding: var(--space-xs) var(--space-sm);
  border: 1px solid var(--color-neutral-300);
  background: white;
  border-radius: calc(var(--space-sm) / 2);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.suggestion-chip:hover {
  border-color: var(--color-primary);
  background: var(--color-primary);
  color: white;
}

/* Modal Footer */
.search-modal-footer {
  padding: var(--space-md) var(--space-lg);
  border-top: 1px solid var(--color-neutral-200);
  background: var(--color-neutral-50);
}

.keyboard-shortcuts {
  display: flex;
  justify-content: center;
}

.shortcut-hint {
  font-size: 12px;
  color: var(--color-neutral-500);
}

kbd {
  display: inline-block;
  padding: 2px 6px;
  background: var(--color-neutral-200);
  border-radius: 3px;
  font-size: 11px;
  font-family: monospace;
  color: var(--color-neutral-700);
}

/* Transitions */
.modal-backdrop-enter-active,
.modal-backdrop-leave-active {
  transition: all 0.3s ease;
}

.modal-backdrop-enter-from,
.modal-backdrop-leave-to {
  opacity: 0;
}

.modal-content-enter-active,
.modal-content-leave-active {
  transition: all 0.3s ease;
}

.modal-content-enter-from,
.modal-content-leave-to {
  opacity: 0;
  transform: translateY(-20px) scale(0.95);
}

.filters-slide-enter-active,
.filters-slide-leave-active {
  transition: all 0.3s ease;
}

.filters-slide-enter-from,
.filters-slide-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

/* Modern Responsive Design - Mobile First Approach */
@media (max-width: 640px) {
  .search-modal-backdrop {
    padding: var(--space-4);
    align-items: flex-start; /* Top align on mobile */
  }
  
  .search-modal-container {
    margin-top: var(--space-8);
    max-height: calc(100vh - var(--space-8));
    max-width: 100%;
    border-radius: var(--radius-xl); /* Slightly smaller radius on mobile */
  }
  
  .search-modal-header {
    padding: var(--space-4) var(--space-5);
    border-radius: var(--radius-xl) var(--radius-xl) 0 0;
  }
  
  .modal-title {
    font-size: 18px; /* Slightly smaller on mobile */
  }
  
  .search-input-section {
    padding: var(--space-3) var(--space-5) var(--space-5);
  }
  
  .search-input-wrapper {
    min-height: var(--touch-spacious); /* Ensure touch-friendly */
  }
  
  .quick-filters {
    gap: var(--space-2);
  }
  
  .quick-filter-button {
    padding: var(--space-2) var(--space-3);
    font-size: 13px; /* Slightly smaller text */
  }
  
  .results-container {
    padding: var(--space-4) var(--space-5);
  }
  
  .filters-grid {
    grid-template-columns: 1fr;
    gap: var(--space-4);
  }
  
  .results-header {
    flex-direction: column;
    gap: var(--space-3);
    align-items: stretch;
    padding: var(--space-3) var(--space-5);
  }
}

/* Tablet adjustments */
@media (min-width: 641px) and (max-width: 1024px) {
  .search-modal-container {
    max-width: min(600px, calc(100vw - var(--space-6)));
  }
}

/* Modern Transition System */
.modal-backdrop-enter-active {
  transition: all var(--duration-slow) var(--easing-enter);
}

.modal-backdrop-leave-active {
  transition: all var(--duration-normal) var(--easing-exit);
}

.modal-backdrop-enter-from,
.modal-backdrop-leave-to {
  opacity: 0;
  backdrop-filter: blur(0px);
}

.modal-content-enter-active {
  transition: all var(--duration-slow) var(--easing-enter);
  transition-delay: 50ms; /* Slight stagger for polish */
}

.modal-content-leave-active {
  transition: all var(--duration-normal) var(--easing-exit);
}

.modal-content-enter-from,
.modal-content-leave-to {
  opacity: 0;
  transform: translateY(-20px) scale(0.96);
}

/* Smooth filter animations */
.filters-slide-enter-active,
.filters-slide-leave-active {
  transition: all var(--duration-normal) var(--easing-standard);
}

.filters-slide-enter-from,
.filters-slide-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

/* Loading animations for search results */
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.loading-state {
  animation: pulse 1.5s ease-in-out infinite;
}

/* Smooth scroll for results */
.results-container {
  scroll-behavior: smooth;
}

/* Enhanced focus outlines for accessibility */
.search-modal-container:focus-within {
  box-shadow: 
    var(--shadow-2xl),
    0 0 0 1px rgba(255, 255, 255, 0.05),
    0 0 0 4px var(--color-primary-light);
}
</style> 