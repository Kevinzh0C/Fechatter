<template>
  <div class="file-preview-container">
    <!-- 🎯 统一的文件预览布局 - 适用于所有状态 -->
    <div class="unified-file-preview">
      <div class="file-preview-horizontal">
        <!-- 左侧：文件缩略图和信息（始终保持一致） -->
        <div class="file-thumbnail-section">
          <!-- 🎯 统一的缩略图容器 -->
          <div class="unified-thumbnail-container">
            <img v-if="isImage(file)" :src="getFilePreviewUrl()" :alt="file.name" class="unified-thumbnail">
            <div v-else class="unified-file-icon" :class="getFileTypeClass(file.type)">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
              </svg>
              <span class="unified-file-extension">{{ getFileExtension(file.name) }}</span>
            </div>

            <!-- 🎯 状态覆盖层 -->
            <div v-if="uploadState.status === 'uploading'" class="thumbnail-overlay">
              <div class="upload-progress-overlay">
                <div class="progress-circle">
                  <svg class="progress-ring" width="40" height="40">
                    <circle class="progress-ring-circle" stroke="var(--color-primary)" stroke-width="3"
                      fill="transparent" r="16" cx="20" cy="20" :stroke-dasharray="`${100.48} ${100.48}`"
                      :stroke-dashoffset="`${100.48 - (uploadState.progress / 100) * 100.48}`"></circle>
                  </svg>
                  <span class="progress-percent">{{ Math.round(uploadState.progress) }}%</span>
                </div>
              </div>
            </div>

            <div v-else-if="uploadState.status === 'failed'" class="thumbnail-overlay error-overlay">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="15" y1="9" x2="9" y2="15"></line>
                <line x1="9" y1="9" x2="15" y2="15"></line>
              </svg>
            </div>

            <div v-else-if="uploadState.status === 'completed'" class="thumbnail-overlay success-overlay">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="20,6 9,17 4,12"></polyline>
              </svg>
            </div>
          </div>

          <!-- 🎯 统一的文件信息区域 -->
          <div class="unified-file-info">
            <h4 class="unified-file-name">{{ file.name }}</h4>
            <div class="unified-file-meta">
              <span class="file-size">{{ formatFileSize(file.size) }}</span>
              <span class="file-type">{{ file.type || 'Unknown type' }}</span>
            </div>

            <!-- 🎯 动态状态指示器 -->
            <div class="unified-status-indicator">
              <span v-if="uploadState.status === 'pending'" class="status-badge status-local">
                📁 Local file selected
              </span>
              <span v-else-if="uploadState.status === 'uploading'" class="status-badge status-uploading">
                📤 Uploading {{ Math.round(uploadState.progress) }}%
              </span>
              <span v-else-if="uploadState.status === 'completed'" class="status-badge status-success">
                ✅ URL Ready for Send
              </span>
              <span v-else-if="uploadState.status === 'failed'" class="status-badge status-error">
                ❌ Upload Failed
              </span>
            </div>

            <!-- 🎯 错误信息显示 -->
            <div v-if="uploadState.status === 'failed'" class="error-message-compact">
              {{ uploadState.error }}
            </div>

            <div class="file-size-limit">
              <span class="size-limit-text">📏 Max file size: 100MB</span>
            </div>
          </div>
        </div>

        <!-- 右侧：动态操作按钮 -->
        <div class="action-buttons-right">
          <!-- 🎯 根据状态显示不同按钮组合 -->

          <!-- 本地预览状态：显示上传按钮 -->
          <template v-if="uploadState.status === 'pending'">
            <button @click="startUpload" class="upload-btn">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                <polyline points="7,10 12,15 17,10"></polyline>
                <line x1="12" y1="15" x2="12" y2="3"></line>
              </svg>
              Upload to Remote
            </button>
            <button @click="handleReplaceFile" class="replace-btn">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"></path>
              </svg>
              Replace File
            </button>
            <button @click="handleRemoveFile" class="remove-btn">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3,6 5,6 21,6"></polyline>
                <path d="M19,6V20A2,2 0 0,1 17,20H7A2,2 0 0,1 5,20V6M8,6V4A2,2 0 0,1 10,4H14A2,2 0 0,1 16,4V6"></path>
              </svg>
              Remove
            </button>
          </template>

          <!-- 上传中状态：只显示取消按钮 -->
          <template v-else-if="uploadState.status === 'uploading'">
            <button @click="handleRemoveFile" class="cancel-btn">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="15" y1="9" x2="9" y2="15"></line>
                <line x1="9" y1="9" x2="15" y2="15"></line>
              </svg>
              Cancel Upload
            </button>
          </template>

          <!-- 上传成功状态：显示替换和移除按钮 -->
          <template v-else-if="uploadState.status === 'completed'">
            <button @click="handleReplaceFile" class="replace-btn">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"></path>
              </svg>
              Replace File
            </button>
            <button @click="handleRemoveFile" class="remove-btn">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3,6 5,6 21,6"></polyline>
                <path d="M19,6V20A2,2 0 0,1 17,20H7A2,2 0 0,1 5,20V6M8,6V4A2,2 0 0,1 10,4H14A2,2 0 0,1 16,4V6"></path>
              </svg>
              Remove
            </button>
          </template>

          <!-- 上传失败状态：显示重试、替换和移除按钮 -->
          <template v-else-if="uploadState.status === 'failed'">
            <button @click="retryUpload" class="retry-btn">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="23,4 23,10 17,10"></polyline>
                <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
              </svg>
              Retry Upload
            </button>
            <button @click="handleReplaceFile" class="replace-btn">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"></path>
              </svg>
              Replace File
            </button>
            <button @click="handleRemoveFile" class="remove-btn">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3,6 5,6 21,6"></polyline>
                <path d="M19,6V20A2,2 0 0,1 17,20H7A2,2 0 0,1 5,20V6M8,6V4A2,2 0 0,1 10,4H14A2,2 0 0,1 16,4V6"></path>
              </svg>
              Remove
            </button>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';

