<template>
  <div class="secure-image-container" :class="containerClass">
    <!-- Loading State -->
    <div v-if="isLoading" class="secure-image-loading">
      <div class="loading-spinner"></div>
      <span class="loading-text">Loading image...</span>
    </div>

    <!-- Error State -->
    <div v-else-if="hasError" class="secure-image-error">
      <div class="error-icon">📷</div>
      <span class="error-text">Failed to load image</span>
    </div>

    <!-- Image Display -->
    <img v-else-if="imageUrl" :src="imageUrl" :alt="alt" :class="imageClass" @load="handleLoad" @error="handleError"
      @click="handleClick" loading="lazy" />
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'
import imageCacheService from '@/services/ImageCacheService'

// Props
const props = defineProps({
  src: {
    type: String,
    required: true
  },
  alt: {
    type: String,
    default: ''
  },
  class: {
    type: String,
    default: ''
  }
})

// Emits
const emit = defineEmits(['load', 'error', 'click'])

// Reactive data
const imageUrl = ref('')
const isLoading = ref(false)
const hasError = ref(false)
const objectUrl = ref('')

// Computed classes
const containerClass = ref(props.class)
const imageClass = ref(props.class)

// 🖼️ Enhanced fetchImage with intelligent caching
const fetchImage = async (url) => {
  if (!url) return

  isLoading.value = true
  hasError.value = false

  try {
    if (import.meta.env.DEV) {
      console.log('🖼️ [CachedSecureImage] Fetching image with cache:', url)
    }

    // 使用缓存服务获取图片
    const cachedImageUrl = await imageCacheService.getCachedImageUrl(url, {
      skipAuthRefresh: false
    })

    if (cachedImageUrl) {
      // 如果是blob URL，保存引用用于清理
      if (cachedImageUrl.startsWith('blob:')) {
        objectUrl.value = cachedImageUrl
      }

      imageUrl.value = cachedImageUrl

      if (import.meta.env.DEV) {
        console.log('[CachedSecureImage] Image loaded from cache/remote')
      }
    } else {
      throw new Error('Failed to get image URL from cache service')
    }
  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('ERROR: [CachedSecureImage] Failed to fetch image:', error)
    }

    hasError.value = true
    emit('error', error)
  } finally {
    isLoading.value = false
  }
}

const cleanup = () => {
  // 🧹 缓存服务会管理blob URLs，我们只需要清理本地引用
  if (objectUrl.value) {
    objectUrl.value = ''
    if (import.meta.env.DEV) {
      console.log('🧹 [CachedSecureImage] Local object URL reference cleared')
    }
  }
}

const handleLoad = (event) => {
  if (import.meta.env.DEV) {
    console.log('[CachedSecureImage] Image loaded successfully')
  }
  emit('load', event)
}

const handleError = (event) => {
  if (import.meta.env.DEV) {
    console.error('ERROR: [CachedSecureImage] Image load error:', event)
  }
  hasError.value = true
  emit('error', event)
}

const handleClick = (event) => {
  emit('click', event)
}

// Watch for src changes
watch(() => props.src, (newSrc) => {
  cleanup() // Clean up previous object URL reference
  imageUrl.value = ''
  hasError.value = false

  if (newSrc) {
    fetchImage(newSrc)
  }
}, { immediate: true })

// Lifecycle
onMounted(() => {
  if (props.src) {
    fetchImage(props.src)
  }
})

onUnmounted(() => {
  cleanup()
})
</script>

<style scoped>
.secure-image-container {
  position: relative;
  display: inline-block;
}

.secure-image-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: #f8f9fa;
  border-radius: 8px;
  color: #6c757d;
  min-height: 120px;
}

.loading-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid #e9ecef;
  border-top: 2px solid #6c757d;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 8px;
}

.loading-text {
  font-size: 14px;
  color: #6c757d;
}

.secure-image-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: #f8d7da;
  border-radius: 8px;
  color: #721c24;
  min-height: 120px;
}

.error-icon {
  font-size: 24px;
  margin-bottom: 8px;
}

.error-text {
  font-size: 14px;
  color: #721c24;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

/* Ensure image inherits container classes */
img {
  max-width: 100%;
  height: auto;
}
</style>