<template>
  <Teleport to="body">
    <transition
      enter-active-class="transition-all duration-300 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-all duration-200 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="isOpen" class="image-viewer-overlay" @click="handleOverlayClick">
        <!-- Close Button -->
        <button 
          @click="close"
          class="close-button"
          title="Close (Esc)"
        >
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"/>
          </svg>
        </button>

        <!-- Navigation Arrows -->
        <button 
          v-if="images.length > 1 && currentIndex > 0"
          @click.stop="previous"
          class="nav-button nav-button-left"
          title="Previous (←)"
        >
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M15 18l-6-6 6-6"/>
          </svg>
        </button>

        <button 
          v-if="images.length > 1 && currentIndex < images.length - 1"
          @click.stop="next"
          class="nav-button nav-button-right"
          title="Next (→)"
        >
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 18l6-6-6-6"/>
          </svg>
        </button>

        <!-- Image Container -->
        <div class="image-container" @click.stop>
          <transition
            enter-active-class="transition-all duration-300 ease-out"
            enter-from-class="opacity-0 scale-95"
            enter-to-class="opacity-100 scale-100"
            leave-active-class="transition-all duration-200 ease-in"
            leave-from-class="opacity-100 scale-100"
            leave-to-class="opacity-0 scale-95"
            mode="out-in"
          >
            <div 
              v-if="!loading && !error"
              :key="currentImage.url"
              class="image-wrapper"
              @wheel="handleWheel"
            >
              <img 
                :src="currentImage.url"
                :alt="currentImage.filename || 'Image'"
                class="main-image"
                :style="imageStyle"
                @load="handleImageLoad"
                @error="handleImageError"
                @mousedown="startDrag"
                @touchstart="startDrag"
              />
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
              <path d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"/>
              <path d="M18 8L6 20"/>
            </svg>
            <p>Failed to load image</p>
            <button @click="retry" class="retry-button">Retry</button>
          </div>
        </div>

        <!-- Image Info Bar -->
        <div class="info-bar" @click.stop>
          <div class="info-content">
            <h3 class="image-title">{{ currentImage.filename || 'Untitled' }}</h3>
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
          
          <div class="info-actions">
            <!-- Zoom Controls -->
            <button @click="zoomOut" class="action-button" title="Zoom Out (-)">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8"/>
                <path d="M21 21l-4.35-4.35M8 11h6"/>
              </svg>
            </button>
            
            <button @click="resetZoom" class="action-button" title="Reset Zoom (0)">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8"/>
                <path d="M21 21l-4.35-4.35M11 8v6M8 11h6"/>
              </svg>
            </button>
            
            <button @click="zoomIn" class="action-button" title="Zoom In (+)">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="8"/>
                <path d="M21 21l-4.35-4.35M11 8v6M8 11h6"/>
              </svg>
            </button>
            
            <div class="separator-vertical"></div>
            
            <!-- Download Button -->
            <a 
              :href="currentImage.url"
              :download="currentImage.filename"
              class="action-button"
              title="Download"
              @click.stop
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/>
              </svg>
            </a>
            
            <!-- Open in New Tab -->
            <a 
              :href="currentImage.url"
              target="_blank"
              rel="noopener noreferrer"
              class="action-button"
              title="Open in new tab"
              @click.stop
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6M15 3h6v6M10 14L21 3"/>
              </svg>
            </a>
          </div>
        </div>
      </div>
    </transition>
  </Teleport>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue';

const props = defineProps({
  images: {
    type: Array,
    required: true,
    default: () => []
  },
  initialIndex: {
    type: Number,
    default: 0
  }
});

const emit = defineEmits(['close']);

// State
const isOpen = ref(false);
const currentIndex = ref(props.initialIndex);
const loading = ref(true);
const error = ref(false);
const zoomLevel = ref(1);
const position = ref({ x: 0, y: 0 });
const isDragging = ref(false);
const dragStart = ref({ x: 0, y: 0 });
const imageDimensions = ref(null);

// Touch handling
let touchStartDistance = 0;