// 🎯 Props - 接收文件对象
const props = defineProps({
  file: {
    type: File,
    required: true
  }
});

// 🎯 Events - 与父组件通信
const emit = defineEmits([
  'file-uploaded',    // 上传成功
  'upload-error',     // 上传失败  
  'file-removed',     // 文件移除
  'trigger-upload'    // 触发文件选择
]);

// 🔄 上传状态管理
const uploadState = ref({
  status: 'pending',  // pending, uploading, completed, failed
  progress: 0,
  error: null,
  result: null,
  startTime: null
});

// 📁 文件预览URL（本地）
const localPreviewUrl = ref(null);

// 🔧 工具函数
const isImage = (file) => {
  return file.type.startsWith('image/');
};

const getFileExtension = (filename) => {
  const lastDot = filename.lastIndexOf('.');
  if (lastDot === -1) return '';
  return filename.substring(lastDot + 1).toUpperCase();
};

const getFileTypeClass = (type) => {
  if (type.startsWith('image/')) return 'file-image';
  if (type.startsWith('video/')) return 'file-video';
  if (type.startsWith('audio/')) return 'file-audio';
  if (type.includes('pdf')) return 'file-pdf';
  if (type.includes('text/') || type.includes('json') || type.includes('javascript')) return 'file-code';
  return 'file-document';
};

