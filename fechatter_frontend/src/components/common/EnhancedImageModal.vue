<template>
  <Teleport to="body">
    <transition enter-active-class="transition-all duration-300 ease-out" enter-from-class="opacity-0"
      enter-to-class="opacity-100" leave-active-class="transition-all duration-200 ease-in"
      leave-from-class="opacity-100" leave-to-class="opacity-0">
      <div v-if="isOpen" class="enhanced-image-modal" @click="handleOverlayClick">
        <!-- Header Toolbar -->
        <div class="modal-header">
          <div class="image-info">
            <h3 class="image-title">{{ currentImage.filename || 'Image' }}</h3>
            <div class="image-meta">
              <span v-if="currentImage.size">{{ formatFileSize(currentImage.size) }}</span>
              <span v-if="images.length > 1" class="separator">•</span>
              <span v-if="images.length > 1">{{ currentIndex + 1 }} / {{ images.length }}</span>
              <span v-if="imageDimensions" class="separator">•</span>
              <span v-if="imageDimensions">{{ imageDimensions }}</span>
              <span v-if="zoomLevel !== 1" class="separator">•</span>
              <span v-if="zoomLevel !== 1">{{ Math.round(zoomLevel * 100) }}%</span>
            </div>
          </div>

          <div class="toolbar-actions">
            <!-- Zoom Controls -->
            <button @click="zoomOut" class="toolbar-btn" title="Zoom Out (-)">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8"></circle>
                <path d="M21 21l-4.35-4.35"></path>
                <line x1="8" y1="11" x2="14" y2="11"></line>
              </svg>
            </button>

            <span class="zoom-indicator">{{ Math.round(zoomLevel * 100) }}%</span>

            <button @click="zoomIn" class="toolbar-btn" title="Zoom In (+)">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8"></circle>
                <path d="M21 21l-4.35-4.35"></path>
                <line x1="11" y1="8" x2="11" y2="14"></line>
                <line x1="8" y1="11" x2="14" y2="11"></line>
              </svg>
            </button>

            <button @click="resetZoom" class="toolbar-btn" title="Reset Zoom (0)">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                <circle cx="12" cy="12" r="3"></circle>
              </svg>
            </button>

            <!-- Rotation Controls -->
            <button @click="rotateLeft" class="toolbar-btn" title="Rotate Left (Shift+R)">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M2.5 2v6h6M2.66 15.57a10 10 0 1 0 .57-8.38"></path>
              </svg>
            </button>

            <button @click="rotateRight" class="toolbar-btn" title="Rotate Right (R)">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38"></path>
              </svg>
            </button>

            <!-- Flip Controls -->
            <button @click="flipHorizontal" class="toolbar-btn" title="Flip Horizontal (H)">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M8 3H5a2 2 0 0 0-2 2v14c0 1.1.9 2 2 2h3m8 0h3a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2h-3M12 3v18"></path>
              </svg>
            </button>

            <button @click="flipVertical" class="toolbar-btn" title="Flip Vertical (V)">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v3m18 8v3a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-3M3 12h18">
                </path>
              </svg>
            </button>

            <!-- Download -->
            <button @click="downloadImage" class="toolbar-btn" title="Download (D)">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                <polyline points="7,10 12,15 17,10"></polyline>
                <line x1="12" y1="15" x2="12" y2="3"></line>
              </svg>
            </button>

            <!-- PhotoSwipe -->
            <button v-if="enablePhotoSwipe" @click="openPhotoSwipe" class="toolbar-btn" title="PhotoSwipe (F)">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                <circle cx="8.5" cy="8.5" r="1.5"></circle>
                <polyline points="21,15 16,10 5,21"></polyline>
              </svg>
            </button>

            <!-- New Tab -->
            <button @click="openInNewTab" class="toolbar-btn" title="Open in New Tab">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                <polyline points="15,3 21,3 21,9"></polyline>
                <line x1="10" y1="14" x2="21" y2="3"></line>
              </svg>
            </button>

            <!-- Close -->
            <button @click="close" class="toolbar-btn close-btn" title="Close (Esc)">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 6L6 18M6 6l12 12"></path>
              </svg>
            </button>
          </div>
        </div>

        <!-- Navigation Arrows -->
        <button v-if="images.length > 1 && currentIndex > 0" @click.stop="previous" class="nav-button nav-button-left"
          title="Previous (←)">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M15 18l-6-6 6-6" />
          </svg>
        </button>

        <button v-if="images.length > 1 && currentIndex < images.length - 1" @click.stop="next"
          class="nav-button nav-button-right" title="Next (→)">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 18l6-6-6-6" />
          </svg>
        </button>

        <!-- Image Container -->
        <div class="image-container" @click.stop>
          <transition enter-active-class="transition-all duration-300 ease-out" enter-from-class="opacity-0 scale-95"
            enter-to-class="opacity-100 scale-100" leave-active-class="transition-all duration-200 ease-in"
            leave-from-class="opacity-100 scale-100" leave-to-class="opacity-0 scale-95" mode="out-in">
            <div v-if="!loading && !error" :key="currentImage.url" class="image-wrapper" @wheel="handleWheel">
              <img ref="imageElement" :src="authenticatedImageUrl" :alt="currentImage.filename || 'Image'"
                class="main-image" :style="imageStyle" @load="handleImageLoad" @error="handleImageError"
                @mousedown="startDrag" @touchstart="startDrag" draggable="false" />
            </div>
          </transition>

          <!-- Loading State -->
          <div v-if="loading" class="loading-state">
            <div class="loading-spinner"></div>
            <p>Loading image...</p>
          </div>

          <!-- Error State -->
          <div v-if="error" class="error-state">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path
                d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
              <path d="M18 8L6 20" />
            </svg>
            <p>Failed to load image</p>
            <button @click="retry" class="retry-button">Retry</button>
          </div>
        </div>

        <!-- PhotoSwipe Integration Point -->
        <div ref="photoSwipeElement" class="pswp" tabindex="-1" role="dialog" aria-hidden="true">
          <div class="pswp__bg"></div>
          <div class="pswp__scroll-wrap">
            <div class="pswp__container">
              <div class="pswp__item"></div>
              <div class="pswp__item"></div>
              <div class="pswp__item"></div>
            </div>
            <div class="pswp__ui pswp__ui--hidden">
              <div class="pswp__top-bar">
                <div class="pswp__counter"></div>
                <button class="pswp__button pswp__button--close" title="Close (Esc)"></button>
                <button class="pswp__button pswp__button--share" title="Share"></button>
                <button class="pswp__button pswp__button--fs" title="Toggle fullscreen"></button>
                <button class="pswp__button pswp__button--zoom" title="Zoom in/out"></button>
                <div class="pswp__preloader">
                  <div class="pswp__preloader__icn">
                    <div class="pswp__preloader__cut">
                      <div class="pswp__preloader__donut"></div>
                    </div>
                  </div>
                </div>
              </div>
              <div class="pswp__share-modal pswp__share-modal--hidden pswp__single-tap">
                <div class="pswp__share-tooltip"></div>
              </div>
              <button class="pswp__button pswp__button--arrow--left" title="Previous (arrow left)"></button>
              <button class="pswp__button pswp__button--arrow--right" title="Next (arrow right)"></button>
              <div class="pswp__caption">
                <div class="pswp__caption__center"></div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </transition>
  </Teleport>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import PhotoSwipe from 'photoswipe'
