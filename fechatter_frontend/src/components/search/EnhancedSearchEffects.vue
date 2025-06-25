<template>
  <div class="enhanced-search-effects">
    <!-- 🎯 Enhanced Search Navigation Bar -->
    <div v-if="isSearchActive" class="search-navigation-bar">
      <div class="search-info">
        <div class="search-status">
          <div class="status-indicator" :class="searchStatusClass">
            <svg v-if="isSearching" class="loading-icon" viewBox="0 0 24 24">
              <circle cx="12" cy="12" r="10" fill="none" stroke="currentColor" stroke-width="2" />
            </svg>
            <svg v-else class="check-icon" viewBox="0 0 24 24">
              <path fill="currentColor" d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
            </svg>
            <span class="status-text">{{ searchStatusText }}</span>
          </div>
          <div class="search-metrics">
            <span class="result-count">{{ totalResults }} results</span>
            <span class="search-time">{{ searchTime }}ms</span>
          </div>
        </div>
      </div>

      <div class="navigation-controls">
        <button @click="navigateToPrevious" :disabled="currentIndex <= 0" class="nav-button prev-button"
          title="Previous result (↑)">
          <svg viewBox="0 0 24 24">
            <path d="M15 19l-7-7 7-7" />
          </svg>
        </button>

        <div class="result-counter">
          <span class="current-position">{{ currentIndex + 1 }}</span>
          <span class="separator">/</span>
          <span class="total-count">{{ totalResults }}</span>
        </div>

        <button @click="navigateToNext" :disabled="currentIndex >= totalResults - 1" class="nav-button next-button"
          title="Next result (↓)">
          <svg viewBox="0 0 24 24">
            <path d="M9 5l7 7-7 7" />
          </svg>
        </button>
      </div>

      <div class="search-actions">
        <button @click="toggleSearchMode" class="mode-toggle" :class="{ active: useAdvancedMode }">
          <svg viewBox="0 0 24 24">
            <path
              d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z" />
          </svg>
        </button>
        <button @click="clearSearch" class="clear-button" title="Clear search (Esc)">
          <svg viewBox="0 0 24 24">
            <path
              d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
          </svg>
        </button>
      </div>
    </div>

    <!-- 🌟 Search Progress Indicator -->
    <div v-if="isSearching" class="search-progress">
      <div class="progress-bar" :style="{ width: `${searchProgress}%` }"></div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useSearchStore } from '@/stores/search'
import { useMessageNavigation } from '@/composables/useMessageNavigation'

// Props
const props = defineProps({
  searchQuery: {
    type: String,
    default: ''
  },
  searchResults: {
    type: Array,
    default: () => []
  },
  currentResultIndex: {
    type: Number,
    default: 0
  },
  isSearchActive: {
    type: Boolean,
    default: false
  },
  isSearching: {
    type: Boolean,
    default: false
  },
  searchTime: {
    type: Number,
    default: 0
  }
})

// Emits
const emit = defineEmits([
  'navigate-to-result',
  'clear-search',
  'toggle-mode',
  'search-mode-changed'
])

// State
const searchStore = useSearchStore()
const { navigateToMessage, highlightMessage } = useMessageNavigation()

const currentIndex = ref(props.currentResultIndex)
const searchProgress = ref(0)
const useAdvancedMode = ref(false)

// Computed
const totalResults = computed(() => props.searchResults.length)

const searchStatusClass = computed(() => ({
  'status-searching': props.isSearching,
  'status-success': !props.isSearching && totalResults.value > 0,
  'status-empty': !props.isSearching && totalResults.value === 0,
  'status-error': searchStore.hasError
}))

const searchStatusText = computed(() => {
  if (props.isSearching) return 'Searching...'
  if (searchStore.hasError) return 'Search failed'
  if (totalResults.value === 0) return 'No results'
  return 'Search complete'
})

// Methods
const navigateToPrevious = () => {
  if (currentIndex.value > 0) {
    currentIndex.value--
    navigateToResult(currentIndex.value)
  }
}