const formatFileSize = (bytes) => {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const getFilePreviewUrl = () => {
  if (uploadState.value.status === 'completed' && uploadState.value.result?.url) {
    return uploadState.value.result.url;
  }
  return localPreviewUrl.value;
};

// 🚀 上传到远端
const startUpload = async () => {
  if (!props.file || uploadState.value.status === 'uploading') {
    return;
  }

  console.log('📤 [FilePreview] Starting upload:', props.file.name);

  uploadState.value = {
    status: 'uploading',
    progress: 0,
    error: null,
    result: null,
    startTime: Date.now()
  };

  try {
    // 动态导入ChatService
    const { default: ChatService } = await import('@/services/ChatService.ts');

    // 调用上传API
    const uploadResult = await ChatService.uploadFile(props.file, (progress) => {
      uploadState.value.progress = progress;
    });

    console.log('✅ [FilePreview] Upload completed:', uploadResult);

    // 更新状态为成功
    uploadState.value = {
      status: 'completed',
      progress: 100,
      error: null,
      result: uploadResult,
      startTime: uploadState.value.startTime
    };

    // 🎯 通知父组件上传成功
    emit('file-uploaded', uploadResult);

  } catch (error) {
    console.error('❌ [FilePreview] Upload failed:', error);

    // 更新状态为失败
    uploadState.value = {
      status: 'failed',
      progress: 0,
      error: error.message,
      result: null,
      startTime: uploadState.value.startTime
    };

    // 🎯 通知父组件上传失败
    emit('upload-error', error);
  }
};

// 🔄 重试上传
const retryUpload = () => {
  console.log('🔄 [FilePreview] Retrying upload');
  startUpload();
};

// 🔄 替换文件 - 清空URL并触发新文件选择
const handleReplaceFile = () => {
  console.log('🔄 [FilePreview] Replace file requested - clearing URL and triggering file selection');

  // 重置上传状态
  uploadState.value = {
    status: 'pending',
    progress: 0,
    error: null,
    result: null,
    startTime: null
  };

  // 清理本地预览URL
  if (localPreviewUrl.value) {
    URL.revokeObjectURL(localPreviewUrl.value);
    localPreviewUrl.value = null;
  }

  // 通知父组件清空URL
  emit('file-removed');

  // 触发新文件选择
  setTimeout(() => {
    emit('trigger-upload');
  }, 100);
};

// 🗑️ 移除文件 - 清空URL和文件
const handleRemoveFile = () => {
  console.log('🗑️ [FilePreview] Remove file requested - clearing URL and file');

  // 重置上传状态
  uploadState.value = {
    status: 'pending',
    progress: 0,
    error: null,
    result: null,
    startTime: null
  };

  // 清理本地预览URL
  if (localPreviewUrl.value) {
    URL.revokeObjectURL(localPreviewUrl.value);
    localPreviewUrl.value = null;
  }

  // 通知父组件清空URL和移除文件
  emit('file-removed');
};

// 🔧 组件生命周期
onMounted(() => {
  // 为图片创建本地预览URL
  if (isImage(props.file)) {
    localPreviewUrl.value = URL.createObjectURL(props.file);
  }

  console.log('📁 [FilePreview] Component mounted for file:', props.file.name);
});

onUnmounted(() => {
  // 清理本地预览URL
  if (localPreviewUrl.value) {
    URL.revokeObjectURL(localPreviewUrl.value);
  }

  console.log('🧹 [FilePreview] Component unmounted, cleaned up resources');
});
</script>

<style scoped>
.file-preview-container {
  background: var(--color-background-soft);
  border-radius: 12px;
  padding: 20px;
  border: 2px dashed var(--color-border);
  transition: all 0.3s ease;
}

.file-preview-container:hover {
  border-color: var(--color-primary);
}

/* 🎯 统一的文件预览样式 */
.unified-file-preview {
  width: 100%;
}

.file-preview-horizontal {
  display: flex;
  align-items: center;
  gap: 16px;
  background: var(--color-background);
  border-radius: 8px;
  padding: 12px;
  border: 1px solid var(--color-border);
}

/* 🎯 统一的缩略图系统 */
.unified-thumbnail-container {
  position: relative;
  flex-shrink: 0;
}

.unified-thumbnail {
  width: 50px;
  height: 50px;
  object-fit: cover;
  border-radius: 6px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transition: transform 0.2s ease;
}

.unified-thumbnail:hover {
  transform: scale(1.05);
}

.unified-file-icon {
  width: 50px;
  height: 50px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 3px;
  border-radius: 6px;
  background: var(--color-background-muted);
  color: var(--color-text-muted);
  transition: transform 0.2s ease;
}

.unified-file-icon:hover {
  transform: scale(1.05);
}

.unified-file-extension {
  font-size: 10px;
  font-weight: bold;
  background: var(--color-primary-soft);
  color: var(--color-primary);
  padding: 1px 4px;
  border-radius: 2px;
  text-transform: uppercase;
}

/* 🎯 状态覆盖层 */
.thumbnail-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  backdrop-filter: blur(2px);
}

