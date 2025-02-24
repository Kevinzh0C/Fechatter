<template>
  <div class="file-preview-container">
    <!-- Enhanced Image Preview for Images -->
    <EnhancedImageThumbnail 
      v-if="isImage" 
      :file="file"
      :mode="'inline'"
      :max-width="350"
      :max-height="250"
      @open="handleImageOpen"
      @download="downloadFile"
    />

    <!-- Generic File Preview for Non-Images -->
    <div v-else class="generic-file-preview">
      <div class="file-icon">
        <Icon :name="getFileIcon" />
      </div>
      <div class="file-info">
        <div class="file-name" :title="file.filename">{{ displayFileName }}</div>
        <div class="file-meta">
          <span class="file-size">{{ formattedSize }}</span>
          <span v-if="fileExtension" class="file-extension">{{ fileExtension }}</span>
        </div>
      </div>
      <button @click="downloadFile" class="download-button" title="Download">
        <Icon name="download-outline" />
      </button>
    </div>
  </div>
</template>

<script setup>
import { computed, ref } from 'vue';
import Icon from '@/components/ui/Icon.vue';
import EnhancedImageThumbnail from './EnhancedImageThumbnail.vue';

const props = defineProps({
  file: {
    type: Object,
    required: true,
  },
  workspaceId: {
    type: Number,
    required: false,
  },
});

const showFullImage = ref(false);
const showImagePreview = ref(true);

const isImage = computed(() => props.file.mime_type && props.file.mime_type.startsWith('image/'));

const displayFileName = computed(() => {
  if (!props.file.filename) return 'Unknown file';
  const maxLength = 25;
  if (props.file.filename.length <= maxLength) {
    return props.file.filename;
  }
  const parts = props.file.filename.split('.');
  if (parts.length > 1) {
    const extension = parts.pop();
    const baseName = parts.join('.');
    const truncateLength = maxLength - extension.length - 4; // 4 for "..." and "."
    if (baseName.length > truncateLength) {
      return `${baseName.substring(0, truncateLength)}...${extension}`;
    }
  }
  return `${props.file.filename.substring(0, maxLength - 3)}...`;
});

const fileExtension = computed(() => {
  if (!props.file.filename) return '';
  const parts = props.file.filename.split('.');
  if (parts.length > 1) {
    const ext = parts.pop().toUpperCase();
    // Avoid showing very long extensions
    return ext.length <= 5 ? ext : 'FILE';
  }
  return '';
});

const formattedSize = computed(() => {
  if (!props.file.size) return 'Unknown size';
  const kb = props.file.size / 1024;
  if (kb < 1024) return `${kb.toFixed(1)} KB`;
  return `${(kb / 1024).toFixed(1)} MB`;
});

const getFileIcon = computed(() => {
  const filename = props.file.filename || '';
  const mimeType = props.file.mime_type || '';
  
  // Check by MIME type first
  if (mimeType.startsWith('video/')) return 'videocam-outline';
  if (mimeType.startsWith('audio/')) return 'musical-notes-outline';
  if (mimeType.includes('pdf')) return 'document-text-outline';
  if (mimeType.includes('word') || mimeType.includes('document')) return 'document-outline';
  if (mimeType.includes('sheet') || mimeType.includes('excel')) return 'grid-outline';
  if (mimeType.includes('zip') || mimeType.includes('archive')) return 'archive-outline';
  
  // Check by file extension
  const ext = filename.split('.').pop()?.toLowerCase();
  switch(ext) {
    case 'pdf': return 'document-text-outline';
    case 'doc':
    case 'docx': return 'document-outline';
    case 'xls':
    case 'xlsx': return 'grid-outline';
    case 'zip':
    case 'rar':
    case '7z': return 'archive-outline';
    case 'mp4':
    case 'avi':
    case 'mov': return 'videocam-outline';
    case 'mp3':
    case 'wav':
    case 'flac': return 'musical-notes-outline';
    default: return 'document-outline';
  }
});