const navigateToNext = () => {
  if (currentIndex.value < totalResults.value - 1) {
    currentIndex.value++
    navigateToResult(currentIndex.value)
  }
}

const navigateToResult = (index) => {
  const result = props.searchResults[index]
  if (result) {
    emit('navigate-to-result', {
      result,
      index,
      highlightOptions: {
        type: useAdvancedMode.value ? 'premium' : 'standard',
        duration: 3000,
        showIndicator: true
      }
    })
  }
}

const clearSearch = () => {
  currentIndex.value = 0
  emit('clear-search')
}

const toggleSearchMode = () => {
  useAdvancedMode.value = !useAdvancedMode.value
  emit('toggle-mode', useAdvancedMode.value)

  // Re-highlight current result with new mode
  if (totalResults.value > 0) {
    navigateToResult(currentIndex.value)
  }
}

// Search progress simulation
const simulateSearchProgress = () => {
  if (!props.isSearching) return

  searchProgress.value = 0
  const interval = setInterval(() => {
    searchProgress.value += Math.random() * 30
    if (searchProgress.value >= 100 || !props.isSearching) {
      searchProgress.value = 100
      clearInterval(interval)
    }
  }, 100)
}

// Keyboard navigation
const handleKeydown = (event) => {
  if (!props.isSearchActive) return

  switch (event.key) {
    case 'ArrowUp':
      event.preventDefault()
      navigateToPrevious()
      break
    case 'ArrowDown':
      event.preventDefault()
      navigateToNext()
      break
    case 'Escape':
      event.preventDefault()
      clearSearch()
      break
    case 'Enter':
      event.preventDefault()
      if (totalResults.value > 0) {
        navigateToResult(currentIndex.value)
      }
      break
  }
}

// Watchers
watch(() => props.isSearching, (newValue) => {
  if (newValue) {
    simulateSearchProgress()
  }
})

watch(() => props.currentResultIndex, (newValue) => {
  currentIndex.value = newValue
})

watch(() => props.searchResults, () => {
  currentIndex.value = 0
})

// Lifecycle
onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
/* 🎯 Enhanced Search Navigation Bar */
.enhanced-search-effects {
  position: relative;
  z-index: 50;
}