import imageCacheService from '@/services/ImageCacheService'
import 'photoswipe/dist/photoswipe.css'

const props = defineProps({
  images: {
    type: Array,
    required: true,
    default: () => []
  },
  initialIndex: {
    type: Number,
    default: 0
  },
  enablePhotoSwipe: {
    type: Boolean,
    default: true
  }
})

const emit = defineEmits(['close'])

// State
const isOpen = ref(false)
const currentIndex = ref(props.initialIndex)
const loading = ref(true)
const error = ref(false)
const zoomLevel = ref(1)
const rotation = ref(0)
const flipH = ref(1)
const flipV = ref(1)
const position = ref({ x: 0, y: 0 })
const isDragging = ref(false)
const dragStart = ref({ x: 0, y: 0 })
const imageDimensions = ref(null)
const authenticatedImageUrl = ref('')

// Refs
const imageElement = ref(null)
const photoSwipeElement = ref(null)

// Touch handling
let touchStartDistance = 0
let photoSwipeInstance = null

// Computed
const currentImage = computed(() => props.images[currentIndex.value] || {})

const imageStyle = computed(() => {
  const transform = [
    `translate(${position.value.x}px, ${position.value.y}px)`,
    `scale(${zoomLevel.value})`,
    `rotate(${rotation.value}deg)`,
    `scaleX(${flipH.value})`,
    `scaleY(${flipV.value})`
  ].join(' ')

  return {
    transform,
    cursor: isDragging.value ? 'grabbing' : (zoomLevel.value > 1 ? 'grab' : 'default'),
    transition: isDragging.value ? 'none' : 'transform 0.3s cubic-bezier(0.16, 1, 0.3, 1)'
  }
})