const downloadFile = () => {
  // Create a temporary link to trigger the download
  const link = document.createElement('a');
  link.href = props.file.url;
  link.download = props.file.filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
};

const handleImageOpen = () => {
  // Emit event or handle image opening
  console.log('Opening image:', props.file);
};

const handleImageError = () => {
  console.warn('Failed to load image:', props.file.url);
  showImagePreview.value = false;
};

const handleImageLoad = () => {
  showImagePreview.value = true;
};

const handleModalImageError = () => {
  showFullImage.value = false;
  showImagePreview.value = false;
};
</script>

<style scoped>
.file-preview-container {
  display: flex;
  align-items: center;
  background-color: #f8f9fa;
  border: 1px solid #e9ecef;
  border-radius: 12px;
  padding: 12px;
  max-width: 350px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  transition: all 0.2s ease;
  /* 🔧 CRITICAL FIX: Ensure it doesn't look like an input */
  cursor: default;
  user-select: none;
}

.file-preview-container:hover {
  background-color: #f1f3f4;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

.image-preview {
  cursor: pointer;
  display: flex;
  align-items: center;
}

.thumbnail-image {
  width: 48px;
  height: 48px;
  border-radius: 8px;
  object-fit: cover;
  margin-right: 12px;
  transition: transform 0.2s ease;
}

.thumbnail-image:hover {
  transform: scale(1.05);
}

.image-error-placeholder {
  width: 48px;
  height: 48px;
  border-radius: 8px;
  background-color: #e5e7eb;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  margin-right: 12px;
  border: 2px dashed #9ca3af;
}

.image-error-placeholder Icon {
  width: 20px;
  height: 20px;
  color: #6b7280;
  margin-bottom: 2px;
}

.error-text {
  font-size: 8px;
  color: #6b7280;
  text-align: center;
}

.generic-file-preview {
  display: flex;
  align-items: center;
  flex-grow: 1;
  gap: 12px;
}

.file-icon {
  font-size: 24px;
  color: #6b7280;
  flex-shrink: 0;
}

.file-info {
  display: flex;
  flex-direction: column;
  flex-grow: 1;
  min-width: 0;
}

.file-name {
  font-weight: 600;
  color: #1f2937;
  font-size: 14px;
  line-height: 1.4;
  margin-bottom: 2px;
  word-break: break-word;
  /* 🔧 CRITICAL FIX: Make sure it's clearly not an input */
  background: none;
  border: none;
  outline: none;
  resize: none;
  overflow: visible;
  white-space: nowrap;
  text-overflow: ellipsis;
  overflow: hidden;
}

.file-meta {
  font-size: 12px;
  color: #6b7280;
  line-height: 1.3;
  /* 🔧 CRITICAL FIX: Prevent input-like appearance */
  background: none;
  border: none;
  outline: none;
  white-space: nowrap;
}

.download-button {
  margin-left: auto;
  background: none;
  border: none;
  cursor: pointer;
  color: #6b7280;
  padding: 8px;
  border-radius: 8px;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.download-button:hover {
  background-color: #e5e7eb;
  color: #1f2937;
  transform: scale(1.1);
}

.full-image-modal {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.8);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: var(--z-modal, 9500);
  cursor: pointer;
  /* Ensure proper stacking context */
  isolation: isolate;
}

.full-image {
  max-width: 90%;
  max-height: 90%;
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .file-preview-container {
    background-color: #374151;
    border-color: #4b5563;
  }

  .file-preview-container:hover {
    background-color: #4b5563;
  }

  .file-name {
    color: #f3f4f6;
  }

  .file-meta {
    color: #d1d5db;
  }

  .download-button {
    color: #d1d5db;
  }

  .download-button:hover {
    background-color: #4b5563;
    color: #f3f4f6;
  }
}
</style>