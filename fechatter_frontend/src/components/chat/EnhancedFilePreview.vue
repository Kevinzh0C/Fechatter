<template>
  <div class="enhanced-file-preview">
    <!-- Selected Files Preview -->
    <div v-if="files.length > 0" class="files-container">
      <div class="files-header">
        <span class="files-count">{{ files.length }} file{{ files.length > 1 ? 's' : '' }} selected</span>
        <div class="files-actions">
          <button @click="toggleViewMode" class="view-toggle" :title="viewMode === 'grid' ? 'Switch to list view' : 'Switch to grid view'">
            <svg v-if="viewMode === 'grid'" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M4 6h16v2H4zm0 5h16v2H4zm0 5h16v2H4z"/>
            </svg>
            <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M4 11h5V5H4v6zm0 7h5v-6H4v6zm6 0h5v-6h-5v6zm6 0h5v-6h-5v6zm-6-7h5V5h-5v6zm6-6v6h5V5h-5z"/>
            </svg>
          </button>
          <button @click="clearAllFiles" class="clear-all-btn" title="Clear all files">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
            </svg>
            Clear All
          </button>
        </div>
      </div>
      
      <!-- Grid View -->
      <div v-if="viewMode === 'grid'" class="files-grid" :class="gridClasses">
        <div v-for="(file, index) in files" :key="file.id || index" 
             class="file-item-grid" 
             :class="{ 'file-loading': file.loading, 'file-error': file.error }">
          
          <!-- Image Preview -->
          <div v-if="isImageFile(file)" class="image-preview-container">
            <img 
              v-if="file.preview" 
              :src="file.preview" 
              :alt="file.name"
              class="image-preview"
              @click="openImageViewer(file)"
              @load="onImageLoad(file)"
              @error="onImageError(file)"
            />
            <div v-else class="image-placeholder">
              <div class="loading-spinner"></div>
            </div>
          </div>
          
          <!-- File Icon for Non-Images -->
          <div v-else class="file-icon-container">
            <div class="file-icon" :class="getFileTypeClass(file)">
              <svg v-if="isVideoFile(file)" width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path d="M17 10.5V7a1 1 0 0 0-1-1H4a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-3.5l4 4v-11l-4 4z"/>
              </svg>
              <svg v-else-if="isAudioFile(file)" width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/>
              </svg>
              <svg v-else-if="isDocumentFile(file)" width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z"/>
              </svg>
              <svg v-else width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path d="M13,9V3.5L18.5,9M6,2C4.89,2 4,2.89 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2H6Z"/>
              </svg>
            </div>
          </div>
          
          <!-- File Info -->
          <div class="file-info">
            <div class="file-name" :title="file.name">{{ file.name }}</div>
            <div class="file-meta">
              <span class="file-size">{{ formatFileSize(file.size) }}</span>
              <span v-if="file.type" class="file-type">{{ getFileExtension(file.name) }}</span>
            </div>
          </div>
          
          <!-- Progress Bar (for uploads) -->
          <div v-if="file.loading && file.progress !== undefined" class="upload-progress">
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: file.progress + '%' }"></div>
            </div>
            <span class="progress-text">{{ file.progress }}%</span>
          </div>
          
          <!-- Actions -->
          <div class="file-actions">
            <button v-if="canPreview(file)" @click="previewFile(file)" class="action-btn" title="Preview">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12,9A3,3 0 0,0 9,12A3,3 0 0,0 12,15A3,3 0 0,0 15,12A3,3 0 0,0 12,9M12,17A5,5 0 0,1 7,12A5,5 0 0,1 12,7A5,5 0 0,1 17,12A5,5 0 0,1 12,17M12,4.5C7,4.5 2.73,7.61 1,12C2.73,16.39 7,19.5 12,19.5C17,19.5 21.27,16.39 23,12C21.27,7.61 17,4.5 12,4.5Z"/>
              </svg>
            </button>
            <button @click="removeFile(index)" class="action-btn remove-btn" title="Remove file">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19,6.41L17.59,5 12,10.59 6.41,5 5,6.41 10.59,12 5,17.59 6.41,19 12,13.41 17.59,19 19,17.59 13.41,12Z"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
      
      <!-- List View -->
      <div v-else class="files-list">
        <div v-for="(file, index) in files" :key="file.id || index" 
             class="file-item-list" 
             :class="{ 'file-loading': file.loading, 'file-error': file.error }">
          
          <!-- File Icon -->
          <div class="file-icon-small" :class="getFileTypeClass(file)">
            <svg v-if="isImageFile(file)" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M8.5,13.5L11,16.5L14.5,12L19,18H5M21,19V5C21,3.89 20.1,3 19,3H5A2,2 0 0,0 3,5V19A2,2 0 0,0 5,21H19A2,2 0 0,0 21,19Z"/>
            </svg>
            <svg v-else-if="isVideoFile(file)" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M17 10.5V7a1 1 0 0 0-1-1H4a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-3.5l4 4v-11l-4 4z"/>
            </svg>
            <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M13,9V3.5L18.5,9M6,2C4.89,2 4,2.89 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2H6Z"/>
            </svg>
          </div>
          
          <!-- File Details -->
          <div class="file-details">
            <div class="file-name" :title="file.name">{{ file.name }}</div>
            <div class="file-meta">
              <span class="file-size">{{ formatFileSize(file.size) }}</span>
              <span class="file-type">{{ getFileExtension(file.name) }}</span>
              <span v-if="file.lastModified" class="file-date">{{ formatDate(file.lastModified) }}</span>
            </div>
          </div>
          
          <!-- Progress (for uploads) -->
          <div v-if="file.loading && file.progress !== undefined" class="file-progress">
            <div class="progress-circle">
              <svg class="progress-ring" width="20" height="20">
                <circle cx="10" cy="10" r="8" fill="none" stroke="#e5e7eb" stroke-width="2"/>
                <circle cx="10" cy="10" r="8" fill="none" stroke="#3b82f6" stroke-width="2"
                        :stroke-dasharray="50.26" 
                        :stroke-dashoffset="50.26 - (file.progress / 100) * 50.26"
                        transform="rotate(-90 10 10)"/>
              </svg>
              <span class="progress-percent">{{ Math.round(file.progress) }}%</span>
            </div>
          </div>
          
          <!-- Actions -->
          <div class="file-actions-list">
            <button v-if="canPreview(file)" @click="previewFile(file)" class="action-btn" title="Preview">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12,9A3,3 0 0,0 9,12A3,3 0 0,0 12,15A3,3 0 0,0 15,12A3,3 0 0,0 12,9M12,17A5,5 0 0,1 7,12A5,5 0 0,1 12,7A5,5 0 0,1 17,12A5,5 0 0,1 12,17M12,4.5C7,4.5 2.73,7.61 1,12C2.73,16.39 7,19.5 12,19.5C17,19.5 21.27,16.39 23,12C21.27,7.61 17,4.5 12,4.5Z"/>
              </svg>
            </button>
            <button @click="removeFile(index)" class="action-btn remove-btn" title="Remove file">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19,6.41L17.59,5 12,10.59 6.41,5 5,6.41 10.59,12 5,17.59 6.41,19 12,13.41 17.59,19 19,17.59 13.41,12Z"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Enhanced Drag and Drop Zone -->
    <div v-if="showDropZone" 
         class="enhanced-drop-zone" 
         :class="{ 'drop-zone-active': isDragOver }"
         @dragover.prevent="handleDragOver"
         @dragleave="handleDragLeave"
         @drop.prevent="handleDrop">
      
      <div class="drop-zone-content">
        <div class="drop-icon">
          <svg width="64" height="64" viewBox="0 0 24 24" fill="currentColor">
            <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z"/>
          </svg>
        </div>
        <h3>Drop files here to upload</h3>
        <p>Support for images, documents, videos, and more</p>
        <div class="supported-formats">
          <span class="format-tag">Images</span>
          <span class="format-tag">Documents</span>
          <span class="format-tag">Videos</span>
          <span class="format-tag">Audio</span>
        </div>
      </div>
    </div>

    <!-- File Preview Modal -->
    <div v-if="previewFile" class="preview-modal-overlay" @click="closePreview">
      <div class="preview-modal" @click.stop>
        <div class="preview-header">
          <h3>{{ previewFile.name }}</h3>
          <button @click="closePreview" class="close-preview">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19,6.41L17.59,5 12,10.59 6.41,5 5,6.41 10.59,12 5,17.59 6.41,19 12,13.41 17.59,19 19,17.59 13.41,12Z"/>
            </svg>
          </button>
        </div>
        
        <div class="preview-content">
          <!-- Image Preview -->
          <img v-if="isImageFile(previewFile) && previewFile.preview" 
               :src="previewFile.preview" 
               :alt="previewFile.name"
               class="preview-image">
          
          <!-- Video Preview -->
          <video v-else-if="isVideoFile(previewFile) && previewFile.preview" 
                 :src="previewFile.preview" 
                 controls 
                 class="preview-video">
          </video>
          
          <!-- Audio Preview -->
          <audio v-else-if="isAudioFile(previewFile) && previewFile.preview" 
                 :src="previewFile.preview" 
                 controls 
                 class="preview-audio">
          </audio>
          
          <!-- Text/Code Preview -->
          <div v-else-if="isTextFile(previewFile)" class="preview-text">
            <pre><code>{{ previewFile.textContent || 'Loading...' }}</code></pre>
          </div>
          
          <!-- Generic File Info -->
          <div v-else class="preview-info">
            <div class="large-file-icon" :class="getFileTypeClass(previewFile)">
              <svg width="64" height="64" viewBox="0 0 24 24" fill="currentColor">
                <path d="M13,9V3.5L18.5,9M6,2C4.89,2 4,2.89 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2H6Z"/>
              </svg>
            </div>
            <h4>{{ previewFile.name }}</h4>
            <p>Size: {{ formatFileSize(previewFile.size) }}</p>
            <p>Type: {{ previewFile.type || 'Unknown' }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick } from 'vue';