// 🔐 CRITICAL FIX: Smart URL handling for modal
const loadAuthenticatedImage = async (imageUrl) => {
  if (!imageUrl) return

  loading.value = true
  error.value = false
  authenticatedImageUrl.value = ''

  try {
    if (import.meta.env.DEV) {
      console.log('🖼️ [EnhancedImageModal] Loading image:', imageUrl)
    }

    // SMART: Check if URL is already a blob URL (pre-authenticated)
    if (imageUrl.startsWith('blob:')) {
      authenticatedImageUrl.value = imageUrl
      if (import.meta.env.DEV) {
        console.log('[EnhancedImageModal] Using pre-authenticated blob URL')
      }
      loading.value = false
      return
    }

    // OPTIMIZATION: Check if we have cached secure URL from message
    if (currentImage.value.secureUrl) {
      authenticatedImageUrl.value = currentImage.value.secureUrl
      if (import.meta.env.DEV) {
        console.log('[EnhancedImageModal] Using cached secure URL')
      }
      loading.value = false
      return
    }

    // FALLBACK: Only authenticate if it's an API URL
    if (imageUrl.startsWith('/api/files/') || imageUrl.startsWith('/files/')) {
      const cachedImageUrl = await imageCacheService.getCachedImageUrl(imageUrl, {
        skipAuthRefresh: false
      })

      if (cachedImageUrl) {
        authenticatedImageUrl.value = cachedImageUrl
        if (import.meta.env.DEV) {
          console.log('[EnhancedImageModal] Authenticated API URL successfully')
        }
      } else {
        throw new Error('Failed to authenticate API URL')
      }
    } else {
      // DIRECT: Use URL as-is for non-API URLs
      authenticatedImageUrl.value = imageUrl
      if (import.meta.env.DEV) {
        console.log('[EnhancedImageModal] Using direct URL')
      }
      loading.value = false
    }
  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('ERROR: [EnhancedImageModal] Failed to load image:', error)
    }
    error.value = true
  }
  // Note: loading.value will be set to false in handleImageLoad or handleImageError
}

// Watch current image changes and load authenticated version
watch(() => currentImage.value.url, (newUrl) => {
  if (newUrl && isOpen.value) {
    loadAuthenticatedImage(newUrl)
  }
}, { immediate: true })

// Methods
function open(index = 0) {
  currentIndex.value = index
  isOpen.value = true
  loading.value = true
  error.value = false
  resetView()
  document.body.style.overflow = 'hidden'

  // Load authenticated image when modal opens
  if (currentImage.value.url) {
    loadAuthenticatedImage(currentImage.value.url)
  }
}

