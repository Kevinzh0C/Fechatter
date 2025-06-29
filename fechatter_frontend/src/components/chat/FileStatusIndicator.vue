<template>
  <div class="file-status-indicator" :class="{
    'status-uploading': file.upload_status === 'uploading',
    'status-uploaded': file.upload_status === 'uploaded',
    'status-failed': file.upload_status === 'failed',
    'message-sending': messageStatus === 'sending',
    'message-sent': messageStatus === 'sent',
    'message-delivered': messageStatus === 'delivered',
    'message-failed': messageStatus === 'failed'
  }">
    <!-- Upload Progress Bar for Files -->
    <div v-if="file.upload_status === 'uploading'" class="upload-progress-container">
      <div class="upload-progress-bar">
        <div class="upload-progress-fill" :style="{ width: `${file.upload_progress || 0}%` }"></div>
      </div>
      <div class="upload-progress-text">
        📤 上传中 {{ file.upload_progress || 0 }}%
      </div>
    </div>

    <!-- File Upload Success -->
    <div v-else-if="file.upload_status === 'uploaded'" class="upload-success">
      <div class="upload-success-icon"></div>
      <div class="upload-success-text">文件已上传</div>
    </div>

    <!-- File Upload Failed -->
    <div v-else-if="file.upload_status === 'failed'" class="upload-failed">
      <div class="upload-failed-icon">❌</div>
      <div class="upload-failed-text">
        上传失败
        <div v-if="file.upload_error" class="upload-error-detail">
          {{ file.upload_error }}
        </div>
      </div>
    </div>

    <!-- Message Status (for uploaded files) -->
    <div v-if="file.upload_status === 'uploaded' && messageStatus" class="message-status-indicator">
      <div class="message-status-icon">
        <span v-if="messageStatus === 'sending'">📨</span>
        <span v-else-if="messageStatus === 'sent'"></span>
        <span v-else-if="messageStatus === 'delivered'"></span>
        <span v-else-if="messageStatus === 'failed'">❌</span>
      </div>
      <div class="message-status-text">
        {{ getMessageStatusText(messageStatus) }}
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  file: {
    type: Object,
    required: true
  },
  messageStatus: {
    type: String,
    default: null
  }
});

const getMessageStatusText = (status) => {
  const statusMap = {
    'file_uploading': '📤 正在上传文件...',
    'file_uploaded': '文件已上传',
    'sending': 'EVENT: 正在发送...',
    'sent': '已发送',
    'delivered': '已送达',
    'failed': 'ERROR: 发送失败'
  };

  return statusMap[status] || status;
};
</script>

<style scoped>
.file-status-indicator {
  margin-top: 4px;
  font-size: 12px;
  min-height: 20px;
  display: flex;
  align-items: center;
  gap: 6px;
}

/* Upload Progress */
.upload-progress-container {
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 100%;
}

.upload-progress-bar {
  width: 100%;
  height: 4px;
  background-color: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
  overflow: hidden;
}

.upload-progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #007bff, #0056b3);
  transition: width 0.3s ease;
}

.upload-progress-text {
  color: #007bff;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 4px;
}

/* Upload Success */
.upload-success {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #28a745;
}

.upload-success-icon {
  font-size: 14px;
}

.upload-success-text {
  font-weight: 500;
}

/* Upload Failed */
.upload-failed {
  display: flex;
  flex-direction: column;
  gap: 2px;
  color: #dc3545;
}

.upload-failed-icon {
  font-size: 14px;
}

.upload-failed-text {
  font-weight: 500;
}

.upload-error-detail {
  font-size: 11px;
  color: #6c757d;
  margin-top: 2px;
}

/* Message Status */
.message-status-indicator {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 8px;
  padding-left: 8px;
  border-left: 1px solid rgba(255, 255, 255, 0.1);
}

.message-status-icon {
  font-size: 12px;
}

.message-status-text {
  font-size: 11px;
  color: #6c757d;
}

/* State-based styling */
.status-uploading {
  opacity: 0.9;
}

.status-uploaded.message-sending {
  opacity: 0.95;
}

.status-uploaded.message-sent {
  opacity: 1;
}

.status-uploaded.message-delivered {
  opacity: 1;
}

.status-failed,
.message-failed {
  opacity: 0.8;
  background-color: rgba(220, 53, 69, 0.1);
  border-radius: 4px;
  padding: 4px 8px;
}

/* Animations */
@keyframes pulse {

  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.7;
  }
}

.status-uploading .upload-progress-text {
  animation: pulse 1.5s infinite;
}

.message-sending .message-status-text {
  animation: pulse 1.5s infinite;
}

/* Responsive design */
@media (max-width: 768px) {
  .file-status-indicator {
    font-size: 11px;
  }

  .upload-progress-bar {
    height: 3px;
  }

  .message-status-indicator {
    margin-left: 4px;
    padding-left: 4px;
  }
}
</style>