const props = defineProps({
  files: {
    type: Array,
    default: () => []
  },
  showDropZone: {
    type: Boolean,
    default: false
  },
  maxFiles: {
    type: Number,
    default: 10
  },
  maxFileSize: {
    type: Number,
    default: 50 * 1024 * 1024 // 50MB
  }
});

const emit = defineEmits(['files-updated', 'file-removed', 'files-cleared', 'files-dropped']);

// State
const viewMode = ref('grid'); // 'grid' or 'list'
const isDragOver = ref(false);
const previewFile = ref(null);

// Computed
const gridClasses = computed(() => ({
  'grid-cols-2': props.files.length <= 2,
  'grid-cols-3': props.files.length > 2 && props.files.length <= 6,
  'grid-cols-4': props.files.length > 6
}));

// File type detection
const isImageFile = (file) => {
  return file.type?.startsWith('image/') || /\.(jpg|jpeg|png|gif|webp|svg|bmp)$/i.test(file.name);
};

const isVideoFile = (file) => {
  return file.type?.startsWith('video/') || /\.(mp4|avi|mov|wmv|flv|webm|mkv)$/i.test(file.name);
};

const isAudioFile = (file) => {
  return file.type?.startsWith('audio/') || /\.(mp3|wav|ogg|m4a|aac|flac)$/i.test(file.name);
};