// Computed
const currentImage = computed(() => props.images[currentIndex.value] || {});

const imageStyle = computed(() => {
  return {
    transform: `translate(${position.value.x}px, ${position.value.y}px) scale(${zoomLevel.value})`,
    cursor: isDragging.value ? 'grabbing' : (zoomLevel.value > 1 ? 'grab' : 'default'),
    transition: isDragging.value ? 'none' : 'transform 0.2s ease-out'
  };
});

// Methods
function open(index = 0) {
  currentIndex.value = index;
  isOpen.value = true;
  loading.value = true;
  error.value = false;
  resetView();
  document.body.style.overflow = 'hidden';
}

function close() {
  isOpen.value = false;
  document.body.style.overflow = '';
  emit('close');
}

function next() {
  if (currentIndex.value < props.images.length - 1) {
    currentIndex.value++;
    resetView();
    loading.value = true;
    error.value = false;
  }
}

function previous() {
  if (currentIndex.value > 0) {
    currentIndex.value--;
    resetView();
    loading.value = true;
    error.value = false;
  }
}

function zoomIn() {
  zoomLevel.value = Math.min(zoomLevel.value * 1.5, 5);
}

function zoomOut() {
  zoomLevel.value = Math.max(zoomLevel.value / 1.5, 0.5);
  constrainPosition();
}

function resetZoom() {
  resetView();
}

function resetView() {
  zoomLevel.value = 1;
  position.value = { x: 0, y: 0 };
}

function handleWheel(event) {
  event.preventDefault();
  const delta = event.deltaY > 0 ? 0.9 : 1.1;
  const newZoom = Math.max(0.5, Math.min(5, zoomLevel.value * delta));
  zoomLevel.value = newZoom;
  constrainPosition();
}

function startDrag(event) {
  if (zoomLevel.value <= 1) return;
  
  isDragging.value = true;
  
  if (event.type === 'mousedown') {
    dragStart.value = {
      x: event.clientX - position.value.x,
      y: event.clientY - position.value.y
    };
    
    document.addEventListener('mousemove', handleDrag);
    document.addEventListener('mouseup', endDrag);
  } else if (event.type === 'touchstart') {
    if (event.touches.length === 1) {
      dragStart.value = {
        x: event.touches[0].clientX - position.value.x,
        y: event.touches[0].clientY - position.value.y
      };
    } else if (event.touches.length === 2) {
      // Pinch zoom start
      touchStartDistance = getTouchDistance(event.touches);
    }
    
    document.addEventListener('touchmove', handleDrag, { passive: false });
    document.addEventListener('touchend', endDrag);
  }
}

function handleDrag(event) {
  if (!isDragging.value) return;
  
  if (event.type === 'mousemove') {
    position.value = {
      x: event.clientX - dragStart.value.x,
      y: event.clientY - dragStart.value.y
    };
  } else if (event.type === 'touchmove') {
    event.preventDefault();
    
    if (event.touches.length === 1) {
      position.value = {
        x: event.touches[0].clientX - dragStart.value.x,
        y: event.touches[0].clientY - dragStart.value.y
      };
    } else if (event.touches.length === 2 && touchStartDistance > 0) {
      // Pinch zoom
      const currentDistance = getTouchDistance(event.touches);
      const scale = currentDistance / touchStartDistance;
      zoomLevel.value = Math.max(0.5, Math.min(5, zoomLevel.value * scale));
      touchStartDistance = currentDistance;
    }
  }
  
  constrainPosition();
}

function endDrag() {
  isDragging.value = false;
  touchStartDistance = 0;
  document.removeEventListener('mousemove', handleDrag);
  document.removeEventListener('mouseup', endDrag);
  document.removeEventListener('touchmove', handleDrag);
  document.removeEventListener('touchend', endDrag);
}

function getTouchDistance(touches) {
  const dx = touches[0].clientX - touches[1].clientX;
  const dy = touches[0].clientY - touches[1].clientY;
  return Math.sqrt(dx * dx + dy * dy);
}

