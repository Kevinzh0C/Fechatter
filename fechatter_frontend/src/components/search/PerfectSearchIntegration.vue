<template>
  <div class="perfect-search-integration">
    <!-- Search Trigger Button -->
    <button 
      @click="openSearch"
      class="search-trigger-button"
      :class="{ 'has-new-messages': hasNewMessages }"
    >
      <div class="search-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <circle cx="11" cy="11" r="8"/>
          <path d="m21 21-4.35-4.35"/>
        </svg>
      </div>
      <span class="search-label">Search Messages</span>
      <div class="search-shortcut">
        <kbd>⌘</kbd><kbd>K</kbd>
      </div>
    </button>

    <!-- Perfect Search Modal -->
    <PerfectSearchModal
      :is-open="isSearchOpen"
      :chat-id="currentChatId"
      @close="closeSearch"
      @navigate="handleNavigateToMessage"
    />

    <!-- Background Search Status -->
    <div v-if="showStatus" class="search-status">
      <div class="status-icon">
        <svg v-if="searchStatus === 'indexing'" class="animate-spin" viewBox="0 0 24 24">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none" opacity="0.25"/>
          <path fill="currentColor" d="M4 12A8 8 0 018 4v2.5l1.5-1.5L8 6.5V4A8 8 0 0112 4v2.5" opacity="0.75"/>
        </svg>
        <svg v-else-if="searchStatus === 'ready'" viewBox="0 0 24 24">
          <path fill="currentColor" d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z"/>
        </svg>
      </div>
      <span class="status-text">
        {{ searchStatus === 'indexing' ? 'Indexing messages...' : 'Search ready' }}
      </span>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useChatStore } from '@/stores/chat'
import PerfectSearchModal from './PerfectSearchModal.vue'
import perfectSearchService from '@/services/perfectSearchService'

// Router and stores
const router = useRouter()
const chatStore = useChatStore()

// State
const isSearchOpen = ref(false)
const searchStatus = ref('ready') // 'ready', 'indexing', 'error'
const hasNewMessages = ref(false)
const showStatus = ref(false)

// Computed
const currentChatId = computed(() => {
  return chatStore.currentChat?.id || null
})

// Methods
const openSearch = () => {
  isSearchOpen.value = true
}

const closeSearch = () => {
  isSearchOpen.value = false
}

const handleNavigateToMessage = async (result) => {
  try {
    // Navigate to the chat if not already there
    if (result.chat_id && result.chat_id !== currentChatId.value) {
      await router.push(`/chat/${result.chat_id}`)
    }
    
    // Scroll to the specific message
    if (result.id) {
      // Wait for navigation to complete
      await new Promise(resolve => setTimeout(resolve, 100))
      
      // Scroll to message
      const messageElement = document.getElementById(`message-${result.id}`)
      if (messageElement) {
        messageElement.scrollIntoView({ 
          behavior: 'smooth', 
          block: 'center' 
        })
        
        // Highlight the message briefly
        messageElement.classList.add('highlighted-message')
        setTimeout(() => {
          messageElement.classList.remove('highlighted-message')
        }, 3000)
      }
    }
    
    closeSearch()
  } catch (error) {
    console.error('Failed to navigate to message:', error)
  }
}

const handleGlobalKeydown = (event) => {
  // ⌘K or Ctrl+K to open search
  if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
    event.preventDefault()
    if (!isSearchOpen.value) {
      openSearch()
    }
  }
  
  // Escape to close search
  if (event.key === 'Escape' && isSearchOpen.value) {
    closeSearch()
  }
}

const initializeSearch = async () => {
  try {
    showStatus.value = true
    searchStatus.value = 'indexing'
    
    // Initialize perfect search service
    await perfectSearchService.initialize()
    
    searchStatus.value = 'ready'
    
    // Hide status after 2 seconds
    setTimeout(() => {
      showStatus.value = false
    }, 2000)
    
  } catch (error) {
    console.error('Failed to initialize search:', error)
    searchStatus.value = 'error'
    
    setTimeout(() => {
      showStatus.value = false
    }, 5000)
  }
}