const isDocumentFile = (file) => {
  return /\.(pdf|doc|docx|xls|xlsx|ppt|pptx|txt|rtf)$/i.test(file.name);
};

const isTextFile = (file) => {
  return file.type?.startsWith('text/') || /\.(txt|json|xml|html|css|js|ts|md|yml|yaml)$/i.test(file.name);
};

const canPreview = (file) => {
  return isImageFile(file) || isVideoFile(file) || isAudioFile(file) || isTextFile(file);
};

// File utilities
const getFileExtension = (filename) => {
  return filename.split('.').pop()?.toUpperCase() || '';
};

const getFileTypeClass = (file) => {
  if (isImageFile(file)) return 'file-type-image';
  if (isVideoFile(file)) return 'file-type-video';
  if (isAudioFile(file)) return 'file-type-audio';
  if (isDocumentFile(file)) return 'file-type-document';
  if (isTextFile(file)) return 'file-type-text';
  return 'file-type-generic';
};

const formatFileSize = (bytes) => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};

const formatDate = (timestamp) => {
  return new Date(timestamp).toLocaleDateString();
};

// Actions
const toggleViewMode = () => {
  viewMode.value = viewMode.value === 'grid' ? 'list' : 'grid';
};

const removeFile = (index) => {
  emit('file-removed', index);
};

const clearAllFiles = () => {
  emit('files-cleared');
};

const previewFileHandler = (file) => {
  previewFile.value = file;
  
  // Load text content for text files if not already loaded
  if (isTextFile(file) && !file.textContent) {
    const reader = new FileReader();
    reader.onload = (e) => {
      file.textContent = e.target.result;
    };
    reader.readAsText(file);
  }
};

const closePreview = () => {
  previewFile.value = null;
};

// Drag and drop handlers
const handleDragOver = (event) => {
  event.preventDefault();
  isDragOver.value = true;
};

