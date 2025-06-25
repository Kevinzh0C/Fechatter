<template>
  <div class="enhanced-image-thumbnail" :class="thumbnailClasses" :style="containerStyle" @click="openFullView">

    <!-- Loading State -->
    <div v-if="loading" class="thumbnail-loading">
      <div class="loading-spinner"></div>
      <span class="loading-text">Loading...</span>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="thumbnail-error">
      <svg class="error-icon" width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
        <path
          d="M21,5c0-1.11-0.89-2-2-2H5C3.89,3,3,3.89,3,5v14c0,1.11,0.89,2,2,2h14c1.11,0,2-0.89,2-2V5z M12,17 l-3.5-4.5l-2.5,3.01L8.5,13l3.5,4.5l2.5-3.01L19,19H5L12,17z" />
      </svg>
      <span class="error-text">Failed to load image</span>
      <button @click.stop="retry" class="retry-btn">Retry</button>
    </div>

    <!-- Image Display -->
    <div v-else class="thumbnail-wrapper" :style="wrapperStyle">
      <img :src="thumbnailSrc" :alt="alt" class="thumbnail-image" :style="imageStyle" @load="onImageLoad"
        @error="onImageError" loading="lazy" decoding="async" />

      <!-- Image Overlay -->
      <div class="thumbnail-overlay">
        <!-- File Info -->
        <div class="overlay-info">
          <span class="file-name">{{ displayFileName }}</span>
          <span class="file-size">{{ formattedSize }}</span>
        </div>

        <!-- Action Buttons -->
        <div class="overlay-actions">
          <button @click.stop="download" class="action-btn" title="Download">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19,9h-4V3H9v6H5l7,7L19,9z M5,18v2h14v-2H5z" />
            </svg>
          </button>
          <button @click.stop="openFullView" class="action-btn" title="View full size">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M7,14H5v5h5v-2H7V14z M5,10h2V7h3V5H5V10z M17,17h-3v2h5v-5h-2V17z M14,5v2h3v3h2V5H14z" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Image Format Badge -->
      <div v-if="formatBadge" class="format-badge">{{ formatBadge }}</div>

      <!-- Loading Progress -->
      <div v-if="loadingProgress > 0 && loadingProgress < 100" class="loading-progress">
        <div class="progress-bar" :style="{ width: loadingProgress + '%' }"></div>
      </div>
    </div>

    <!-- Full View Modal -->
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="showFullView" class="full-view-modal" @click="closeFullView">
          <div class="modal-content" @click.stop>
            <img :src="fullImageSrc" :alt="alt" class="full-image" :style="fullImageStyle" />
            <button @click="closeFullView" class="close-btn">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path
                  d="M19,6.41L17.59,5 12,10.59 6.41,5 5,6.41 10.59,12 5,17.59 6.41,19 12,13.41 17.59,19 19,17.59 13.41,12Z" />
              </svg>
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { getStandardFileUrl } from '@/utils/fileUrlHandler';
import { useAuthStore } from '@/stores/auth';

const props = defineProps({
  file: {
    type: Object,
    required: true
  },
  // Display mode: 'inline', 'grid', 'list'
  mode: {
    type: String,
    default: 'inline'
  },
  // Maximum dimensions
  maxWidth: {
    type: Number,
    default: 400
  },
  maxHeight: {
    type: Number,
    default: 300
  },
  // Whether to show loading progress
  showProgress: {
    type: Boolean,
    default: true
  },
  // Quality settings for thumbnail
  thumbnailQuality: {
    type: Number,
    default: 0.8
  }
});

const emit = defineEmits(['open', 'download', 'load', 'error']);

// Get auth store for workspace ID
const authStore = useAuthStore();

// State
const loading = ref(true);
const error = ref(false);
const showFullView = ref(false);
const loadingProgress = ref(0);
const naturalWidth = ref(0);
const naturalHeight = ref(0);

// 🔧 CRITICAL FIX: Use unified file URL handler for correct test.rest format
const getCorrectFileUrl = (file) => {
  return getStandardFileUrl(file, {
    workspaceId: authStore.user?.workspace_id || 2
  });
};

// Computed properties
const thumbnailSrc = computed(() => {
  // If backend supports thumbnail generation
  if (props.file.thumbnail_url) {
    return props.file.thumbnail_url;
  }

  // 🔧 CRITICAL FIX: Use unified URL handler instead of direct file properties
  const correctUrl = getCorrectFileUrl(props.file);
  if (!correctUrl) {
    console.error('❌ [EnhancedImageThumbnail] No valid URL for file:', props.file);
    return '';
  }

  if (import.meta.env.DEV) {
    console.log('🖼️ [EnhancedImageThumbnail] Generated URL:', correctUrl);
  }

  return correctUrl;
});

const fullImageSrc = computed(() => {
  // 🔧 CRITICAL FIX: Use unified URL handler for full image as well
  return getCorrectFileUrl(props.file) || '';
});

