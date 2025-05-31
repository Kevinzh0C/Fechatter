<template>
  <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
    <!-- Search Input -->
    <div class="p-4 border-b border-gray-200 dark:border-gray-700">
      <div class="relative">
        <svg class="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
        </svg>
        <input v-model="searchQuery" 
               @keyup.enter="search"
               type="text" 
               placeholder="Search messages..." 
               class="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-purple-500 focus:border-transparent">
        <button @click="toggleFilters" 
                class="absolute right-2 top-1/2 transform -translate-y-1/2 p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4"></path>
          </svg>
        </button>
      </div>
    </div>

    <!-- Compact Filters -->
    <div v-if="showFilters" class="p-3 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 text-sm">
        <!-- Time Range -->
        <select v-model="timeRange" class="px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100">
          <option value="all">All time</option>
          <option value="today">Today</option>
          <option value="week">This week</option>
          <option value="month">This month</option>
        </select>

        <!-- Message Type -->
        <select v-model="messageType" class="px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100">
          <option value="all">All types</option>
          <option value="text">Text only</option>
          <option value="files">With files</option>
          <option value="images">With images</option>
        </select>

        <!-- From -->
        <select v-model="fromUser" class="px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100">
          <option value="">Anyone</option>
          <option v-for="user in users" :key="user.id" :value="user.id">
            {{ user.fullname }}
          </option>
        </select>

        <!-- Sort -->
        <select v-model="sortBy" class="px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100">
          <option value="newest">Newest first</option>
          <option value="oldest">Oldest first</option>
          <option value="relevance">Most relevant</option>
        </select>
      </div>
      
      <div class="flex justify-between items-center mt-3">
        <button @click="clearFilters" class="text-xs text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
          Clear filters
        </button>
        <button @click="search" 
                class="px-3 py-1 bg-purple-600 text-white text-xs rounded hover:bg-purple-700">
          Search
        </button>
      </div>
    </div>

    <!-- Quick Action Bar -->
    <div class="flex items-center justify-between px-4 py-2 bg-gray-50 dark:bg-gray-900">
      <div class="flex items-center space-x-2 text-sm text-gray-600 dark:text-gray-400">
        <span v-if="loading">Searching...</span>
        <span v-else-if="results.length > 0">{{ results.length }} results</span>
        <span v-else-if="hasSearched">No results</span>
        <span v-else>Enter search terms</span>
      </div>
      
      <div class="flex items-center space-x-1">
        <button v-if="searchQuery || hasActiveFilters" 
                @click="clearAll" 
                class="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
          </svg>
        </button>
      </div>
    </div>

    <!-- Results -->
    <div v-if="loading" class="p-8 text-center">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-600 mx-auto"></div>
      <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">Searching...</p>
    </div>

    <div v-else-if="results.length > 0" class="max-h-96 overflow-y-auto">
      <div v-for="result in results" :key="result.id" 
           class="p-4 border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer"
           @click="$emit('select', result)">
        <div class="flex items-start space-x-3">
          <div class="flex-shrink-0">
            <div class="w-8 h-8 bg-purple-500 rounded-full flex items-center justify-center text-white text-sm font-medium">
              {{ getInitials(result.author?.fullname || 'U') }}
            </div>
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-center space-x-2 mb-1">
              <span class="font-medium text-gray-900 dark:text-gray-100">
                {{ result.author?.fullname || 'Unknown User' }}
              </span>
              <span class="text-xs text-gray-500 dark:text-gray-400">
                {{ formatTime(result.created_at) }}
              </span>
              <span class="text-xs text-purple-600 dark:text-purple-400">
                #{{ result.chat?.name }}
              </span>
            </div>
            <p class="text-sm text-gray-800 dark:text-gray-200 line-clamp-2" v-html="highlightSearchTerms(result.content)">
            </p>
          </div>
        </div>
      </div>
    </div>

    <div v-else-if="hasSearched" class="p-8 text-center">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">No messages found</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">Try adjusting your search terms or filters.</p>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';

const emit = defineEmits(['select', 'search']);

const searchQuery = ref('');
const showFilters = ref(false);
const timeRange = ref('all');
const messageType = ref('all');
const fromUser = ref('');
const sortBy = ref('newest');
const results = ref([]);
const loading = ref(false);
const hasSearched = ref(false);
const users = ref([]);

const hasActiveFilters = computed(() => {
  return timeRange.value !== 'all' || 
         messageType.value !== 'all' || 
         fromUser.value !== '' || 
         sortBy.value !== 'newest';
});

function toggleFilters() {
  showFilters.value = !showFilters.value;
}

function clearFilters() {
  timeRange.value = 'all';
  messageType.value = 'all';
  fromUser.value = '';
  sortBy.value = 'newest';
}

function clearAll() {
  searchQuery.value = '';
  clearFilters();
  results.value = [];
  hasSearched.value = false;
}

async function search() {
  if (!searchQuery.value.trim() && !hasActiveFilters.value) return;
  
  loading.value = true;
  hasSearched.value = true;
  
  try {
    const searchParams = {
      query: searchQuery.value,
      timeRange: timeRange.value,
      messageType: messageType.value,
      fromUser: fromUser.value,
      sortBy: sortBy.value
    };
    
    emit('search', searchParams);
    
    // Mock search results for demo
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    results.value = [
      {
        id: 1,
        content: 'Hello world! This is a test message.',
        author: { fullname: 'John Doe' },
        created_at: new Date().toISOString(),
        chat: { name: 'general' }
      },
      {
        id: 2,
        content: 'Another message with some content here.',
        author: { fullname: 'Jane Smith' },
        created_at: new Date(Date.now() - 3600000).toISOString(),
        chat: { name: 'random' }
      }
    ];
  } catch (error) {
    console.error('Search failed:', error);
    results.value = [];
  } finally {
    loading.value = false;
  }
}

function getInitials(name) {
  return name.split(' ').map(n => n[0]).join('').toUpperCase().slice(0, 2);
}

function formatTime(timestamp) {
  const date = new Date(timestamp);
  const now = new Date();
  const diffInHours = (now - date) / (1000 * 60 * 60);
  
  if (diffInHours < 1) {
    return 'Just now';
  } else if (diffInHours < 24) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  } else {
    return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
  }
}

function highlightSearchTerms(content) {
  if (!searchQuery.value) return content;
  
  const regex = new RegExp(`(${searchQuery.value})`, 'gi');
  return content.replace(regex, '<mark class="bg-yellow-200 dark:bg-yellow-800">$1</mark>');
}

onMounted(() => {
  // Load users for filter dropdown
  users.value = [
    { id: 1, fullname: 'John Doe' },
    { id: 2, fullname: 'Jane Smith' },
    { id: 3, fullname: 'Bob Johnson' }
  ];
});
</script>

<style scoped>
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style> 