const handleDragLeave = (event) => {
  // Only set to false if we're leaving the drop zone entirely
  if (!event.currentTarget.contains(event.relatedTarget)) {
    isDragOver.value = false;
  }
};

const handleDrop = (event) => {
  event.preventDefault();
  isDragOver.value = false;
  
  const droppedFiles = Array.from(event.dataTransfer.files);
  emit('files-dropped', droppedFiles);
};

// Image loading handlers
const onImageLoad = (file) => {
  file.loaded = true;
};

const onImageError = (file) => {
  file.error = true;
};

const openImageViewer = (file) => {
  // This could open a dedicated image viewer component
  previewFileHandler(file);
};

// Watch for file changes to generate previews
watch(() => props.files, (newFiles) => {
  newFiles.forEach(file => {
    if (isImageFile(file) && !file.preview && !file.previewLoading) {
      file.previewLoading = true;
      const reader = new FileReader();
      reader.onload = (e) => {
        file.preview = e.target.result;
        file.previewLoading = false;
      };
      reader.readAsDataURL(file);
    }
  });
}, { deep: true, immediate: true });
</script>

<style scoped>
.enhanced-file-preview {
  width: 100%;
}

/* Files Container */
.files-container {
  background: rgba(248, 246, 243, 0.4);
  border-bottom: 1px solid #e5e7eb;
  padding: 16px 24px;
}

.files-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.files-count {
  font-size: 14px;
  font-weight: 600;
  color: #374151;
}

.files-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.view-toggle,
.clear-all-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  background: white;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 12px;
  color: #6b7280;
  cursor: pointer;
  transition: all 0.2s;
}

.view-toggle:hover,
.clear-all-btn:hover {
  background: #f9fafb;
  border-color: #9ca3af;
}

.clear-all-btn {
  color: #dc2626;
  border-color: #fecaca;
}

.clear-all-btn:hover {
  background: #fef2f2;
  border-color: #dc2626;
}

/* Grid View */
.files-grid {
  display: grid;
  gap: 12px;
}

.file-item-grid {
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 12px;
  position: relative;
  transition: all 0.2s;
}

.file-item-grid:hover {
  border-color: #d1d5db;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.file-item-grid.file-loading {
  opacity: 0.7;
}

.file-item-grid.file-error {
  border-color: #fca5a5;
  background: #fef2f2;
}

/* Image Preview */
.image-preview-container {
  width: 100%;
  height: 80px;
  border-radius: 6px;
  overflow: hidden;
  margin-bottom: 8px;
  background: #f9fafb;
  display: flex;
  align-items: center;
  justify-content: center;
}

.image-preview {
  width: 100%;
  height: 100%;
  object-fit: cover;
  cursor: pointer;
  transition: transform 0.2s;
}

.image-preview:hover {
  transform: scale(1.05);
}

.image-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.loading-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid #e5e7eb;
  border-top: 2px solid #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* File Icon */
.file-icon-container {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 80px;
  margin-bottom: 8px;
}

