<template>
  <div class="file-preview-container">
    <!-- Enhanced Image Preview for Images -->
    <EnhancedImageThumbnail v-if="isImage" :file="file" :mode="'inline'" :max-width="350" :max-height="250"
      @open="handleImageOpen" @download="downloadFile" />

    <!-- PDF Preview with Thumbnail -->
    <div v-else-if="isPDF" class="pdf-file-preview">
      <div class="pdf-thumbnail">
        <div class="pdf-icon">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z" />
            <path d="M9,13V19H7V13H9M12,13V19H10V15H12V13M15,13V19H13V16H15V13Z" />
          </svg>
        </div>
        <div class="pdf-badge">PDF</div>
      </div>
      <div class="file-info">
        <div class="file-name" :title="file.filename || file.file_name">{{ displayFileName }}</div>
        <div class="file-meta">
          <span class="file-size">{{ formattedSize }}</span>
          <span class="file-extension">{{ fileExtension }}</span>
        </div>
      </div>
      <div class="file-actions">
        <button @click="downloadFile" class="action-button download-btn" title="Download PDF">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="7,10 12,15 17,10" />
            <line x1="12" y1="15" x2="12" y2="3" />
          </svg>
        </button>
        <button @click="openFile" class="action-button open-btn" title="Open PDF">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
            <polyline points="15,3 21,3 21,9" />
            <line x1="10" y1="14" x2="21" y2="3" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Generic File Preview for Other Files -->
    <div v-else class="generic-file-preview">
      <div class="file-icon">
        <Icon :name="getFileIcon" />
      </div>
      <div class="file-info">
        <div class="file-name" :title="file.filename || file.file_name">{{ displayFileName }}</div>
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
import { getStandardFileUrl } from '@/utils/fileUrlHandler';
import EnhancedImageThumbnail from './EnhancedImageThumbnail.vue';
import api from '@/services/api';
import { useAuthStore } from '@/stores/auth';

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

const authStore = useAuthStore();
const showFullImage = ref(false);
const showImagePreview = ref(true);

const isImage = computed(() => {
  const mimeType = props.file.mime_type || props.file.type || '';
  const filename = props.file.filename || props.file.file_name || '';
  return mimeType.startsWith('image/') ||
    /\.(jpg|jpeg|png|gif|webp|svg|bmp|ico|heic|heif)$/i.test(filename);
});

const isPDF = computed(() => {
  const mimeType = props.file.mime_type || props.file.type || '';
  const filename = props.file.filename || props.file.file_name || '';
  return mimeType.includes('pdf') || filename.toLowerCase().endsWith('.pdf');
});

const displayFileName = computed(() => {
  const filename = props.file.filename || props.file.file_name || props.file.name || 'Unknown file';
  const maxLength = 25;
  if (filename.length <= maxLength) {
    return filename;
  }
  const parts = filename.split('.');
  if (parts.length > 1) {
    const extension = parts.pop();
    const baseName = parts.join('.');
    const truncateLength = maxLength - extension.length - 4; // 4 for "..." and "."
    if (baseName.length > truncateLength) {
      return `${baseName.substring(0, truncateLength)}...${extension}`;
    }
  }
  return `${filename.substring(0, maxLength - 3)}...`;
});

const fileExtension = computed(() => {
  const filename = props.file.filename || props.file.file_name || props.file.name || '';
  if (!filename) return '';
  const parts = filename.split('.');
  if (parts.length > 1) {
    const ext = parts.pop().toUpperCase();
    return ext.length <= 5 ? ext : 'FILE';
  }
  return '';
});

const formattedSize = computed(() => {
  const size = props.file.size || props.file.file_size || 0;
  if (!size) return 'Unknown size';
  const kb = size / 1024;
  if (kb < 1024) return `${kb.toFixed(1)} KB`;
  return `${(kb / 1024).toFixed(1)} MB`;
});

