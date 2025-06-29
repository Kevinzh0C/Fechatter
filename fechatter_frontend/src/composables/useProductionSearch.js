/**
 * Production Search Composable
 * Comprehensive search state management with performance optimization
 */

import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import searchService from '@/services/searchService.js';
import { useNotifications } from '@/composables/useNotifications';
import { useChatStore } from '@/stores/chat';

export function useProductionSearch(options = {}) {
  const {
    chatId = null,
    chatName = '',
    autoSearch = true,
    debounceDelay = 300,
    minQueryLength = 2,
    maxResults = 50,
    enableHistory = true,
    enableSuggestions = true
  } = options;

  const router = useRouter();
  const { notifyError, notifySuccess } = useNotifications();
  const chatStore = useChatStore();

  // Search state
  const searchQuery = ref('');
  const searchResults = ref([]);
  const searchSuggestions = ref([]);
  const searchHistory = ref([]);
  const isSearching = ref(false);
  const hasSearched = ref(false);
  const searchError = ref(null);
  const searchMode = ref('chat'); // 'chat', 'global', 'advanced'

  // Pagination state
  const currentPage = ref(0);
  const hasMoreResults = ref(false);
  const totalResults = ref(0);
  const searchTookMs = ref(0);

  // Advanced search filters
  const filters = ref({
    senderId: '',
    timeRange: 'all', // 'all', 'today', 'week', 'month', 'year'
    messageType: 'all', // 'all', 'text', 'image', 'file', 'link'
    hasFiles: false,
    hasLinks: false,
    sort: 'relevance' // 'relevance', 'date_desc', 'date_asc'
  });

  // UI state
  const showAdvancedFilters = ref(false);
  const showSuggestions = ref(false);
  const selectedResultIndex = ref(-1);
  const searchInputFocused = ref(false);

  // Performance tracking
  const metrics = ref({
    searchCount: 0,
    avgResponseTime: 0,
    cacheHitRate: 0,
    errorRate: 0
  });

  // Debounce timer
  let debounceTimer = null;
  let searchAbortController = null;

  /**
   * Computed properties
   */
  const hasQuery = computed(() => searchQuery.value.trim().length >= minQueryLength);
  const hasResults = computed(() => searchResults.value.length > 0);
  const hasFilters = computed(() => {
    return filters.value.senderId ||
      filters.value.timeRange !== 'all' ||
      filters.value.messageType !== 'all' ||
      filters.value.hasFiles ||
      filters.value.hasLinks;
  });

  const searchPlaceholder = computed(() => {
    switch (searchMode.value) {
      case 'chat':
        return chatName ? `Search messages in #${chatName}...` : 'Search messages in this chat...';
      case 'global':
        return 'Search messages across all chats...';
      case 'advanced':
        return 'Advanced search with filters...';
      default:
        return 'Search messages...';
    }
  });

  const searchStats = computed(() => {
    if (!hasSearched.value) return '';

    const count = totalResults.value;
    const time = searchTookMs.value;

    if (count === 0) return 'No results found';
    if (count === 1) return `1 result in ${time}ms`;
    return `${count.toLocaleString()} results in ${time}ms`;
  });

  /**
   * Core search functionality with improved error handling
   */
  async function performSearch(query = searchQuery.value, resetResults = true) {
    const trimmedQuery = query?.trim();

    if (!trimmedQuery || trimmedQuery.length < minQueryLength) {
      clearResults();
      return;
    }

    // Prevent duplicate searches
    if (isSearching.value && trimmedQuery === searchQuery.value) {
      console.log('🔄 [useProductionSearch] Search already in progress, skipping duplicate');
      return;
    }

    // Cancel previous search
    if (searchAbortController) {
      searchAbortController.abort();
    }

    // Create new abort controller
    searchAbortController = new AbortController();

    console.log(`[useProductionSearch] Starting search for: "${trimmedQuery}" (mode: ${searchMode.value})`);

    isSearching.value = true;
    searchError.value = null;

    if (resetResults) {
      searchResults.value = [];
      currentPage.value = 0;
    }

    try {
      const searchParams = {
        query: trimmedQuery,
        chatId,
        limit: maxResults,
        offset: currentPage.value * maxResults,
        sort: filters.value.sort,
        ...buildSearchFilters()
      };

      let searchResult;

      switch (searchMode.value) {
        case 'chat':
          if (!chatId) {
            throw new Error('Chat ID is required for chat search');
          }
          searchResult = await searchService.searchInChat(searchParams);
          break;

        case 'global':
          searchResult = await searchService.searchGlobalMessages(searchParams);
          break;

        case 'advanced':
          searchResult = await searchService.advancedSearch(searchParams);
          break;

        default:
          throw new Error('Invalid search mode');
      }

      // Check if search was aborted
      if (searchAbortController?.signal.aborted) {
        console.log('🚫 [useProductionSearch] Search was aborted');
        return;
      }

      // Process results
      const hits = searchResult.hits || [];
      const total = searchResult.total || 0;
      const isFallback = searchResult.fallback || false;

      console.log(`[useProductionSearch] Search completed: ${hits.length} hits, ${total} total ${isFallback ? '(FALLBACK)' : ''}`);

      // Update results
      if (resetResults) {
        searchResults.value = hits;
      } else {
        searchResults.value.push(...hits);
      }

      totalResults.value = total;
      searchTookMs.value = searchResult.took_ms || 0;
      hasMoreResults.value = searchResult.page?.has_more || false;
      hasSearched.value = true;

      // Update search history
      if (enableHistory && resetResults) {
        addToHistory(trimmedQuery);
      }

      // Update metrics
      updateMetrics();

      // Show fallback notification if needed
      if (isFallback && hits.length > 0) {
        console.log('[useProductionSearch] Using local search results (backend unavailable)');
      }

    } catch (error) {
      if (error.name === 'AbortError' || searchAbortController?.signal.aborted) {
        console.log('🚫 [useProductionSearch] Search was cancelled');
        return;
      }

      const errorMessage = error.message || 'Search failed';
      searchError.value = errorMessage;
      console.error('ERROR: [useProductionSearch] Search failed:', error);

      // Don't show notification for network errors that have fallback
      if (!error.message?.includes('Search service is temporarily unavailable')) {
        notifyError(errorMessage);
      }
    } finally {
      isSearching.value = false;
      searchAbortController = null;
    }
  }

  /**
   * Debounced search with intelligent API call management
   */
  function handleSearchInput(query) {
    searchQuery.value = query;

    if (!autoSearch) return;

    clearTimeout(debounceTimer);

    if (!query || query.trim().length < minQueryLength) {
      clearResults();
      hideSuggestions();
      return;
    }

    // Only perform main search, suggestions are handled separately to avoid duplicate API calls
    debounceTimer = setTimeout(() => {
      console.log(`[useProductionSearch] Debounced search triggered for: "${query}"`);
      performSearch(query);
      // Note: loadSuggestions removed here to prevent duplicate API calls
      // Suggestions can be loaded on focus or manually if needed
    }, debounceDelay);
  }

  /**
   * Load more results (pagination)
   */
  async function loadMoreResults() {
    if (!hasMoreResults.value || isSearching.value) return;

    currentPage.value++;
    await performSearch(searchQuery.value, false);
  }

  /**
   * Load suggestions manually (separate from main search)
   */
  async function loadSuggestionsManually(query) {
    if (!enableSuggestions || !query || query.trim().length < minQueryLength) {
      searchSuggestions.value = [];
      return;
    }

    try {
      console.log(`[useProductionSearch] Loading suggestions for: "${query}"`);
      const suggestions = await searchService.getSearchSuggestions(query.trim(), 5);
      searchSuggestions.value = suggestions;
      showSuggestions.value = suggestions.length > 0;
    } catch (error) {
      console.warn('WARNING: [useProductionSearch] Load suggestions failed:', error);
      searchSuggestions.value = [];
    }
  }

  function selectSuggestion(suggestion) {
    searchQuery.value = suggestion;
    showSuggestions.value = false;
    performSearch(suggestion);
  }

  function hideSuggestions() {
    showSuggestions.value = false;
    searchSuggestions.value = [];
  }

  /**
   * Search history management
   */
  function addToHistory(query) {
    if (!enableHistory || !query) return;

    // Remove duplicates
    const filtered = searchHistory.value.filter(item => item.query !== query);

    // Add to beginning
    searchHistory.value = [
      { query, timestamp: Date.now(), mode: searchMode.value },
      ...filtered
    ].slice(0, 10); // Keep last 10 searches

    // Persist to localStorage
    localStorage.setItem('fechatter_search_history', JSON.stringify(searchHistory.value));
  }

  function loadSearchHistory() {
    if (!enableHistory) return;

    try {
      const saved = localStorage.getItem('fechatter_search_history');
      if (saved) {
        searchHistory.value = JSON.parse(saved);
      }
    } catch (error) {
      console.warn('WARNING: [useProductionSearch] Load search history failed:', error);
      searchHistory.value = [];
    }
  }

  function clearSearchHistory() {
    searchHistory.value = [];
    localStorage.removeItem('fechatter_search_history');
  }

  function searchFromHistory(historyItem) {
    searchQuery.value = historyItem.query;
    searchMode.value = historyItem.mode || 'chat';
    performSearch(historyItem.query);
  }

  /**
   * Advanced search filters
   */
  function buildSearchFilters() {
    const searchFilters = {};

    if (filters.value.senderId) {
      searchFilters.senderId = filters.value.senderId;
    }

    if (filters.value.timeRange !== 'all') {
      const timeFilter = getTimeRangeFilter(filters.value.timeRange);
      if (timeFilter) {
        searchFilters.startDate = timeFilter.startDate;
        searchFilters.endDate = timeFilter.endDate;
      }
    }

    if (filters.value.hasFiles) {
      searchFilters.hasFiles = true;
    }

    if (filters.value.hasLinks) {
      searchFilters.hasLinks = true;
    }

    return searchFilters;
  }

  function getTimeRangeFilter(timeRange) {
    const now = new Date();
    let startDate;

    switch (timeRange) {
      case 'today':
        startDate = new Date(now.getFullYear(), now.getMonth(), now.getDate());
        break;
      case 'week':
        startDate = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
        break;
      case 'month':
        startDate = new Date(now.getFullYear(), now.getMonth() - 1, now.getDate());
        break;
      case 'year':
        startDate = new Date(now.getFullYear() - 1, now.getMonth(), now.getDate());
        break;
      default:
        return null;
    }

    return {
      startDate: startDate.toISOString(),
      endDate: now.toISOString()
    };
  }

  function resetFilters() {
    filters.value = {
      senderId: '',
      timeRange: 'all',
      messageType: 'all',
      hasFiles: false,
      hasLinks: false,
      sort: 'relevance'
    };

    if (hasQuery.value) {
      performSearch();
    }
  }

  function toggleAdvancedFilters() {
    showAdvancedFilters.value = !showAdvancedFilters.value;
  }

  /**
   * Result navigation
   */
  function selectResult(result, index) {
    selectedResultIndex.value = index;

    // Navigate to message
    if (result.chat_id && result.id) {
      router.push({
        name: 'chat',
        params: { chatId: result.chat_id },
        query: { highlight: result.id }
      });
    }
  }

  function navigateResults(direction) {
    if (!hasResults.value) return;

    const maxIndex = searchResults.value.length - 1;

    if (direction === 'next') {
      selectedResultIndex.value = selectedResultIndex.value < maxIndex
        ? selectedResultIndex.value + 1
        : 0;
    } else if (direction === 'prev') {
      selectedResultIndex.value = selectedResultIndex.value > 0
        ? selectedResultIndex.value - 1
        : maxIndex;
    }

    // Auto-scroll to selected result
    nextTick(() => {
      const element = document.querySelector(`[data-result-index="${selectedResultIndex.value}"]`);
      if (element) {
        element.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
      }
    });
  }

  /**
   * Utility functions
   */
  function clearResults() {
    searchResults.value = [];
    totalResults.value = 0;
    searchTookMs.value = 0;
    hasMoreResults.value = false;
    hasSearched.value = false;
    selectedResultIndex.value = -1;
    searchError.value = null;
  }

  function clearSearch() {
    searchQuery.value = '';
    clearResults();
    hideSuggestions();
  }

  function updateMetrics() {
    metrics.value = searchService.getMetrics();
  }

  function switchSearchMode(mode) {
    searchMode.value = mode;
    if (hasQuery.value) {
      performSearch();
    }
  }

  /**
   * Keyboard shortcuts
   */
  function handleKeydown(event) {
    if (!searchInputFocused.value) return;

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        if (showSuggestions.value && searchSuggestions.value.length > 0) {
          // Handle suggestion navigation
        } else if (hasResults.value) {
          navigateResults('next');
        }
        break;

      case 'ArrowUp':
        event.preventDefault();
        if (hasResults.value) {
          navigateResults('prev');
        }
        break;

      case 'Enter':
        event.preventDefault();
        if (showSuggestions.value && searchSuggestions.value.length > 0) {
          selectSuggestion(searchSuggestions.value[0]);
        } else if (selectedResultIndex.value >= 0) {
          const result = searchResults.value[selectedResultIndex.value];
          selectResult(result, selectedResultIndex.value);
        } else if (hasQuery.value) {
          performSearch();
        }
        break;

      case 'Escape':
        event.preventDefault();
        if (showSuggestions.value) {
          hideSuggestions();
        } else {
          clearSearch();
        }
        break;
    }
  }

  /**
   * Lifecycle hooks
   */
  onMounted(() => {
    loadSearchHistory();
    document.addEventListener('keydown', handleKeydown);
  });

  onUnmounted(() => {
    clearTimeout(debounceTimer);
    if (searchAbortController) {
      searchAbortController.abort();
    }
    document.removeEventListener('keydown', handleKeydown);
  });

  /**
   * Watchers
   */
  watch(() => filters.value, () => {
    if (hasQuery.value) {
      performSearch();
    }
  }, { deep: true });

  // Expose reactive state and methods
  return {
    // State
    searchQuery,
    searchResults,
    searchSuggestions,
    searchHistory,
    isSearching,
    hasSearched,
    searchError,
    searchMode,
    currentPage,
    hasMoreResults,
    totalResults,
    searchTookMs,
    filters,
    showAdvancedFilters,
    showSuggestions,
    selectedResultIndex,
    searchInputFocused,
    metrics,

    // Computed
    hasQuery,
    hasResults,
    hasFilters,
    searchPlaceholder,
    searchStats,

    // Methods
    performSearch,
    handleSearchInput,
    loadMoreResults,
    loadSuggestionsManually,
    selectSuggestion,
    hideSuggestions,
    addToHistory,
    loadSearchHistory,
    clearSearchHistory,
    searchFromHistory,
    resetFilters,
    toggleAdvancedFilters,
    selectResult,
    navigateResults,
    clearResults,
    clearSearch,
    updateMetrics,
    switchSearchMode,
    handleKeydown
  };
} 