.file-icon {
  width: 48px;
  height: 48px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.file-type-image { background: #3b82f6; }
.file-type-video { background: #8b5cf6; }
.file-type-audio { background: #06b6d4; }
.file-type-document { background: #f59e0b; }
.file-type-text { background: #10b981; }
.file-type-generic { background: #6b7280; }

/* File Info */
.file-info {
  text-align: center;
}

.file-name {
  font-size: 12px;
  font-weight: 500;
  color: #374151;
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-meta {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 10px;
  color: #9ca3af;
}

.file-size, .file-type {
  background: #f3f4f6;
  padding: 2px 6px;
  border-radius: 4px;
}

/* Progress */
.upload-progress {
  margin-top: 8px;
}

.progress-bar {
  width: 100%;
  height: 4px;
  background: #e5e7eb;
  border-radius: 2px;
  overflow: hidden;
  margin-bottom: 4px;
}

.progress-fill {
  height: 100%;
  background: #3b82f6;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 10px;
  color: #6b7280;
  text-align: center;
  display: block;
}

/* File Actions */
.file-actions {
  position: absolute;
  top: 4px;
  right: 4px;
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s;
}

.file-item-grid:hover .file-actions {
  opacity: 1;
}

.action-btn {
  width: 24px;
  height: 24px;
  background: rgba(0, 0, 0, 0.7);
  color: white;
  border: none;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  background: rgba(0, 0, 0, 0.9);
}

.remove-btn {
  background: rgba(220, 38, 38, 0.8);
}

.remove-btn:hover {
  background: rgba(220, 38, 38, 1);
}

/* List View */
.files-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.file-item-list {
  display: flex;
  align-items: center;
  gap: 12px;
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  padding: 12px;
  transition: all 0.2s;
}

.file-item-list:hover {
  border-color: #d1d5db;
  background: #f9fafb;
}

.file-icon-small {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.file-details {
  flex: 1;
  min-width: 0;
}

.file-details .file-name {
  font-size: 14px;
  font-weight: 500;
  color: #374151;
  margin-bottom: 2px;
  text-align: left;
}

.file-details .file-meta {
  justify-content: flex-start;
  font-size: 12px;
}

.file-date {
  color: #9ca3af;
}

.file-progress {
  position: relative;
  flex-shrink: 0;
}

.progress-circle {
  position: relative;
  width: 20px;
  height: 20px;
}

.progress-ring {
  transform: rotate(-90deg);
}

.progress-percent {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 8px;
  font-weight: 600;
  color: #3b82f6;
}

.file-actions-list {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

/* Enhanced Drop Zone */
.enhanced-drop-zone {
  border: 2px dashed #d1d5db;
  border-radius: 12px;
  padding: 40px 20px;
  text-align: center;
  background: #f9fafb;
  transition: all 0.3s ease;
  margin: 16px 24px;
}

.enhanced-drop-zone.drop-zone-active {
  border-color: #3b82f6;
  background: #eff6ff;
  transform: scale(1.02);
}

.drop-zone-content {
  color: #6b7280;
}

.drop-icon {
  margin: 0 auto 16px;
  color: #9ca3af;
}

.drop-zone-active .drop-icon {
  color: #3b82f6;
  animation: bounce 0.5s ease-in-out;
}

@keyframes bounce {
  0%, 20%, 60%, 100% { transform: translateY(0); }
  40% { transform: translateY(-10px); }
  80% { transform: translateY(-5px); }
}

.enhanced-drop-zone h3 {
  margin: 0 0 8px 0;
  font-size: 18px;
  font-weight: 600;
  color: #374151;
}

.enhanced-drop-zone p {
  margin: 0 0 16px 0;
  font-size: 14px;
}

.supported-formats {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  flex-wrap: wrap;
}

.format-tag {
  background: #e5e7eb;
  color: #374151;
  padding: 4px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 500;
}

.drop-zone-active .format-tag {
  background: #dbeafe;
  color: #1d4ed8;
}

/* Preview Modal */
.preview-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.preview-modal {
  background: white;
  border-radius: 12px;
  max-width: 90vw;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid #e5e7eb;
}

.preview-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #374151;
}

.close-preview {
  background: none;
  border: none;
  color: #6b7280;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-preview:hover {
  background: #f3f4f6;
  color: #374151;
}

.preview-content {
  flex: 1;
  overflow: auto;
  padding: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.preview-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

.preview-video {
  max-width: 100%;
  max-height: 100%;
}

.preview-audio {
  width: 100%;
  max-width: 400px;
}

.preview-text {
  width: 100%;
  max-width: 600px;
  max-height: 400px;
  overflow: auto;
  background: #f9fafb;
  border-radius: 6px;
  padding: 16px;
}

.preview-text pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: 'SF Mono', Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.5;
}

.preview-info {
  text-align: center;
}

.large-file-icon {
  width: 80px;
  height: 80px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  margin: 0 auto 16px;
}

.preview-info h4 {
  margin: 0 0 8px 0;
  font-size: 18px;
  font-weight: 600;
  color: #374151;
}

.preview-info p {
  margin: 4px 0;
  color: #6b7280;
}

/* Mobile Responsive */
@media (max-width: 768px) {
  .files-container {
    padding: 12px 16px;
  }

  .files-grid {
    grid-template-columns: repeat(2, 1fr) !important;
    gap: 8px;
  }

  .file-item-grid {
    padding: 8px;
  }

  .image-preview-container {
    height: 60px;
  }

  .file-icon {
    width: 32px;
    height: 32px;
  }

  .file-icon svg {
    width: 16px;
    height: 16px;
  }

  .enhanced-drop-zone {
    margin: 12px 16px;
    padding: 24px 16px;
  }

  .preview-modal {
    max-width: 95vw;
    max-height: 95vh;
  }

  .preview-content {
    padding: 16px;
  }
}
</style>