const getFileIcon = computed(() => {
  const filename = props.file.filename || props.file.file_name || '';
  const mimeType = props.file.mime_type || props.file.type || '';

  // Check by MIME type first
  if (mimeType.startsWith('video/')) return 'videocam-outline';
  if (mimeType.startsWith('audio/')) return 'musical-notes-outline';
  if (mimeType.includes('pdf')) return 'document-text-outline';
  if (mimeType.includes('word') || mimeType.includes('document')) return 'document-outline';
  if (mimeType.includes('sheet') || mimeType.includes('excel')) return 'grid-outline';
  if (mimeType.includes('zip') || mimeType.includes('archive')) return 'archive-outline';

  // Check by file extension
  const ext = filename.split('.').pop()?.toLowerCase();
  switch (ext) {
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

// 🔧 Enhanced file URL getter with workspace support and hash path handling
const getFileUrl = (file) => {
  // 🔧 Use unified file URL handler to automatically handle all formats
  return getStandardFileUrl(file, {
    workspaceId: props.workspaceId || authStore.user?.workspace_id || 2
  });
};

// 🔐 Enhanced download with authentication
const downloadFile = async () => {
  const fileName = props.file.filename || props.file.file_name || props.file.name || 'file';

  try {
    const fileUrl = getFileUrl(props.file);
    if (!fileUrl) {
      console.error('❌ No URL available for file download:', fileName);
      return;
    }

    console.log('📥 [FilePreview] Starting download:', fileName, 'URL:', fileUrl);

    // 🔐 For API URLs, download using authenticated fetch
    if (fileUrl.startsWith('/api/')) {
      // Remove /api/ prefix since api client adds it automatically
      let apiPath = fileUrl.substring(5);

      // Use authenticated API client
      const response = await api.get(apiPath, {
        responseType: 'blob',
        skipAuthRefresh: false
      });

      if (response.data) {
        // Create blob URL and trigger download
        const blob = response.data;
        const url = URL.createObjectURL(blob);

        const link = document.createElement('a');
        link.href = url;
        link.download = fileName;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);

        // Clean up the blob URL
        setTimeout(() => URL.revokeObjectURL(url), 1000);

        console.log('✅ [FilePreview] Downloaded authenticated file:', fileName);
      } else {
        throw new Error('No file data received');
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

      console.log('✅ [FilePreview] Downloaded file:', fileName);
    }
  } catch (error) {
    console.error('❌ [FilePreview] Download failed for file:', fileName, error);

    // Show user-friendly error message
    if (typeof window !== 'undefined' && window.alert) {
      alert(`Failed to download ${fileName}. Please try again.`);
    }
  }
};

// Open file in new tab (for PDFs and other viewable files)
const openFile = async () => {
  try {
    const fileUrl = getFileUrl(props.file);
    if (fileUrl && fileUrl.startsWith('/api/')) {
      // For authenticated files, we need to create a blob URL first
      const apiPath = fileUrl.substring(5);
      const response = await api.get(apiPath, {
        responseType: 'blob',
        skipAuthRefresh: false
      });

      if (response.data) {
        const blob = response.data;
        const url = URL.createObjectURL(blob);
        window.open(url, '_blank');

        // Clean up after a delay
        setTimeout(() => URL.revokeObjectURL(url), 5000);
      }
    } else if (fileUrl) {
      window.open(fileUrl, '_blank');
    }
  } catch (error) {
    console.error('❌ [FilePreview] Failed to open file:', error);
    alert('Failed to open file. Please try downloading instead.');
  }
};

const handleImageOpen = () => {
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
  cursor: default;
  user-select: none;
}

.file-preview-container:hover {
  background-color: #f1f3f4;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

/* PDF Preview Styles */
.pdf-file-preview {
  display: flex;
  align-items: center;
  flex-grow: 1;
  gap: 12px;
}

.pdf-thumbnail {
  position: relative;
  flex-shrink: 0;
  width: 48px;
  height: 48px;
  background: linear-gradient(135deg, #dc2626, #ef4444);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.pdf-icon {
  display: flex;
  align-items: center;
  justify-content: center;
}

.pdf-badge {
  position: absolute;
  bottom: -6px;
  right: -6px;
  background: #dc2626;
  color: white;
  font-size: 8px;
  font-weight: bold;
  padding: 2px 4px;
  border-radius: 3px;
  line-height: 1;
}

/* Generic File Preview */
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
  white-space: nowrap;
  text-overflow: ellipsis;
  overflow: hidden;
}

.file-meta {
  font-size: 12px;
  color: #6b7280;
  line-height: 1.3;
  white-space: nowrap;
  display: flex;
  gap: 8px;
}

.file-extension {
  background: #e5e7eb;
  color: #374151;
  padding: 1px 4px;
  border-radius: 3px;
  font-weight: 500;
}

/* Action Buttons */
.file-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.action-button {
  background: none;
  border: none;
  cursor: pointer;
  color: #6b7280;
  padding: 6px;
  border-radius: 6px;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.action-button:hover {
  background-color: #e5e7eb;
  color: #1f2937;
  transform: scale(1.05);
}

.download-btn:hover {
  background-color: #dbeafe;
  color: #2563eb;
}

.open-btn:hover {
  background-color: #dcfce7;
  color: #16a34a;
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

  .file-extension {
    background: #4b5563;
    color: #e5e7eb;
  }

  .action-button {
    color: #d1d5db;
  }

  .action-button:hover {
    background-color: #4b5563;
    color: #f3f4f6;
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