function close() {
  isOpen.value = false
  document.body.style.overflow = ''
  authenticatedImageUrl.value = ''
  if (photoSwipeInstance) {
    photoSwipeInstance.close()
  }
  emit('close')
}

function next() {
  if (currentIndex.value < props.images.length - 1) {
    currentIndex.value++
    resetView()
    loading.value = true
    error.value = false
  }
}

function previous() {
  if (currentIndex.value > 0) {
    currentIndex.value--
    resetView()
    loading.value = true
    error.value = false
  }
}

function zoomIn() {
  zoomLevel.value = Math.min(zoomLevel.value * 1.5, 5)
}

function zoomOut() {
  zoomLevel.value = Math.max(zoomLevel.value / 1.5, 0.1)
}

function resetZoom() {
  zoomLevel.value = 1
  position.value = { x: 0, y: 0 }
}

function rotateLeft() {
  rotation.value = (rotation.value - 90) % 360
}

function rotateRight() {
  rotation.value = (rotation.value + 90) % 360
}

function flipHorizontal() {
  flipH.value *= -1
}

function flipVertical() {
  flipV.value *= -1
}

function resetView() {
  zoomLevel.value = 1
  rotation.value = 0
  flipH.value = 1
  flipV.value = 1
  position.value = { x: 0, y: 0 }
}

function handleWheel(event) {
  event.preventDefault()
  const delta = event.deltaY < 0 ? 1.2 : 0.8
  const newZoom = Math.min(Math.max(zoomLevel.value * delta, 0.1), 5)

  // Zoom towards cursor position
  const rect = event.currentTarget.getBoundingClientRect()
  const centerX = rect.width / 2
  const centerY = rect.height / 2
  const offsetX = event.clientX - rect.left - centerX
  const offsetY = event.clientY - rect.top - centerY

  const zoomRatio = newZoom / zoomLevel.value
  position.value = {
    x: position.value.x - offsetX * (zoomRatio - 1),
    y: position.value.y - offsetY * (zoomRatio - 1)
  }

  zoomLevel.value = newZoom
}

function startDrag(event) {
  // ENHANCED: Allow dragging at any zoom level (removed zoom restriction)
  event.preventDefault()

  isDragging.value = true
  const clientX = event.clientX || event.touches?.[0]?.clientX
  const clientY = event.clientY || event.touches?.[0]?.clientY

  dragStart.value = {
    x: clientX - position.value.x,
    y: clientY - position.value.y
  }

  // Enhanced drag experience
  document.body.style.userSelect = 'none'
  document.body.style.cursor = 'grabbing'

  document.addEventListener('mousemove', handleDrag)
  document.addEventListener('mouseup', stopDrag)
  document.addEventListener('touchmove', handleDrag, { passive: false })
  document.addEventListener('touchend', stopDrag)
}

function handleDrag(event) {
  if (!isDragging.value) return

  event.preventDefault()

  const clientX = event.clientX || event.touches?.[0]?.clientX
  const clientY = event.clientY || event.touches?.[0]?.clientY

  const newX = clientX - dragStart.value.x
  const newY = clientY - dragStart.value.y

  // ENHANCED: Smart boundary limiting for better UX
  const container = document.querySelector('.image-container')
  const image = imageElement.value

  if (container && image && zoomLevel.value > 1) {
    const containerRect = container.getBoundingClientRect()
    const imageRect = image.getBoundingClientRect()

    // Calculate boundaries based on zoom level  
    const scaledWidth = imageRect.width * zoomLevel.value
    const scaledHeight = imageRect.height * zoomLevel.value

    const maxX = Math.max(0, (scaledWidth - containerRect.width) / 2)
    const maxY = Math.max(0, (scaledHeight - containerRect.height) / 2)

    // Apply smart boundaries with some overflow allowance
    const boundaryMargin = 100
    const limitedX = Math.max(-maxX - boundaryMargin, Math.min(maxX + boundaryMargin, newX))
    const limitedY = Math.max(-maxY - boundaryMargin, Math.min(maxY + boundaryMargin, newY))

    position.value = { x: limitedX, y: limitedY }
  } else {
    // Free dragging for normal zoom level
    position.value = { x: newX, y: newY }
  }
}

