<template>
  <div class="advanced-search">
    <!-- Search Input -->
    <div class="search-input-section">
      <div class="relative">
        <input 
          v-model="searchQuery"
          @keyup.enter="performSearch"
          @input="onSearchInput"
          placeholder="Search messages..."
          class="w-full pl-10 pr-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent text-sm"
          autofocus />
        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
          <svg class="h-5 w-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
          </svg>
        </div>
      </div>
    </div>

    <!-- Search Filters -->
    <div class="search-filters">
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <!-- Time Range Filter -->
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">Time Range</label>
          <select v-model="filters.timeRange" 
            class="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:ring-1 focus:ring-purple-500 focus:border-purple-500">
            <option value="all">All time</option>
            <option value="today">Today</option>
            <option value="week">This week</option>
            <option value="month">This month</option>
            <option value="quarter">Last 3 months</option>
            <option value="year">This year</option>
          </select>
        </div>

        <!-- Message Type Filter -->
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">Message Type</label>
          <select v-model="filters.messageType" 
            class="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:ring-1 focus:ring-purple-500 focus:border-purple-500">
            <option value="all">All messages</option>
            <option value="text">Text only</option>
            <option value="files">With files</option>
            <option value="images">With images</option>
            <option value="documents">With documents</option>
          </select>
        </div>

        <!-- Sender Filter -->
        <div>
          <label class="block text-xs font-medium text-gray-700 mb-1">From</label>
          <select v-model="filters.senderId" 
            class="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:ring-1 focus:ring-purple-500 focus:border-purple-500">
            <option value="">Anyone</option>
            <option v-for="member in chatMembers" :key="member.id" :value="member.id">
              {{ member.fullname }}
            </option>
          </select>
        </div>
      </div>

      <!-- Sort Options -->
      <div class="flex items-center justify-between mt-4">
        <div class="flex items-center space-x-4">
          <div>
            <label class="block text-xs font-medium text-gray-700 mb-1">Sort by</label>
            <select v-model="sortBy" 
              class="px-3 py-2 text-sm border border-gray-300 rounded-md focus:ring-1 focus:ring-purple-500 focus:border-purple-500">
              <option value="date">Date</option>
              <option value="relevance">Relevance</option>
            </select>
          </div>
          <div>
            <label class="block text-xs font-medium text-gray-700 mb-1">Order</label>
            <select v-model="sortOrder" 
              class="px-3 py-2 text-sm border border-gray-300 rounded-md focus:ring-1 focus:ring-purple-500 focus:border-purple-500">
              <option value="desc">Newest first</option>
              <option value="asc">Oldest first</option>
            </select>
          </div>
        </div>

        <!-- Search Actions -->
        <div class="flex items-center space-x-2">
          <button @click="clearFilters" 
            class="px-3 py-2 text-sm text-gray-600 hover:text-gray-800">
            Clear
          </button>
          <button @click="performSearch"
            :disabled="!searchQuery.trim() || loading"
            class="px-4 py-2 bg-purple-600 text-white text-sm rounded-md hover:bg-purple-700 disabled:opacity-50 disabled:cursor-not-allowed">
            <span v-if="loading">Searching...</span>
            <span v-else>Search</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Search Results -->
    <div class="search-results-section">
      <!-- Results Header -->
      <div v-if="searchQuery && !loading" class="flex items-center justify-between py-3 border-b border-gray-200">
        <div class="text-sm text-gray-600">
          <span v-if="results.length > 0">
            {{ results.length }} {{ results.length === 1 ? 'result' : 'results' }} for "{{ searchQuery }}"
          </span>
          <span v-else>
            No results found for "{{ searchQuery }}"
          </span>
        </div>
        <button @click="$emit('close')" 
          class="text-gray-400 hover:text-gray-600">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
          </svg>
        </button>
      </div>

      <!-- Loading State -->
      <div v-if="loading" class="text-center py-8">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-600 mx-auto"></div>
        <p class="mt-2 text-sm text-gray-500">Searching messages...</p>
      </div>

      <!-- Results List -->
      <div v-else-if="results.length > 0" class="space-y-3 max-h-96 overflow-y-auto py-4">
        <div v-for="message in results" :key="message.id"
          class="bg-gray-50 rounded-lg p-4 hover:bg-gray-100 cursor-pointer transition-colors border border-gray-200"
          @click="$emit('select-message', message)">
          
          <!-- Message Header -->
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center space-x-2">
              <div class="w-6 h-6 bg-purple-500 rounded-full flex items-center justify-center">
                <span class="text-xs font-medium text-white">
                  {{ message.sender_name ? message.sender_name.charAt(0).toUpperCase() : 'U' }}
                </span>
              </div>
              <span class="text-sm font-medium text-gray-900">
                {{ message.sender_name || `User ${message.sender_id}` }}
              </span>
            </div>
            <div class="flex items-center space-x-2 text-xs text-gray-500">
              <span>{{ formatDate(message.created_at) }}</span>
              <span v-if="message.relevance_score && sortBy === 'relevance'" 
                class="bg-purple-100 text-purple-800 px-2 py-1 rounded">
                {{ Math.round(message.relevance_score * 100) }}% match
              </span>
            </div>
          </div>

          <!-- Message Content -->
          <div class="text-sm text-gray-700 mb-2">
            <p v-html="highlightSearchTerm(message.content, searchQuery)"></p>
          </div>

          <!-- Files -->
          <div v-if="message.files && message.files.length > 0" class="flex flex-wrap gap-2">
            <span v-for="file in message.files" :key="file"
              class="inline-flex items-center px-2 py-1 rounded-full text-xs bg-blue-100 text-blue-800">
              <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"></path>
              </svg>
              {{ getFileName(file) }}
            </span>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else-if="searchQuery && !loading" class="text-center py-8">
        <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
        </svg>
        <h3 class="mt-2 text-sm font-medium text-gray-900">No messages found</h3>
        <p class="mt-1 text-sm text-gray-500">
          Try adjusting your search terms or filters.
        </p>
      </div>

      <!-- Search Tips -->
      <div v-else class="text-center py-8">
        <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
        </svg>
        <h3 class="mt-2 text-sm font-medium text-gray-900">Search Messages</h3>
        <div class="mt-1 text-sm text-gray-500 space-y-1">
          <p>Enter keywords to search through messages in this chat.</p>
          <p class="text-xs">Use filters to narrow down your search results.</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue';

