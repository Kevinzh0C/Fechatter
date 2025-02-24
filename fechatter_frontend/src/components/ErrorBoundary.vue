<!--
  ErrorBoundary.vue
  Basic Error Boundary Component for Vue Applications
-->
<template>
  <div class="error-boundary">
    <slot v-if="!hasError"></slot>
    <div v-else class="error-fallback">
      <h2>🚨 Something went wrong</h2>
      <p>{{ error?.message || 'An unexpected error occurred' }}</p>
      <button @click="retry" class="retry-btn">
        Try Again
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, onErrorCaptured } from 'vue'

// ================================
// 🎯 Props & Emits
// ================================

const props = defineProps({
  fallback: {
    type: String,
    default: 'An error occurred'
  }
})

const emit = defineEmits(['error', 'retry'])

// ================================
// 🎯 Reactive State
// ================================

const hasError = ref(false)
const error = ref(null)

// ================================
// 🎯 Error Handling
// ================================

onErrorCaptured((err, instance, info) => {
  hasError.value = true
  error.value = err
  
  console.error('ErrorBoundary caught error:', {
    error: err,
    component: instance?.$options.name || 'Unknown',
    info
  })
  
  emit('error', { error: err, instance, info })
  
  // Return false to prevent the error from propagating further
  return false
})

// ================================
// 🎯 Methods
// ================================

const retry = () => {
  hasError.value = false
  error.value = null
  emit('retry')
}

// Expose retry method for parent components
defineExpose({
  retry,
  hasError: () => hasError.value,
  getError: () => error.value
})
</script>

<style scoped>
.error-boundary {
  width: 100%;
  height: 100%;
}

.error-fallback {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  text-align: center;
  background: #f8f9fa;
  border-radius: 8px;
  border: 1px solid #e9ecef;
  min-height: 200px;
}

.error-fallback h2 {
  color: #dc3545;
  margin-bottom: 1rem;
  font-size: 1.5rem;
}

.error-fallback p {
  color: #6c757d;
  margin-bottom: 1.5rem;
  max-width: 400px;
}

.retry-btn {
  background: #007bff;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
  transition: background 0.2s ease;
}

.retry-btn:hover {
  background: #0056b3;
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .error-fallback {
    background: #2d3748;
    border-color: #4a5568;
  }
  
  .error-fallback h2 {
    color: #fc8181;
  }
  
  .error-fallback p {
    color: #a0aec0;
  }
}
</style> 