.upload-progress-overlay {
  background: rgba(255, 255, 255, 0.9);
  border-radius: 6px;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.progress-circle {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.progress-ring {
  transform: rotate(-90deg);
}

.progress-ring-circle {
  transition: stroke-dashoffset 0.3s ease;
}

.progress-percent {
  position: absolute;
  font-size: 10px;
  font-weight: bold;
  color: var(--color-primary);
}

.success-overlay {
  background: rgba(34, 197, 94, 0.9);
  color: white;
}

.error-overlay {
  background: rgba(239, 68, 68, 0.9);
  color: white;
}

/* 🎯 统一的文件信息 */
.unified-file-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  flex: 1;
  min-width: 0;
}

.unified-file-name {
  margin: 0 0 4px 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text);
  word-break: break-word;
  line-height: 1.3;
  text-align: center;
  max-width: 120px;
}

.unified-file-meta {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  font-size: 11px;
  color: var(--color-text-muted);
  margin-bottom: 6px;
}

/* 🎯 动态状态指示器 */
.unified-status-indicator {
  display: flex;
  justify-content: center;
  margin-bottom: 6px;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  border-radius: 12px;
  font-size: 10px;
  font-weight: 500;
  white-space: nowrap;
}

.status-local {
  background: var(--color-warning-soft);
  color: var(--color-warning);
  border: 1px solid var(--color-warning);
}

.status-uploading {
  background: var(--color-primary-soft);
  color: var(--color-primary);
  border: 1px solid var(--color-primary);
  animation: pulse 2s infinite;
}

.status-success {
  background: var(--color-success-soft);
  color: var(--color-success);
  border: 1px solid var(--color-success);
}

.status-error {
  background: var(--color-danger-soft);
  color: var(--color-danger);
  border: 1px solid var(--color-danger);
}

@keyframes pulse {

  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.7;
  }
}

/* 🎯 紧凑的错误信息 */
.error-message-compact {
  font-size: 10px;
  color: var(--color-danger);
  background: var(--color-danger-soft);
  padding: 4px 8px;
  border-radius: 6px;
  margin-bottom: 6px;
  text-align: center;
  max-width: 120px;
  word-break: break-word;
}

/* 🎯 取消上传按钮 */
.cancel-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 8px 12px;
  background: var(--color-danger-soft);
  color: var(--color-danger);
  border: 1px solid var(--color-danger);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 12px;
  white-space: nowrap;
  min-width: 110px;
  width: 110px;
}

.cancel-btn:hover {
  background: var(--color-danger);
  color: white;
  transform: translateY(-1px);
}

/* 📤 上传中状态 */
.uploading-state {
  text-align: center;
}

.upload-progress {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.progress-header h4 {
  margin: 0;
  color: var(--color-primary);
  font-size: 16px;
}

.progress-text {
  font-weight: bold;
  color: var(--color-primary);
}

.progress-bar {
  width: 100%;
  height: 8px;
  background: var(--color-background);
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--color-primary), var(--color-primary-dark));
  transition: width 0.3s ease;
  border-radius: 4px;
}

.upload-details {
  display: flex;
  justify-content: space-between;
  font-size: 14px;
  color: var(--color-text-muted);
}

/* ✅ 上传成功状态 - 简化版 */
.upload-success-simple {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.upload-success {
  text-align: center;
}

.success-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-bottom: 16px;
  color: var(--color-success);
}

.success-indicator h4 {
  margin: 0;
  font-size: 16px;
}

/* URL成功信号样式 */
.url-success-indicator {
  margin-top: 6px;
  display: flex;
  justify-content: center;
}

.url-success-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 10px;
  color: var(--color-success);
  background: var(--color-success-soft);
  padding: 3px 8px;
  border-radius: 12px;
  border: 1px solid var(--color-success);
  white-space: nowrap;
  font-weight: 500;
}

/* 上传成功状态的左右布局 */
.uploaded-file-horizontal {
  display: flex;
  align-items: center;
  gap: 16px;
  background: var(--color-background);
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 16px;
  border: 1px solid var(--color-border);
}