function constrainPosition() {
  // Add position constraints based on zoom level
  const maxX = Math.max(0, (zoomLevel.value - 1) * window.innerWidth / 2);
  const maxY = Math.max(0, (zoomLevel.value - 1) * window.innerHeight / 2);
  
  position.value = {
    x: Math.max(-maxX, Math.min(maxX, position.value.x)),
    y: Math.max(-maxY, Math.min(maxY, position.value.y))
  };
}

function handleImageLoad(event) {
  loading.value = false;
  error.value = false;
  
  // Set image dimensions
  const img = event.target;
  imageDimensions.value = `${img.naturalWidth} × ${img.naturalHeight}`;
}

function handleImageError() {
  loading.value = false;
  error.value = true;
  imageDimensions.value = null;
}

function retry() {
  loading.value = true;
  error.value = false;
  // Force reload by adding timestamp
  const img = new Image();
  img.src = currentImage.value.url + '?t=' + Date.now();
}

function handleOverlayClick(event) {
  if (event.target === event.currentTarget) {
    close();
  }
}

function handleKeydown(event) {
  if (!isOpen.value) return;
  
  switch (event.key) {
    case 'Escape':
      close();
      break;
    case 'ArrowLeft':
      previous();
      break;
    case 'ArrowRight':
      next();
      break;
    case '+':
    case '=':
      zoomIn();
      break;
    case '-':
    case '_':
      zoomOut();
      break;
    case '0':
      resetZoom();
      break;
  }
}

function formatFileSize(bytes) {
  if (!bytes) return '';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

// Lifecycle
onMounted(() => {
  document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown);
  document.body.style.overflow = '';
});

// Watch for prop changes
watch(() => props.initialIndex, (newIndex) => {
  currentIndex.value = newIndex;
});

// Expose methods for parent component
defineExpose({
  open,
  close
});
</script>

<style scoped>
.image-viewer-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.95);
  z-index: 9999;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.close-button {
  position: absolute;
  top: 20px;
  right: 20px;
  width: 48px;
  height: 48px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  cursor: pointer;
  transition: all 0.2s;
  z-index: 10;
}

.close-button:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: scale(1.1);
}

.nav-button {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 56px;
  height: 56px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  cursor: pointer;
  transition: all 0.2s;
  z-index: 10;
}

.nav-button:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: translateY(-50%) scale(1.1);
}

.nav-button-left {
  left: 20px;
}

.nav-button-right {
  right: 20px;
}

.image-container {
  flex: 1;
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  position: relative;
}

.image-wrapper {
  position: relative;
  max-width: 90vw;
  max-height: 80vh;
  display: flex;
  align-items: center;
  justify-content: center;
}

.main-image {
  max-width: 100%;
  max-height: 80vh;
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
  gap: 16px;
}

.loading-spinner {
  width: 48px;
  height: 48px;
  border: 3px solid rgba(255, 255, 255, 0.2);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.retry-button {
  padding: 8px 16px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 6px;
  color: white;
  cursor: pointer;
  transition: all 0.2s;
}

.retry-button:hover {
  background: rgba(255, 255, 255, 0.2);
}

.info-bar {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(10px);
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  padding: 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: white;
}

.info-content {
  flex: 1;
  min-width: 0;
}

.image-title {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.image-meta {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.7);
  display: flex;
  align-items: center;
  gap: 8px;
}

.separator {
  opacity: 0.4;
}

.separator-vertical {
  width: 1px;
  height: 24px;
  background: rgba(255, 255, 255, 0.2);
  margin: 0 8px;
}

.info-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.action-button {
  width: 40px;
  height: 40px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  cursor: pointer;
  transition: all 0.2s;
}

.action-button:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: scale(1.05);
}

/* Mobile responsiveness */
@media (max-width: 768px) {
  .nav-button {
    width: 44px;
    height: 44px;
  }
  
  .nav-button-left {
    left: 10px;
  }
  
  .nav-button-right {
    right: 10px;
  }
  
  .info-bar {
    padding: 16px;
  }
  
  .action-button {
    width: 36px;
    height: 36px;
  }
  
  .image-meta {
    font-size: 12px;
  }
}
</style>