// Lifecycle
onMounted(() => {
  // Add global keyboard listeners
  document.addEventListener('keydown', handleGlobalKeydown)
  
  // Initialize search
  initializeSearch()
  
  // Listen for new messages to update search indicator
  chatStore.$subscribe((mutation, state) => {
    if (mutation.events?.some(e => e.type === 'add' && e.key === 'messages')) {
      hasNewMessages.value = true
      
      // Reset indicator after 5 seconds
      setTimeout(() => {
        hasNewMessages.value = false
      }, 5000)
    }
  })
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleGlobalKeydown)
})

// Expose methods for parent components
defineExpose({
  openSearch,
  closeSearch
})
</script>

<style scoped>
/* Perfect Search Integration - Elegant & Accessible */

.perfect-search-integration {
  position: relative;
}

/* Search Trigger Button - Minimalist Design */
.search-trigger-button {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.2, 0, 0, 1);
  
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  font-size: 14px;
  color: #374151;
  
  position: relative;
  overflow: hidden;
}

.search-trigger-button:hover {
  background: rgba(255, 255, 255, 1);
  border-color: #007aff;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.search-trigger-button:active {
  transform: translateY(0);
}

.search-trigger-button.has-new-messages::before {
  content: '';
  position: absolute;
  top: 8px;
  right: 8px;
  width: 6px;
  height: 6px;
  background: #007aff;
  border-radius: 50%;
  animation: pulse-glow 2s infinite;
}

@keyframes pulse-glow {
  0%, 100% { 
    opacity: 1;
    transform: scale(1);
  }
  50% { 
    opacity: 0.5;
    transform: scale(1.2);
  }
}

.search-icon {
  width: 18px;
  height: 18px;
  color: #6b7280;
  transition: color 0.2s ease;
}

.search-trigger-button:hover .search-icon {
  color: #007aff;
}

.search-label {
  font-weight: 500;
  flex: 1;
}

.search-shortcut {
  display: flex;
  gap: 2px;
  opacity: 0.6;
}

.search-shortcut kbd {
  background: rgba(0, 0, 0, 0.05);
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 4px;
  padding: 2px 6px;
  font-size: 11px;
  font-family: monospace;
  color: #6b7280;
}

/* Search Status - Subtle Feedback */
.search-status {
  position: fixed;
  bottom: 24px;
  right: 24px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: rgba(0, 0, 0, 0.8);
  color: white;
  border-radius: 12px;
  font-size: 13px;
  font-weight: 500;
  z-index: 1000;
  
  backdrop-filter: blur(20px);
  animation: slide-in-up 0.3s cubic-bezier(0.2, 0, 0, 1);
}

@keyframes slide-in-up {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.status-icon {
  width: 16px;
  height: 16px;
  color: #10b981;
}

.animate-spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Global Message Highlight Styles */
:global(.highlighted-message) {
  background: rgba(0, 122, 255, 0.1) !important;
  border-left: 3px solid #007aff !important;
  animation: highlight-pulse 3s ease-out;
}

@keyframes highlight-pulse {
  0% {
    background: rgba(0, 122, 255, 0.2);
  }
  100% {
    background: rgba(0, 122, 255, 0.05);
  }
}

/* Responsive Design */
@media (max-width: 640px) {
  .search-trigger-button {
    padding: 10px 14px;
    font-size: 13px;
  }
  
  .search-shortcut {
    display: none;
  }
  
  .search-status {
    bottom: 16px;
    right: 16px;
    left: 16px;
    font-size: 12px;
  }
}

/* Dark Mode Support */
@media (prefers-color-scheme: dark) {
  .search-trigger-button {
    background: rgba(0, 0, 0, 0.8);
    border-color: rgba(255, 255, 255, 0.1);
    color: #e5e7eb;
  }
  
  .search-trigger-button:hover {
    background: rgba(0, 0, 0, 0.9);
    border-color: #007aff;
  }
  
  .search-shortcut kbd {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
    color: #9ca3af;
  }
}

/* High Contrast Support */
@media (prefers-contrast: high) {
  .search-trigger-button {
    border-width: 2px;
  }
  
  .search-trigger-button:hover {
    border-width: 2px;
    border-color: #0051d5;
  }
}

/* Reduced Motion Support */
@media (prefers-reduced-motion: reduce) {
  .search-trigger-button,
  .search-status,
  .status-icon {
    transition: none !important;
    animation: none !important;
  }
  
  .search-trigger-button.has-new-messages::before {
    animation: none;
  }
}
</style> 