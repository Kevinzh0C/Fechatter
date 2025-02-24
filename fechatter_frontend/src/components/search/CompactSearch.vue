<template>
  <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
    <!-- Search Input -->
    <div class="p-4 border-b border-gray-200 dark:border-gray-700">
      <div class="relative">
        <svg 
          class="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" 
          fill="none" 
          stroke="currentColor" 
          viewBox="0 0 24 24"
          aria-hidden="true"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
        </svg>
        <input 
          v-model="query" 
          @keyup.enter="search"
          type="text" 
          :placeholder="'Search messages...'"
          :aria-label="'Search messages'"
          data-testid="search-input"
          class="w-full pl-10 pr-12 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-colors duration-200"
        >
        <button 
          @click="toggleFilters" 
          :aria-label="'Toggle filters'"
          :aria-expanded="showFilters"
          data-testid="filter-toggle"
          class="absolute right-2 top-1/2 transform -translate-y-1/2 p-1 text-gray-400 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors duration-200"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4"></path>
          </svg>
        </button>
      </div>
    </div>

    <!-- Compact Filters -->
    <div v-if="showFilters" class="p-3 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 text-sm">
        <!-- Time Range -->
        <select 
          v-model="timeRange" 
          :aria-label="'Time range filter'"
          class="px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
        >
          <option value="all">All time</option>
          <option value="today">Today</option>
          <option value="week">This week</option>
          <option value="month">This month</option>
        </select>

        <!-- Message Type -->
        <select 
          v-model="messageType"
          :aria-label="'Message type filter'"
          class="px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
        >
          <option value="all">All types</option>
          <option value="text">Text only</option>
          <option value="files">With files</option>
          <option value="images">With images</option>
        </select>

        <!-- From User -->
        <select 
          v-model="fromUser"
          :aria-label="'From user filter'"
          class="px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
        >
          <option value="">Anyone</option>
          <option v-for="user in availableUsers" :key="user.id" :value="user.id">
            {{ user.fullname }}
          </option>
        </select>

        <!-- Sort -->
        <select 
          v-model="sortBy"
          :aria-label="'Sort by'"
          class="px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
        >
          <option value="newest">Newest first</option>
          <option value="oldest">Oldest first</option>
          <option value="relevance">Most relevant</option>
        </select>
      </div>
      
      <div class="flex justify-between items-center mt-3">
        <button 
          @click="clearFilters" 
          class="text-xs text-gray-500 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors duration-200"
        >
          Clear filters
        </button>
        <button 
          @click="search" 
          data-testid="search-btn"
          :disabled="!canSearch"
          class="px-3 py-1 bg-indigo-600 text-white text-xs rounded hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200"
        >
          Search
        </button>
      </div>
    </div>

    <!-- Quick Action Bar -->
    <div class="flex items-center justify-between px-4 py-2 bg-gray-50 dark:bg-gray-900">
      <div 
        class="flex items-center space-x-2 text-sm text-gray-600 dark:text-gray-400"
        aria-live="polite"
        aria-atomic="true"
      >
        <span v-if="searchState.loading">Searching...</span>
        <span v-else-if="searchState.results.length > 0">
          {{ searchState.results.length }} of {{ searchState.pagination.total }} results
        </span>
        <span v-else-if="searchState.hasSearched">No results found</span>
        <span v-else>Enter search terms</span>
      </div>
      
      <div class="flex items-center space-x-1">
        <button 
          v-if="query || hasActiveFilters" 
          @click="clearAll" 
          :aria-label="'Clear all'"
          class="p-1 text-gray-400 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors duration-200"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
          </svg>
        </button>
      </div>
    </div>

    <!-- Loading Skeleton -->
    <div v-if="searchState.loading" class="p-4 space-y-4">
      <div v-for="i in 3" :key="i" class="animate-pulse">
        <div class="flex items-start space-x-3">
          <div class="w-8 h-8 bg-gray-300 dark:bg-gray-600 rounded-full"></div>
          <div class="flex-1 space-y-2">
            <div class="h-4 bg-gray-300 dark:bg-gray-600 rounded w-1/4"></div>
            <div class="h-3 bg-gray-300 dark:bg-gray-600 rounded w-3/4"></div>
            <div class="h-3 bg-gray-300 dark:bg-gray-600 rounded w-1/2"></div>
          </div>
        </div>
      </div>
    </div>

    <!-- Results -->
    <div v-else-if="searchState.results.length > 0" class="max-h-96 overflow-y-auto">
      <div 
        v-for="result in searchState.results" 
        :key="result.id" 
        data-testid="result-item"
        class="p-4 border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition-colors duration-200"
        @click="handleResultSelect(result)"
        @keyup.enter="handleResultSelect(result)"
        @keyup.space="handleResultSelect(result)"
        tabindex="0"
        role="button"
        :aria-label="`Select result: ${result.content.slice(0, 50)}`"
      >
        <div class="flex items-start space-x-3">
          <div class="flex-shrink-0">
            <div class="w-8 h-8 bg-indigo-500 rounded-full flex items-center justify-center text-white text-sm font-medium">
              {{ getInitials(result.sender_fullname || result.sender_name || result.sender?.fullname || 'U') }}
            </div>
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-center space-x-2 mb-1">
              <span class="font-medium text-gray-900 dark:text-gray-100">
                {{ result.sender_fullname || result.sender_name || result.sender?.fullname || 'Unknown User' }}
              </span>
              <span class="text-xs text-gray-500 dark:text-gray-400">
                {{ formatTime(result.created_at) }}
              </span>
              <span v-if="result.chat_name" class="text-xs text-indigo-600 dark:text-indigo-400">
                #{{ result.chat_name }}
              </span>
            </div>
            <p 
              class="text-sm text-gray-800 dark:text-gray-200 line-clamp-2" 
              v-html="highlightSearchTerms(result.content)"
            ></p>
            <!-- 显示附件信息 -->
            <div v-if="result.files && result.files.length > 0" class="mt-1 flex items-center space-x-2">
              <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"></path>
              </svg>
              <span class="text-xs text-gray-500">{{ result.files.length }} file(s)</span>
            </div>
          </div>
        </div>
      </div>
      
      <!-- Load More Button -->
      <div v-if="searchState.pagination.hasMore" class="p-4 text-center">
        <button 
          @click="loadMore"
          :disabled="searchState.loading"
          class="px-4 py-2 text-sm text-indigo-600 dark:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-900 rounded-md disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200"
        >
          Load More
        </button>
      </div>
    </div>

    <!-- No Results -->
    <div v-else-if="searchState.hasSearched" class="p-8 text-center">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">
        No messages found
      </h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        Try adjusting your search criteria
      </p>
    </div>

    <!-- Error State -->
    <div v-if="searchState.error" class="p-4 bg-red-50 dark:bg-red-900 border-l-4 border-red-400">
      <div class="flex">
        <svg class="h-5 w-5 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
        </svg>
        <div class="ml-3">
          <p class="text-sm text-red-700 dark:text-red-200">
            {{ searchState.error }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { SearchService } from '@/services/api'

// Props and Emits
const props = defineProps({
  users: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['select', 'search'])

// State management
const searchState = ref({
  loading: false,
  hasSearched: false,
  results: [],
  error: null,
  pagination: {
    total: 0,
    hasMore: false,
    page: 1
  }
})

const query = ref('')
const timeRange = ref('all')
const messageType = ref('all')
const fromUser = ref('')
const sortBy = ref('newest')

// Computed
const hasActiveFilters = computed(() => {
  return timeRange.value !== 'all' || 
         messageType.value !== 'all' || 
         fromUser.value !== '' ||
         sortBy.value !== 'newest'
})

const canSearch = computed(() => {
  return query.value.trim().length > 0
})

// Methods
const search = async () => {
  if (!canSearch.value) return
  
  searchState.value.loading = true
  searchState.value.error = null
  searchState.value.hasSearched = true
  
  try {
    // 使用真实的搜索 API
    const response = await SearchService.search({
      query: query.value.trim(),
      timeRange: timeRange.value,
      messageType: messageType.value,
      fromUser: fromUser.value,
      sortBy: sortBy.value,
      page: 1,
      limit: 20
    })

    // 更新搜索结果
    searchState.value.results = response.results
    searchState.value.pagination = {
      total: response.total,
      hasMore: response.hasMore,
      page: response.page
    }

    // 保存搜索历史 - 注释掉，因为后端暂不支持
    // if (response.results.length > 0) {
    //   SearchService.saveSearch(query.value, response.results).catch(err => {
    //     console.warn('Failed to save search history:', err)
    //   })
    // }

    // 发出搜索事件
    emit('search', {
      query: query.value,
      results: response.results,
      filters: {
        timeRange: timeRange.value,
        messageType: messageType.value,
        fromUser: fromUser.value,
        sortBy: sortBy.value
      }
    })

  } catch (error) {
    console.error('Search failed:', error)
    searchState.value.error = error.response?.data?.message || 'Search failed. Please try again.'
    searchState.value.results = []
  } finally {
    searchState.value.loading = false
  }

const loadMore = async () => {
  if (!searchState.value.pagination.hasMore || searchState.value.loading) return
  
  searchState.value.loading = true
  
  try {
    const nextPage = searchState.value.pagination.page + 1
    const response = await SearchService.search({
      query: query.value.trim(),
      timeRange: timeRange.value,
      messageType: messageType.value,
      fromUser: fromUser.value,
      sortBy: sortBy.value,
      page: nextPage,
      limit: 20
    })

    // 追加新结果
    searchState.value.results.push(...response.results)
    searchState.value.pagination = {
      total: response.total,
      hasMore: response.hasMore,
      page: response.page
    }

  } catch (error) {
    console.error('Load more failed:', error)
    searchState.value.error = 'Failed to load more results.'
  } finally {
    searchState.value.loading = false
  }

const clearFilters = () => {
  timeRange.value = 'all'
  messageType.value = 'all'
  fromUser.value = ''
  sortBy.value = 'newest'
}

const clearAll = () => {
  query.value = ''
  clearFilters()
  searchState.value.hasSearched = false
  searchState.value.results = []
  searchState.value.pagination = {
    total: 0,
    hasMore: false,
    page: 1
  }

const highlightSearchTerms = (text) => {
  if (!query.value) return text
  // 转义特殊正则字符
  const escapedQuery = query.value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const regex = new RegExp(`(${escapedQuery})`, 'gi')
  return text.replace(regex, '<mark class="bg-yellow-200 dark:bg-yellow-800">$1</mark>')
}

const getInitials = (name) => {
  return name.split(' ').map(n => n[0]).join('').toUpperCase().slice(0, 2)
}

const formatTime = (timestamp) => {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now - date
  
  if (diff < 60000) return 'just now'
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`
  return date.toLocaleDateString()
}

// Local state
const showFilters = ref(false)
const availableUsers = ref([])

// Methods
const toggleFilters = () => {
  showFilters.value = !showFilters.value
}

const handleResultSelect = (result) => {
  emit('select', result)
}

// Load available users
onMounted(async () => {
  try {
    // Load from props first
    if (props.users.length > 0) {
      availableUsers.value = props.users
      return
    }

    // Fallback to API or mock data
    availableUsers.value = [
      { id: 1, fullname: 'John Doe' },
      { id: 2, fullname: 'Jane Smith' },
      { id: 3, fullname: 'Bob Johnson' }
    ]
  } catch (error) {
    console.error('Failed to load users:', error)
  }
})
</script>

<style scoped>
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* Focus styles for accessibility */
[role="button"]:focus {
  outline: 2px solid theme('colors.indigo.500');
  outline-offset: 2px;
}

/* Smooth transitions */
.transition-colors {
  transition-property: color, background-color, border-color;
}

/* Custom scrollbar for results */
.overflow-y-auto::-webkit-scrollbar {
  width: 6px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  @apply bg-gray-100 dark:bg-gray-800;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  @apply bg-gray-300 dark:bg-gray-600 rounded-full;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  @apply bg-gray-400 dark:bg-gray-500;
}

/* High contrast mode support */
@media (prefers-contrast: high) {
  .border-gray-200 {
    @apply border-gray-900;
  }
  
  .text-gray-400 {
    @apply text-gray-700;
  }

/* Reduced motion support */
@media (prefers-reduced-motion: reduce) {
  .transition-colors,
  .animate-pulse {
    transition: none;
    animation: none;
  }
</style>