.search-navigation-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  background: linear-gradient(135deg,
      rgba(255, 255, 255, 0.95) 0%,
      rgba(248, 250, 252, 0.9) 100%);
  border: 1px solid rgba(0, 122, 255, 0.15);
  border-radius: 12px;
  backdrop-filter: blur(20px);
  box-shadow:
    0 8px 32px rgba(0, 122, 255, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
  margin-bottom: 16px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.search-navigation-bar:hover {
  box-shadow:
    0 12px 40px rgba(0, 122, 255, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.9);
  transform: translateY(-1px);
}

/* 📊 Search Info Section */
.search-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.search-status {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 600;
  transition: all 0.3s ease;
}

.status-indicator.status-searching {
  background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
  color: white;
}

.status-indicator.status-success {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  color: white;
}

.status-indicator.status-empty {
  background: linear-gradient(135deg, #6b7280 0%, #4b5563 100%);
  color: white;
}

.status-indicator.status-error {
  background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
  color: white;
}

.loading-icon,
.check-icon {
  width: 14px;
  height: 14px;
}

.loading-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

.search-metrics {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: #64748b;
}

.result-count {
  padding: 4px 8px;
  background: rgba(0, 122, 255, 0.1);
  border-radius: 12px;
  font-weight: 500;
}

.search-time {
  padding: 4px 8px;
  background: rgba(245, 158, 11, 0.1);
  border-radius: 12px;
  font-weight: 500;
}

/* 🎯 Navigation Controls */
.navigation-controls {
  display: flex;
  align-items: center;
  gap: 12px;
  background: rgba(0, 122, 255, 0.08);
  padding: 8px;
  border-radius: 10px;
  border: 1px solid rgba(0, 122, 255, 0.12);
}

.nav-button {
  width: 32px;
  height: 32px;
  border: none;
  background: linear-gradient(135deg, #007aff 0%, #5856d6 100%);
  color: white;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 2px 8px rgba(0, 122, 255, 0.3);
}

.nav-button:hover:not(:disabled) {
  background: linear-gradient(135deg, #0056b3 0%, #4338ca 100%);
  transform: translateY(-1px) scale(1.05);
  box-shadow: 0 4px 12px rgba(0, 122, 255, 0.4);
}

.nav-button:active:not(:disabled) {
  transform: translateY(0) scale(0.98);
}

.nav-button:disabled {
  background: linear-gradient(135deg, #e2e8f0 0%, #cbd5e1 100%);
  color: #94a3b8;
  cursor: not-allowed;
  box-shadow: none;
  transform: none;
}

.nav-button svg {
  width: 16px;
  height: 16px;
  stroke: currentColor;
  stroke-width: 2;
  fill: none;
}

.result-counter {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  background: linear-gradient(135deg, #007aff 0%, #5856d6 100%);
  color: white;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 700;
  min-width: 60px;
  justify-content: center;
  box-shadow: 0 4px 12px rgba(0, 122, 255, 0.3);
}

.current-position {
  color: #ffffff;
}

.separator {
  color: rgba(255, 255, 255, 0.7);
  margin: 0 2px;
}

.total-count {
  color: rgba(255, 255, 255, 0.9);
}

/* ⚙️ Search Actions */
.search-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.mode-toggle,
.clear-button {
  width: 36px;
  height: 36px;
  border: none;
  background: rgba(107, 114, 128, 0.1);
  color: #6b7280;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.mode-toggle:hover,
.clear-button:hover {
  background: rgba(107, 114, 128, 0.2);
  color: #374151;
  transform: scale(1.05);
}

.mode-toggle.active {
  background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%);
  color: white;
  box-shadow: 0 4px 12px rgba(139, 92, 246, 0.3);
}

.mode-toggle svg,
.clear-button svg {
  width: 18px;
  height: 18px;
  fill: currentColor;
}

/* 🌊 Search Progress */
.search-progress {
  height: 3px;
  background: rgba(0, 122, 255, 0.1);
  border-radius: 2px;
  overflow: hidden;
  margin-bottom: 12px;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg,
      #007aff 0%,
      #5856d6 50%,
      #007aff 100%);
  border-radius: 2px;
  transition: width 0.3s ease;
  position: relative;
}

.progress-bar::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(90deg,
      transparent 0%,
      rgba(255, 255, 255, 0.3) 50%,
      transparent 100%);
  animation: progressShine 2s ease-in-out infinite;
}

@keyframes progressShine {
  0% {
    transform: translateX(-100%);
  }

  100% {
    transform: translateX(100%);
  }
}

/* 📱 Responsive Design */
@media (max-width: 768px) {
  .search-navigation-bar {
    flex-direction: column;
    gap: 12px;
    padding: 16px;
  }

  .search-info {
    justify-content: center;
  }

  .navigation-controls {
    justify-content: center;
  }

  .search-actions {
    justify-content: center;
  }
}

/* 🌙 Dark Mode Support */
@media (prefers-color-scheme: dark) {
  .search-navigation-bar {
    background: linear-gradient(135deg,
        rgba(30, 41, 59, 0.95) 0%,
        rgba(51, 65, 85, 0.9) 100%);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .search-metrics {
    color: #94a3b8;
  }
}

/* 🎨 Enhanced Focus States */
.nav-button:focus,
.mode-toggle:focus,
.clear-button:focus {
  outline: 3px solid rgba(0, 122, 255, 0.5);
  outline-offset: 2px;
}

/* ⚡ Performance Optimizations */
.search-navigation-bar {
  contain: layout style;
  will-change: transform;
}

.nav-button,
.mode-toggle,
.clear-button {
  contain: layout style;
  will-change: transform;
}
</style>