function stopDrag() {
  isDragging.value = false

  // Restore UI state
  document.body.style.userSelect = ''
  document.body.style.cursor = ''

  document.removeEventListener('mousemove', handleDrag)
  document.removeEventListener('mouseup', stopDrag)
  document.removeEventListener('touchmove', handleDrag)
  document.removeEventListener('touchend', stopDrag)

  // ENHANCED: Auto-snap back if image is dragged too far
  snapBackIfNeeded()
}

// NEW: Smart snap back functionality
function snapBackIfNeeded() {
  const container = document.querySelector('.image-container')
  const image = imageElement.value

  if (!container || !image) return

  const containerRect = container.getBoundingClientRect()
  const imageRect = image.getBoundingClientRect()

  // Check if image is completely outside visible area
  const isCompletelyOutside =
    imageRect.right < containerRect.left + 50 ||
    imageRect.left > containerRect.right - 50 ||
    imageRect.bottom < containerRect.top + 50 ||
    imageRect.top > containerRect.bottom - 50

  if (isCompletelyOutside) {
    // Smoothly snap back to center with animation
    position.value = { x: 0, y: 0 }

    if (import.meta.env.DEV) {
      console.log('[EnhancedImageModal] Snapped image back to center')
    }
  }
}

function handleImageLoad(event) {
  loading.value = false
  error.value = false
  const img = event.target
  imageDimensions.value = `${img.naturalWidth} × ${img.naturalHeight}`

  if (import.meta.env.DEV) {
    console.log('[EnhancedImageModal] Image loaded successfully')
  }
}

function handleImageError() {
  loading.value = false
  error.value = true
  imageDimensions.value = null

  if (import.meta.env.DEV) {
    console.error('ERROR: [EnhancedImageModal] Image load error')
  }
}

function retry() {
  if (currentImage.value.url) {
    loadAuthenticatedImage(currentImage.value.url)
  }
}

async function downloadImage() {
  if (!currentImage.value.url) return

  try {
    if (import.meta.env.DEV) {
      console.log('📥 [EnhancedImageModal] Starting download for:', currentImage.value.filename)
    }

    // 🔐 Use authenticated download
    if (currentImage.value.url.includes('/api/files/') || currentImage.value.url.includes('/files/')) {
      // Get authenticated blob URL through ImageCacheService  
      const authenticatedUrl = await imageCacheService.getCachedImageUrl(currentImage.value.url, {
        skipAuthRefresh: false
      })

      if (authenticatedUrl) {
        const link = document.createElement('a')
        link.href = authenticatedUrl
        link.download = currentImage.value.filename || 'image'
        document.body.appendChild(link)
        link.click()
        document.body.removeChild(link)

        if (import.meta.env.DEV) {
          console.log('[EnhancedImageModal] Authenticated download completed')
        }
      } else {
        throw new Error('Failed to get authenticated download URL')
      }
    } else {
      // Direct download for non-API URLs
      const link = document.createElement('a')
      link.href = currentImage.value.url
      link.download = currentImage.value.filename || 'image'
      link.target = '_blank'
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
    }
  } catch (error) {
    console.error('ERROR: [EnhancedImageModal] Download failed:', error)
    alert('Failed to download image. Please try again.')
  }
}

function openInNewTab() {
  if (authenticatedImageUrl.value) {
    window.open(authenticatedImageUrl.value, '_blank')
  } else if (currentImage.value.url) {
    window.open(currentImage.value.url, '_blank')
  }
}

