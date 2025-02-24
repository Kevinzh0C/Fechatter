<template>
  <div class="image-preview-overlay" @click="handleOverlayClick">
    <div class="image-preview-container">
      <!-- Close Button -->
      <button class="close-btn" @click="$emit('close')" title="Close (Esc)">
        <Icon name="x" />
      </button>

      <!-- Image -->
      <img
        :src="src"
        :alt="alt"
        class="preview-image"
        @load="handleImageLoad"
        @error="handleImageError"
      />

      <!-- Loading State -->
      <div v-if="loading" class="loading-state">
        <div class="loading-spinner"></div>
        <p>Loading image...</p>
      </div>

      <!-- Error State -->
      <div v-if="error" class="error-state">
        <Icon name="image-off" />
        <p>Failed to load image</p>
        <button @click="retryLoad" class="retry-btn">Retry</button>
      </div>

      <!-- Image Info -->
      <div v-if="!loading && !error" class="image-info">
        <span class="image-name">{{ alt }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import Icon from '@/components/ui/Icon.vue'

// Props
const props = defineProps({
  src: {
    type: String,
    required: true
  },
  alt: {
    type: String,
    default: 'Image'
  }
})

// Emits
const emit = defineEmits(['close'])

// Reactive data
const loading = ref(true)
const error = ref(false)

// Methods
const handleOverlayClick = (event) => {
  // Close modal if clicking on overlay (not the image)
  if (event.target === event.currentTarget) {
    emit('close')
  }
}

const handleImageLoad = () => {
  loading.value = false
  error.value = false
}

const handleImageError = () => {
  loading.value = false
  error.value = true
}

const retryLoad = () => {
  loading.value = true
  error.value = false
  // Force image reload by adding timestamp
  const img = document.querySelector('.preview-image')
  if (img) {
    const src = props.src.includes('?') 
      ? `${props.src}&t=${Date.now()}` 
      : `${props.src}?t=${Date.now()}`
    img.src = src
  }
}

const handleKeydown = (event) => {
  if (event.key === 'Escape') {
    emit('close')
  }
}

// Lifecycle
onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
  document.body.style.overflow = 'hidden' // Prevent background scrolling
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  document.body.style.overflow = '' // Restore scrolling
})
</script>

<style scoped>
.image-preview-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  padding: 20px;
}

.image-preview-container {
  position: relative;
  max-width: 90vw;
  max-height: 90vh;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn {
  position: absolute;
  top: -40px;
  right: 0;
  background: rgba(255, 255, 255, 0.1);
  border: none;
  color: white;
  padding: 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s ease;
  z-index: 10;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.preview-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.loading-state,
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: white;
  text-align: center;
  min-width: 200px;
  min-height: 200px;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(255, 255, 255, 0.3);
  border-top: 3px solid white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 16px;
}

.error-state svg {
  width: 48px;
  height: 48px;
  margin-bottom: 16px;
  opacity: 0.7;
}

.retry-btn {
  margin-top: 12px;
  padding: 8px 16px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: white;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.retry-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.image-info {
  position: absolute;
  bottom: -40px;
  left: 0;
  right: 0;
  text-align: center;
  color: white;
  opacity: 0.8;
}

.image-name {
  font-size: 14px;
  word-break: break-all;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* Mobile responsive */
@media (max-width: 768px) {
  .image-preview-overlay {
    padding: 10px;
  }
  
  .close-btn {
    top: -35px;
    padding: 6px;
  }
  
  .image-info {
    bottom: -35px;
  }
}
</style> 