const displayFileName = computed(() => {
  const name = props.file.file_name || props.file.filename || 'image';
  if (name.length > 30) {
    const ext = name.split('.').pop();
    return `${name.substring(0, 20)}...${ext}`;
  }
  return name;
});

const formattedSize = computed(() => {
  const bytes = props.file.file_size || props.file.size || 0;
  if (bytes === 0) return '';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
});

const formatBadge = computed(() => {
  const mimeType = props.file.mime_type || props.file.type || '';
  const ext = (props.file.file_name || props.file.filename || '').split('.').pop()?.toUpperCase();

  if (mimeType.includes('gif')) return 'GIF';
  if (mimeType.includes('webp')) return 'WEBP';
  if (mimeType.includes('svg')) return 'SVG';
  if (ext === 'HEIC' || ext === 'HEIF') return ext;

  return '';
});

const alt = computed(() => {
  return props.file.file_name || props.file.filename || 'Image';
});

const thumbnailClasses = computed(() => ({
  [`mode-${props.mode}`]: true,
  'is-loading': loading.value,
  'has-error': error.value,
  'is-portrait': naturalHeight.value > naturalWidth.value,
  'is-landscape': naturalWidth.value > naturalHeight.value,
  'is-square': naturalWidth.value === naturalHeight.value
}));

// Dynamic sizing calculations
const containerStyle = computed(() => {
  if (props.mode === 'grid') {
    return {
      width: '100%',
      paddingBottom: '100%', // 1:1 aspect ratio
      position: 'relative'
    };
  }

  if (!naturalWidth.value || !naturalHeight.value) {
    return {
      maxWidth: `${props.maxWidth}px`,
      maxHeight: `${props.maxHeight}px`
    };
  }

  // Calculate optimal dimensions while maintaining aspect ratio
  const aspectRatio = naturalWidth.value / naturalHeight.value;
  let width = naturalWidth.value;
  let height = naturalHeight.value;

  // Scale down if necessary
  if (width > props.maxWidth) {
    width = props.maxWidth;
    height = width / aspectRatio;
  }

  if (height > props.maxHeight) {
    height = props.maxHeight;
    width = height * aspectRatio;
  }

  return {
    width: `${Math.round(width)}px`,
    height: `${Math.round(height)}px`
  };
});

const wrapperStyle = computed(() => {
  if (props.mode === 'grid') {
    return {
      position: 'absolute',
      top: 0,
      left: 0,
      width: '100%',
      height: '100%'
    };
  }
  return {};
});

const imageStyle = computed(() => {
  if (props.mode === 'grid') {
    return {
      width: '100%',
      height: '100%',
      objectFit: 'cover'
    };
  }
  return {
    width: '100%',
    height: '100%',
    objectFit: 'contain'
  };
});

const fullImageStyle = computed(() => {
  if (!naturalWidth.value || !naturalHeight.value) return {};

  const viewportWidth = window.innerWidth * 0.9;
  const viewportHeight = window.innerHeight * 0.9;
  const aspectRatio = naturalWidth.value / naturalHeight.value;

  let width = naturalWidth.value;
  let height = naturalHeight.value;

  if (width > viewportWidth) {
    width = viewportWidth;
    height = width / aspectRatio;
  }

  if (height > viewportHeight) {
    height = viewportHeight;
    width = height * aspectRatio;
  }

  return {
    maxWidth: `${width}px`,
    maxHeight: `${height}px`
  };
});

// Methods
const onImageLoad = (event) => {
  loading.value = false;
  error.value = false;
  naturalWidth.value = event.target.naturalWidth;
  naturalHeight.value = event.target.naturalHeight;
  loadingProgress.value = 100;
  emit('load', event);
};

const onImageError = (event) => {
  loading.value = false;
  error.value = true;
  emit('error', event);
};

const retry = () => {
  loading.value = true;
  error.value = false;
  // Force reload by adding timestamp
  const img = new Image();
  img.src = thumbnailSrc.value + (thumbnailSrc.value.includes('?') ? '&' : '?') + 't=' + Date.now();
};

const openFullView = () => {
  showFullView.value = true;
  emit('open', props.file);
};

const closeFullView = () => {
  showFullView.value = false;
};

const download = async () => {
  const fileName = props.file.file_name || props.file.filename || 'image';

  try {
    const fileUrl = getCorrectFileUrl(props.file);
    if (!fileUrl) {
      console.error('❌ No URL available for download:', fileName);
      return;
    }

    // 🔐 For API URLs, use authenticated download
    if (fileUrl.startsWith('/api/')) {
      // Import api client dynamically to avoid circular dependencies
      const { default: api } = await import('@/services/api');

      // Remove /api/ prefix since api client adds it automatically
      const apiPath = fileUrl.substring(5);

      const response = await api.get(apiPath, {
        responseType: 'blob',
        skipAuthRefresh: false
      });

      if (response.data) {
        const blob = response.data;
        const url = URL.createObjectURL(blob);

        const link = document.createElement('a');
        link.href = url;
        link.download = fileName;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);

        // Clean up blob URL
        setTimeout(() => URL.revokeObjectURL(url), 1000);

        console.log('✅ [EnhancedImageThumbnail] Downloaded:', fileName);
      }
    } else {
      // 🔗 For direct URLs, use standard download
      const link = document.createElement('a');
      link.href = fileUrl;
      link.download = fileName;
      link.target = '_blank';
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    }

    emit('download', props.file);
  } catch (error) {
    console.error('❌ [EnhancedImageThumbnail] Download failed:', error);
  }
};