.uploaded-thumbnail-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.uploaded-file-info {
  display: flex;
  gap: 16px;
  align-items: center;
  background: var(--color-background);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.file-preview-thumb {
  flex-shrink: 0;
}

.thumbnail {
  width: 80px;
  height: 80px;
  object-fit: cover;
  border-radius: 6px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.thumbnail-compact {
  width: 50px;
  height: 50px;
  object-fit: cover;
  border-radius: 6px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transition: transform 0.2s ease;
}

.thumbnail-compact:hover {
  transform: scale(1.05);
}

.file-icon {
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  background: var(--color-background-muted);
  color: var(--color-text-muted);
  flex-direction: column;
  gap: 4px;
}

.file-icon-uploaded {
  width: 50px;
  height: 50px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  background: var(--color-background-muted);
  color: var(--color-text-muted);
  flex-direction: column;
  gap: 3px;
}

.file-extension {
  font-size: 12px;
  font-weight: bold;
  background: var(--color-primary-soft);
  color: var(--color-primary);
  padding: 2px 6px;
  border-radius: 3px;
  text-transform: uppercase;
}

.file-details {
  flex: 1;
  text-align: left;
}

.file-details .file-name {
  margin: 0 0 8px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
  word-break: break-word;
}

.file-details-compact {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  flex: 1;
  min-width: 0;
}

.file-name-uploaded {
  margin: 0 0 4px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
  word-break: break-word;
  line-height: 1.3;
}

.file-meta {
  display: flex;
  gap: 16px;
  margin-bottom: 12px;
  font-size: 14px;
  color: var(--color-text-muted);
}

.file-meta-uploaded {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: 8px;
  font-size: 12px;
  color: var(--color-text-muted);
}

.remote-url {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.remote-url label {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-muted);
}

.url-link {
  color: var(--color-primary);
  text-decoration: none;
  word-break: break-all;
  font-family: monospace;
  font-size: 13px;
  padding: 4px 6px;
  background: var(--color-background-soft);
  border-radius: 4px;
  border: 1px solid var(--color-border);
  transition: all 0.2s ease;
}

.url-link:hover {
  background: var(--color-primary-soft);
  border-color: var(--color-primary);
  text-decoration: underline;
}

/* ❌ 上传失败状态 */
.upload-error {
  text-align: center;
}

.error-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-bottom: 16px;
  color: var(--color-danger);
}

.error-indicator h4 {
  margin: 0;
  font-size: 16px;
}

.error-details {
  background: var(--color-danger-soft);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.error-message {
  margin: 0 0 16px 0;
  color: var(--color-danger);
  font-size: 14px;
}

.error-actions {
  display: flex;
  justify-content: center;
  gap: 12px;
  flex-wrap: wrap;
}

/* 📁 本地文件预览状态 - 左右布局 */
.local-file-preview {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.file-preview-horizontal {
  display: flex;
  align-items: center;
  gap: 16px;
  background: var(--color-background);
  border-radius: 8px;
  padding: 12px;
  border: 1px solid var(--color-border);
}

/* 左侧：缩略图和信息区域（垂直布局） */
.file-thumbnail-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
  /* 允许收缩 */
}

.file-thumbnail-container {
  flex-shrink: 0;
}

.file-thumbnail-compact {
  width: 50px;
  height: 50px;
  object-fit: cover;
  border-radius: 6px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transition: transform 0.2s ease;
}

.file-thumbnail-compact:hover {
  transform: scale(1.05);
}

.file-icon-compact {
  width: 50px;
  height: 50px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 3px;
  border-radius: 6px;
  background: var(--color-background-muted);
  color: var(--color-text-muted);
}

.file-extension-compact {
  font-size: 10px;
  font-weight: bold;
  background: var(--color-primary-soft);
  color: var(--color-primary);
  padding: 1px 4px;
  border-radius: 2px;
  text-transform: uppercase;
}

.file-info-compact {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  flex: 1;
  min-width: 0;
  /* 允许收缩 */
}

.file-name-compact {
  margin: 0 0 4px 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text);
  word-break: break-word;
  line-height: 1.3;
  text-align: center;
  max-width: 120px;
}

.file-meta-compact {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  font-size: 11px;
  color: var(--color-text-muted);
}

.file-status {
  display: flex;
  justify-content: center;
  margin-top: 8px;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
}

.status-local {
  background: var(--color-warning-soft);
  color: var(--color-warning);
}

/* 文件大小限制说明 */
.file-size-limit {
  margin-top: 6px;
  display: flex;
  justify-content: center;
}

.size-limit-text {
  font-size: 10px;
  color: var(--color-text-muted);
  background: var(--color-background-soft);
  padding: 3px 8px;
  border-radius: 12px;
  border: 1px solid var(--color-border);
  white-space: nowrap;
}

/* 🎯 操作按钮 - 强制水平排列 */
.action-buttons {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 8px;
  flex-wrap: nowrap;
  /* 🔧 强制不换行，保持水平排列 */
}

/* 🎯 右侧按钮区域 - 垂直排列节省空间 */
.action-buttons-right {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  min-width: 120px;
  /* 确保按钮有足够宽度 */
}

.upload-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 8px 12px;
  background: var(--color-primary);
  color: white;
  border: none;
  border-radius: 6px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  font-size: 12px;
  /* 🔧 与其他按钮一致的字体大小 */
  white-space: nowrap;
  min-width: 110px;
  width: 110px;
  /* 🔧 与其他按钮一致的宽度 */
}