const props = defineProps({
  results: {
    type: Array,
    default: () => []
  },
  loading: {
    type: Boolean,
    default: false
  },
  chatMembers: {
    type: Array,
    default: () => []
  }
});

const emit = defineEmits(['search', 'close', 'select-message']);

const searchQuery = ref('');
const sortBy = ref('date');
const sortOrder = ref('desc');

const filters = ref({
  timeRange: 'all',
  messageType: 'all',
  senderId: ''
});

const results = computed(() => props.results);

function performSearch() {
  if (!searchQuery.value.trim()) return;
  
  const searchParams = {
    query: searchQuery.value,
    filters: filters.value,
    sortBy: sortBy.value,
    sortOrder: sortOrder.value
  };
  
  emit('search', searchParams);
}

function onSearchInput() {
  // Auto-search after a delay when user stops typing
  clearTimeout(searchTimeout);
  if (searchQuery.value.trim()) {
    searchTimeout = setTimeout(performSearch, 500);
  }
}

let searchTimeout;

function clearFilters() {
  searchQuery.value = '';
  filters.value = {
    timeRange: 'all',
    messageType: 'all',
    senderId: ''
  };
  sortBy.value = 'date';
  sortOrder.value = 'desc';
}

function formatDate(dateString) {
  const date = new Date(dateString);
  const now = new Date();
  const diffInHours = (now - date) / (1000 * 60 * 60);
  
  if (diffInHours < 1) {
    return 'Just now';
  } else if (diffInHours < 24) {
    return `${Math.floor(diffInHours)}h ago`;
  } else if (diffInHours < 24 * 7) {
    const days = Math.floor(diffInHours / 24);
    return `${days}d ago`;
  } else {
    return date.toLocaleDateString([], { 
      month: 'short', 
      day: 'numeric',
      year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined
    });
  }
}

function highlightSearchTerm(content, searchTerm) {
  if (!searchTerm || !content) return content;
  
  const regex = new RegExp(`(${escapeRegExp(searchTerm)})`, 'gi');
  return content.replace(regex, '<mark class="bg-yellow-200 px-1 rounded">$1</mark>');
}

function escapeRegExp(string) {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function getFileName(filePath) {
  return filePath.split('/').pop() || filePath;
}

// Watch for filter changes and auto-search
watch([filters, sortBy, sortOrder], () => {
  if (searchQuery.value.trim()) {
    performSearch();
  }
}, { deep: true });
</script>

<style scoped>
.advanced-search {
  @apply bg-white rounded-lg shadow-lg p-6 space-y-6;
}

.search-input-section {
  @apply border-b border-gray-200 pb-4;
}

.search-filters {
  @apply border-b border-gray-200 pb-4;
}

.search-results-section {
  @apply flex-1 min-h-0;
}

/* Custom scrollbar */
.search-results-section ::-webkit-scrollbar {
  width: 6px;
}

.search-results-section ::-webkit-scrollbar-track {
  @apply bg-gray-100 rounded;
}

.search-results-section ::-webkit-scrollbar-thumb {
  @apply bg-gray-300 rounded;
}

.search-results-section ::-webkit-scrollbar-thumb:hover {
  @apply bg-gray-400;
}
</style> 