function openPhotoSwipe() {
  if (!props.enablePhotoSwipe || !photoSwipeElement.value) return

  const items = props.images.map(image => ({
    src: image.url,
    w: image.width || 1200,
    h: image.height || 800,
    title: image.filename || 'Image'
  }))

  const options = {
    index: currentIndex.value,
    bgOpacity: 0.9,
    showHideOpacity: true,
    getThumbBoundsFn: () => ({ x: 0, y: 0, w: 0 })
  }

  photoSwipeInstance = new PhotoSwipe(photoSwipeElement.value, PhotoSwipe.UI_Default, items, options)
  photoSwipeInstance.init()

  photoSwipeInstance.listen('close', () => {
    close()
  })
}

function handleOverlayClick(event) {
  if (event.target === event.currentTarget) {
    close()
  }
}

function handleKeydown(event) {
  if (!isOpen.value) return

  switch (event.key) {
    case 'Escape':
      close()
      break
    case 'ArrowLeft':
      previous()
      break
    case 'ArrowRight':
      next()
      break
    case '+':
    case '=':
      zoomIn()
      break
    case '-':
    case '_':
      zoomOut()
      break
    case '0':
      resetZoom()
      break
    case 'r':
      rotateRight()
      break
    case 'R':
      rotateLeft()
      break
    case 'h':
      flipHorizontal()
      break
    case 'v':
      flipVertical()
      break
    case 'd':
      downloadImage()
      break
    case 'f':
      if (props.enablePhotoSwipe) {
        openPhotoSwipe()
      }
      break
  }
}

function formatFileSize(bytes) {
  if (!bytes) return ''
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

// Lifecycle
onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  document.body.style.overflow = ''
  if (photoSwipeInstance) {
    photoSwipeInstance.destroy()
  }
})

// Watch for prop changes
watch(() => props.initialIndex, (newIndex) => {
  currentIndex.value = newIndex
})

// Expose methods for parent component
defineExpose({
  open,
  close,
  openPhotoSwipe
})
</script>

<style scoped>
.enhanced-image-modal {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.95);
  z-index: 9999;
  display: flex;
  flex-direction: column;
}

.modal-header {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  z-index: 10;
  background: linear-gradient(to bottom, rgba(0, 0, 0, 0.8), transparent);
  padding: 20px;
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.image-info {
  flex: 1;
}

.image-title {
  color: white;
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 4px 0;
  word-break: break-all;
}

.image-meta {
  color: rgba(255, 255, 255, 0.7);
  font-size: 14px;
}

.separator {
  margin: 0 8px;
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.toolbar-btn {
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(4px);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: white;
  border-radius: 8px;
  padding: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.toolbar-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: scale(1.05);
}

.close-btn {
  background: rgba(239, 68, 68, 0.8) !important;
}

.close-btn:hover {
  background: rgba(239, 68, 68, 1) !important;
}

.zoom-indicator {
  color: white;
  font-size: 12px;
  min-width: 40px;
  text-align: center;
}

.nav-button {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(4px);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: white;
  border-radius: 50%;
  width: 60px;
  height: 60px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 5;
}

.nav-button:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: translateY(-50%) scale(1.1);
}

.nav-button-left {
  left: 24px;
}

.nav-button-right {
  right: 24px;
}

.image-container {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  padding: 100px 20px 20px;
}

.image-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}

.main-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  user-select: none;
  -webkit-user-drag: none;
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
  width: 64px;
  height: 64px;
  margin-bottom: 16px;
  opacity: 0.7;
}

.retry-button {
  margin-top: 12px;
  padding: 8px 16px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: white;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.retry-button:hover {
  background: rgba(255, 255, 255, 0.2);
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

/* Mobile responsive */
@media (max-width: 768px) {
  .modal-header {
    padding: 16px;
  }

  .toolbar-actions {
    flex-direction: column;
    align-items: flex-end;
  }

  .nav-button {
    width: 50px;
    height: 50px;
  }

  .nav-button-left {
    left: 16px;
  }

  .nav-button-right {
    right: 16px;
  }

  .image-container {
    padding: 80px 10px 10px;
  }
}

/* PhotoSwipe overrides */
:deep(.pswp) {
  z-index: 10000;
}
</style>