.upload-btn:hover:not(:disabled) {
  background: var(--color-primary-dark);
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
}

.upload-btn:disabled {
  background: var(--color-text-muted);
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.replace-btn,
.remove-btn,
.retry-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  /* 🔧 缩小图标和文字间距 */
  padding: 8px 12px;
  /* 🔧 调整为更紧凑的padding */
  background: var(--color-background);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 12px;
  /* 🔧 稍小的字体 */
  white-space: nowrap;
  /* 🔧 防止文字换行 */
  min-width: 110px;
  /* 🔧 固定最小宽度 */
  width: 110px;
  /* 🔧 固定宽度使按钮一致 */
}

.replace-btn:hover,
.retry-btn:hover {
  background: var(--color-background-muted);
  border-color: var(--color-primary);
  color: var(--color-primary);
  transform: translateY(-1px);
}

.remove-btn:hover {
  background: var(--color-danger-soft);
  border-color: var(--color-danger);
  color: var(--color-danger);
  transform: translateY(-1px);
}

/* 文件类型图标样式 */
.file-icon.file-image {
  background: #eff6ff;
  color: #3b82f6;
}

.file-icon.file-video {
  background: #fef3e2;
  color: #f59e0b;
}

.file-icon.file-audio {
  background: #f0fdf4;
  color: #10b981;
}

.file-icon.file-pdf {
  background: #fef2f2;
  color: #ef4444;
}

.file-icon.file-code {
  background: #f8fafc;
  color: #64748b;
}

.file-icon.file-document {
  background: #fafafa;
  color: #6b7280;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .file-preview-container {
    padding: 16px;
  }

  .uploaded-file-info {
    flex-direction: column;
    text-align: center;
  }

  .uploaded-file-horizontal {
    flex-direction: column;
    gap: 12px;
    padding: 12px;
  }

  .uploaded-thumbnail-section {
    justify-content: center;
  }

  .thumbnail-compact {
    width: 50px;
    height: 50px;
  }

  .file-icon-uploaded {
    width: 50px;
    height: 50px;
  }

  .file-name-uploaded {
    font-size: 13px;
  }

  .file-meta-uploaded {
    font-size: 11px;
  }

  /* 🎯 移动端也保持水平排列，但按钮更紧凑 */
  .action-buttons {
    justify-content: center;
    gap: 6px;
    flex-wrap: nowrap;
  }

  .action-buttons button {
    padding: 8px 12px;
    /* 更紧凑的padding */
    font-size: 12px;
    /* 稍小的字体 */
    flex: 1;
    /* 等宽分布 */
    min-width: 0;
    /* 允许收缩 */
  }

  .upload-btn {
    padding: 10px 14px;
    /* Upload按钮稍大一点 */
  }

  /* 🎯 移动端左右布局调整 */
  .file-preview-horizontal {
    flex-direction: column;
    gap: 12px;
    padding: 12px;
  }

  .file-thumbnail-section {
    justify-content: center;
  }

  .action-buttons-right {
    flex-direction: row;
    justify-content: center;
    min-width: auto;
    width: 100%;
    gap: 6px;
  }

  .action-buttons-right button {
    flex: 1;
    padding: 8px 10px;
    font-size: 12px;
    min-width: 0;
  }

  .file-thumbnail-compact {
    width: 50px;
    height: 50px;
  }

  .file-icon-compact {
    width: 50px;
    height: 50px;
  }

  .file-name-compact {
    font-size: 13px;
  }

  .file-meta-compact {
    font-size: 11px;
  }
}
</style>