import { ref, computed, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import searchService from '@/services/searchService'
import { useToast } from '@/composables/useToast'

// Types
export interface SearchFilter {
  timeRange: 'all' | 'today' | 'week' | 'month'
  messageType: 'all' | 'text' | 'files' | 'images'
  fromUser: string
  sortBy: 'newest' | 'oldest' | 'relevance'
}

export interface SearchResult {
  id: number
  content: string
  author?: {
    id: number
    fullname: string
    avatar?: string
  }
  created_at: string
  chat?: {
    id: number
    name: string
  }
  highlighted_content?: string
}

export interface SearchPagination {
  page: number
  limit: number
  total: number
  hasMore: boolean
}

export interface SearchState {
  query: string
  filters: SearchFilter
  results: SearchResult[]
  loading: boolean
  hasSearched: boolean
  pagination: SearchPagination
  error: string | null
}

// Rate limiting
const searchRequestCount = ref(0)
const resetRequestCount = () => {
  searchRequestCount.value = 0
}

// Reset rate limit every minute
setInterval(resetRequestCount, 60000)

export function useCompactSearch() {
  const router = useRouter()
  const route = useRoute()
  const { notifyError } = useToast()

  // Reactive state
  const state = ref<SearchState>({
    query: '',
    filters: {
      timeRange: 'all',
      messageType: 'all',
      fromUser: '',
      sortBy: 'newest'
    },
    results: [],
    loading: false,
    hasSearched: false,
    pagination: {
      page: 1,
      limit: 20,
      total: 0,
      hasMore: false
    },
    error: null
  })

  // Cache for search results
  const searchCache = new Map<string, { results: SearchResult[], timestamp: number }>()
  const CACHE_DURATION = 5 * 60 * 1000 // 5 minutes

  // Debounce timer
  let debounceTimer: NodeJS.Timeout | null = null

  // Computed properties
  const hasActiveFilters = computed(() => {
    const { filters } = state.value
    return filters.timeRange !== 'all' ||
      filters.messageType !== 'all' ||
      filters.fromUser !== '' ||
      filters.sortBy !== 'newest'
  })

  const canSearch = computed(() => {
    return state.value.query.length >= 2 || hasActiveFilters.value
  })

  const isRateLimited = computed(() => {
    return searchRequestCount.value >= 30
  })

  // Generate cache key
  const getCacheKey = () => {
    const { query, filters, pagination } = state.value
    return JSON.stringify({ query, filters, page: pagination.page })
  }

  // Check cache
  const getCachedResults = (key: string) => {
    const cached = searchCache.get(key)
    if (cached && (Date.now() - cached.timestamp) < CACHE_DURATION) {
      return cached.results
    }
    return null
  }

  // Set cache
  const setCachedResults = (key: string, results: SearchResult[]) => {
    searchCache.set(key, { results, timestamp: Date.now() })
  }

  // Initialize from URL query parameters
  const initializeFromRoute = () => {
    const query = route.query

    if (query.q) state.value.query = String(query.q)
    if (query.timeRange) state.value.filters.timeRange = String(query.timeRange) as SearchFilter['timeRange']
    if (query.messageType) state.value.filters.messageType = String(query.messageType) as SearchFilter['messageType']
    if (query.fromUser) state.value.filters.fromUser = String(query.fromUser)
    if (query.sortBy) state.value.filters.sortBy = String(query.sortBy) as SearchFilter['sortBy']
    if (query.page) state.value.pagination.page = Number(query.page) || 1
  }

  // Sync state to URL
  const syncToRoute = () => {
    const query: Record<string, string> = {}

    if (state.value.query) query.q = state.value.query
    if (state.value.filters.timeRange !== 'all') query.timeRange = state.value.filters.timeRange
    if (state.value.filters.messageType !== 'all') query.messageType = state.value.filters.messageType
    if (state.value.filters.fromUser) query.fromUser = state.value.filters.fromUser
    if (state.value.filters.sortBy !== 'newest') query.sortBy = state.value.filters.sortBy
    if (state.value.pagination.page > 1) query.page = String(state.value.pagination.page)

    router.replace({ query })
  }

  // Perform search
  const performSearch = async (resetPagination = true) => {
    // Validation
    if (!canSearch.value) {
      notifyError('Search query must be at least 2 characters long')
      return
    }

    if (isRateLimited.value) {
      notifyError('Too many search requests. Please wait a moment.')
      return
    }

    // Check cache first
    const cacheKey = getCacheKey()
    const cachedResults = getCachedResults(cacheKey)
    if (cachedResults && !resetPagination) {
      state.value.results = cachedResults
      return
    }

    state.value.loading = true
    state.value.error = null
    searchRequestCount.value++

    if (resetPagination) {
      state.value.pagination.page = 1
      state.value.results = []
    }

    try {
      const searchParams = {
        query: state.value.query,
        ...state.value.filters,
        page: state.value.pagination.page,
        limit: state.value.pagination.limit
      }

      const response = await searchService.intelligentSearch(searchParams)

      if (resetPagination) {
        state.value.results = response.results
      } else {
        state.value.results.push(...response.results)
      }

      state.value.pagination = {
        ...state.value.pagination,
        total: response.total,
        hasMore: response.hasMore
      }

      state.value.hasSearched = true

      // Cache results
      setCachedResults(cacheKey, state.value.results)

      // Sync to URL
      syncToRoute()

    } catch (error) {
      console.error('Search failed:', error)
      state.value.error = error instanceof Error ? error.message : 'Search failed'
      notifyError('Search failed. Please try again.')
      state.value.results = []
    } finally {
      state.value.loading = false
    }
  }

  // Debounced search
  const debouncedSearch = (resetPagination = true) => {
    if (debounceTimer) {
      clearTimeout(debounceTimer)
    }

    debounceTimer = setTimeout(() => {
      performSearch(resetPagination)
    }, 400)
  }

  // Search functions
  const search = () => performSearch(true)
  const loadMore = () => {
    if (state.value.pagination.hasMore && !state.value.loading) {
      state.value.pagination.page++
      performSearch(false)
    }
  }

  // Filter management
  const updateFilter = <K extends keyof SearchFilter>(key: K, value: SearchFilter[K]) => {
    state.value.filters[key] = value
    debouncedSearch()
  }

  const clearFilters = () => {
    state.value.filters = {
      timeRange: 'all',
      messageType: 'all',
      fromUser: '',
      sortBy: 'newest'
    }
    debouncedSearch()
  }

  const clearAll = () => {
    state.value.query = ''
    clearFilters()
    state.value.results = []
    state.value.hasSearched = false
    state.value.error = null
    syncToRoute()
  }

  // Text highlighting
  const highlightSearchTerms = (content: string): string => {
    if (!state.value.query) return content

    const regex = new RegExp(`(${state.value.query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi')
    return content.replace(regex, '<mark class="bg-yellow-200 dark:bg-yellow-800 px-1 rounded">$1</mark>')
  }

  // Utility functions
  const getInitials = (name: string): string => {
    return name.split(' ').map(n => n[0]).join('').toUpperCase().slice(0, 2)
  }

  const formatTime = (timestamp: string): string => {
    const date = new Date(timestamp)
    const now = new Date()
    const diffInHours = (now.getTime() - date.getTime()) / (1000 * 60 * 60)

    if (diffInHours < 1) {
      return 'Just now'
    } else if (diffInHours < 24) {
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    } else {
      return date.toLocaleDateString([], { month: 'short', day: 'numeric' })
    }
  }

  // Watch for query changes
  watch(() => state.value.query, (newQuery) => {
    if (newQuery.length >= 2) {
      debouncedSearch()
    } else if (newQuery.length === 0) {
      state.value.results = []
      state.value.hasSearched = false
    }
  })

  // Initialize from route on mount
  initializeFromRoute()

  return {
    // State
    state: computed(() => state.value),

    // Computed
    hasActiveFilters,
    canSearch,
    isRateLimited,

    // Actions
    search,
    debouncedSearch,
    loadMore,
    updateFilter,
    clearFilters,
    clearAll,

    // Utilities
    highlightSearchTerms,
    getInitials,
    formatTime,

    // Direct state access for v-model
    query: computed({
      get: () => state.value.query,
      set: (value: string) => { state.value.query = value }
    }),
    timeRange: computed({
      get: () => state.value.filters.timeRange,
      set: (value: SearchFilter['timeRange']) => updateFilter('timeRange', value)
    }),
    messageType: computed({
      get: () => state.value.filters.messageType,
      set: (value: SearchFilter['messageType']) => updateFilter('messageType', value)
    }),
    fromUser: computed({
      get: () => state.value.filters.fromUser,
      set: (value: string) => updateFilter('fromUser', value)
    }),
    sortBy: computed({
      get: () => state.value.filters.sortBy,
      set: (value: SearchFilter['sortBy']) => updateFilter('sortBy', value)
    })
  }
}