// Simulate loading progress for better UX
let progressInterval;
onMounted(() => {
  if (props.showProgress && loading.value) {
    progressInterval = setInterval(() => {
      if (loadingProgress.value < 90) {
        loadingProgress.value += Math.random() * 30;
      }
    }, 300);
  }
});

onUnmounted(() => {
  if (progressInterval) {
    clearInterval(progressInterval);
  }
});
</script>

<style scoped>
/* Base styles */
.enhanced-image-thumbnail {
  position: relative;
  display: inline-block;
  border-radius: 8px;
  overflow: hidden;
  background: var(--bg-secondary, #f3f4f6);
  transition: all 0.2s ease;
  cursor: pointer;
}

.enhanced-image-thumbnail:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

/* Mode specific styles */
.mode-inline {
  max-width: 100%;
}

.mode-grid {
  width: 100%;
  aspect-ratio: 1;
}

.mode-list {
  height: 60px;
  width: 60px;
}

/* Loading state */
.thumbnail-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 150px;
  color: var(--text-muted, #6b7280);
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid rgba(0, 0, 0, 0.1);
  border-top-color: var(--primary, #3b82f6);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.loading-text {
  margin-top: 8px;
  font-size: 12px;
}

/* Error state */
.thumbnail-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 150px;
  padding: 20px;
  text-align: center;
  color: var(--text-muted, #6b7280);
}

.error-icon {
  color: var(--danger, #ef4444);
  margin-bottom: 8px;
}

.error-text {
  font-size: 14px;
  margin-bottom: 8px;
}

.retry-btn {
  padding: 4px 12px;
  background: var(--primary, #3b82f6);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.2s;
}

.retry-btn:hover {
  background: var(--primary-hover, #2563eb);
}

/* Image wrapper */
.thumbnail-wrapper {
  position: relative;
  width: 100%;
  height: 100%;
}

.thumbnail-image {
  display: block;
  border-radius: 8px;
  transition: transform 0.2s ease;
}

/* Overlay */
.thumbnail-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(to bottom,
      rgba(0, 0, 0, 0) 0%,
      rgba(0, 0, 0, 0) 50%,
      rgba(0, 0, 0, 0.7) 100%);
  opacity: 0;
  transition: opacity 0.2s ease;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  padding: 12px;
}

.enhanced-image-thumbnail:hover .thumbnail-overlay {
  opacity: 1;
}

.overlay-info {
  display: flex;
  flex-direction: column;
  color: white;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
  align-self: flex-start;
  margin-top: auto;
}

.file-name {
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 2px;
}

.file-size {
  font-size: 12px;
  opacity: 0.9;
}

.overlay-actions {
  display: flex;
  gap: 8px;
  align-self: flex-end;
  margin-top: auto;
}

.action-btn {
  width: 36px;
  height: 36px;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  color: white;
  border: none;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  background: rgba(0, 0, 0, 0.8);
  transform: scale(1.1);
}

/* Format badge */
.format-badge {
  position: absolute;
  top: 8px;
  right: 8px;
  padding: 2px 8px;
  background: rgba(0, 0, 0, 0.7);
  color: white;
  font-size: 11px;
  font-weight: 600;
  border-radius: 4px;
  letter-spacing: 0.5px;
}

/* Loading progress */
.loading-progress {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: rgba(0, 0, 0, 0.1);
}

.progress-bar {
  height: 100%;
  background: var(--primary, #3b82f6);
  transition: width 0.3s ease;
}

/* Full view modal */
.full-view-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  cursor: zoom-out;
}

.modal-content {
  position: relative;
  max-width: 90vw;
  max-height: 90vh;
}

.full-image {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: contain;
  border-radius: 8px;
}

.close-btn {
  position: absolute;
  top: -40px;
  right: 0;
  width: 40px;
  height: 40px;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(4px);
  color: white;
  border: none;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: scale(1.1);
}

/* Transitions */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .enhanced-image-thumbnail {
    background: var(--bg-secondary, #374151);
  }

  .thumbnail-loading,
  .thumbnail-error {
    color: var(--text-muted, #9ca3af);
  }
}

/* Responsive */
@media (max-width: 768px) {
  .mode-inline {
    max-width: 100%;
  }

  .overlay-info {
    font-size: 12px;
  }

  .action-btn {
    width: 32px;
    height: 32px;
  }
}
</style>