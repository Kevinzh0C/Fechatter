<template>
  <div class="search-results">
    <!-- Search Header -->
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-lg font-semibold text-gray-900">
        Search Results
        <span v-if="results.length > 0" class="text-sm font-normal text-gray-500">
          ({{ results.length }} {{ results.length === 1 ? 'result' : 'results' }})
        </span>
      </h3>
      <div class="flex items-center space-x-2">
        <button @click="$emit('clear')" 
          class="text-sm text-gray-500 hover:text-gray-700">
          Clear
        </button>
        <button @click="$emit('close')" 
          class="text-gray-400 hover:text-gray-600">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
          </svg>
        </button>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="text-center py-8">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-600 mx-auto"></div>
      <p class="mt-2 text-gray-500">Searching messages...</p>
    </div>

    <!-- Empty State -->
    <div v-else-if="results.length === 0 && query" class="text-center py-8">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900">No messages found</h3>
      <p class="mt-1 text-sm text-gray-500">
        No messages found for "{{ query }}". Try different keywords.
      </p>
    </div>

    <!-- Search Results -->
    <div v-else-if="results.length > 0" class="space-y-3 max-h-96 overflow-y-auto">
      <div v-for="message in results" :key="message.id"
        class="bg-gray-50 rounded-lg p-4 hover:bg-gray-100 cursor-pointer transition-colors"
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
            <span v-if="message.relevance_score" class="bg-purple-100 text-purple-800 px-2 py-1 rounded">
              {{ Math.round(message.relevance_score * 100) }}% match
            </span>
          </div>
        </div>

        <!-- Message Content -->
        <div class="text-sm text-gray-700">
          <p v-html="highlightSearchTerm(message.content, query)"></p>
        </div>

        <!-- Files -->
        <div v-if="message.files && message.files.length > 0" class="mt-2">
          <div class="flex flex-wrap gap-2">
            <span v-for="file in message.files" :key="file"
              class="inline-flex items-center px-2 py-1 rounded-full text-xs bg-blue-100 text-blue-800">
              <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"></path>
              </svg>
              {{ getFileName(file) }}
            </span>
          </div>
        </div>

        <!-- Historical Message Indicator -->
        <div v-if="isHistoricalMessage(message)" class="mt-2">
          <span class="inline-flex items-center px-2 py-1 rounded-full text-xs bg-yellow-100 text-yellow-800">
            <svg class="w-3 h-3 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"></path>
            </svg>
            Historical message
          </span>
        </div>
      </div>
    </div>

    <!-- Search Tips -->
    <div v-if="!query" class="text-center py-8">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900">Search Messages</h3>
      <div class="mt-1 text-sm text-gray-500 space-y-1">
        <p>Enter keywords to search through all messages in this chat.</p>
        <p class="text-xs">
          <strong>Tips:</strong> Use quotes for exact phrases, or multiple words for broader search.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  results: {
    type: Array,
    default: () => []
  },
  loading: {
    type: Boolean,
    default: false
  },
  query: {
    type: String,
    default: ''
  }
});

const emit = defineEmits(['close', 'clear', 'select-message']);

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

function isHistoricalMessage(message) {
  // A message is considered historical if it was sent before the user joined
  // This would need to be determined by the backend based on user join date
  // For now, we'll use a simple heuristic based on message age
  const messageDate = new Date(message.created_at);
  const thirtyDaysAgo = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000);
  return messageDate < thirtyDaysAgo;
}
</script>

<style scoped>
.search-results {
  @apply bg-white rounded-lg shadow-lg;
}

/* Custom scrollbar for search results */
.search-results ::-webkit-scrollbar {
  width: 6px;
}

.search-results ::-webkit-scrollbar-track {
  @apply bg-gray-100 rounded;
}

.search-results ::-webkit-scrollbar-thumb {
  @apply bg-gray-300 rounded;
}

.search-results ::-webkit-scrollbar-thumb:hover {
  @apply bg-gray-400;
}
</style> 