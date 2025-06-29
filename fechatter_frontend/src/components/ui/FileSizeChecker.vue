<template>
  <div v-if="showChecker" class="file-size-checker">
    <!-- File size limit info -->
    <div class="size-limit-info">
      <span class="limit-icon">📏</span>
      <span class="limit-text">Max file size: {{ maxSizeFormatted }}</span>
    </div>

    <!-- File validation results -->
    <div v-if="validationResults.length > 0" class="validation-results">
      <div
        v-for="result in validationResults"
        :key="result.fileName"
        class="file-result"
        :class="{
          'result-success': result.isValid,
          'result-warning': result.isNearLimit,
          'result-error': !result.isValid
        }"
      >
        <div class="file-info">
          <span class="file-name">{{ result.fileName }}</span>
          <span class="file-size">{{ result.fileSize }}</span>
        </div>
        
        <div class="file-status">
          <span v-if="result.isValid && !result.isNearLimit" class="status-success">✅</span>
          <span v-else-if="result.isNearLimit" class="status-warning">⚠️</span>
          <span v-else class="status-error">❌</span>
        </div>

        <!-- Error messages -->
        <div v-if="result.errors.length > 0" class="error-messages">
          <div v-for="error in result.errors" :key="error" class="error-message">
            {{ error }}
          </div>
        </div>

        <!-- Warning messages -->
        <div v-if="result.warnings.length > 0" class="warning-messages">
          <div v-for="warning in result.warnings" :key="warning" class="warning-message">
            {{ warning }}
          </div>
        </div>
      </div>
    </div>

    <!-- Action buttons -->
    <div v-if="hasInvalidFiles" class="action-buttons">
      <button @click="removeInvalidFiles" class="btn-remove-invalid">
        🗑️ Remove Invalid Files
      </button>
      <button @click="compressLargeImages" class="btn-compress" :disabled="!hasCompressibleImages">
        🗜️ Try Compress Images
      </button>
    </div>

    <!-- Compression progress -->
    <div v-if="isCompressing" class="compression-progress">
      <div class="progress-bar">
        <div class="progress-fill" :style="{ width: compressionProgress + '%' }"></div>
      </div>
      <span class="progress-text">Compressing images... {{ compressionProgress }}%</span>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue';
import { useFileUploadStore } from '@/stores/fileUploadStore';
import imageCompression from 'browser-image-compression';

const props = defineProps({
  files: {
    type: Array,
    default: () => []
  },
  autoShow: {
    type: Boolean,
    default: true
  }
});

const emit = defineEmits(['files-validated', 'files-compressed', 'invalid-files-removed']);

const fileUploadStore = useFileUploadStore();
const validationResults = ref([]);
const isCompressing = ref(false);
const compressionProgress = ref(0);

// Computed properties
const showChecker = computed(() => {
  return props.autoShow && (props.files.length > 0 || validationResults.value.length > 0);
});

const maxSizeFormatted = computed(() => {
  return fileUploadStore.getMaxFileSizeFormatted();
});

const hasInvalidFiles = computed(() => {
  return validationResults.value.some(result => !result.isValid);
});

const hasCompressibleImages = computed(() => {
  return validationResults.value.some(result => 
    !result.isValid && 
    result.file.type.startsWith('image/') &&
    result.file.size > fileUploadStore.getMaxFileSize()
  );
});

// File size formatting utility
const formatFileSize = (bytes) => {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

// Validate files function
const validateFiles = (files) => {
  if (!files || files.length === 0) {
    validationResults.value = [];
    return;
  }

  const results = [];
  
  for (const file of files) {
    const validation = validateFile(file);
    results.push({
      file,
      fileName: file.name,
      fileSize: formatFileSize(file.size),
      isValid: validation.isValid,
      isNearLimit: validation.isNearLimit,
      errors: validation.errors,
      warnings: validation.warnings
    });
  }

  validationResults.value = results;
  
  // Emit validation results
  emit('files-validated', {
    valid: results.filter(r => r.isValid),
    invalid: results.filter(r => !r.isValid),
    total: results.length
  });
};

// Validate single file
const validateFile = (file) => {
  const maxSize = fileUploadStore.getMaxFileSize();
  const errors = [];
  const warnings = [];

  // Size check
  if (file.size > maxSize) {
    const fileSize = formatFileSize(file.size);
    const maxSizeFormatted = formatFileSize(maxSize);
    errors.push(`File exceeds 100MB limit (${fileSize} > ${maxSizeFormatted})`);
  } else if (file.size > maxSize * 0.9) {
    warnings.push('File is close to 100MB limit');
  }

  // Type check
  const allowedTypes = fileUploadStore.uploadConfig.allowedTypes;
  if (!allowedTypes.includes(file.type)) {
    errors.push(`File type "${file.type}" not supported`);
  }

  // Empty file check
  if (file.size === 0) {
    errors.push('File is empty');
  }

  return {
    isValid: errors.length === 0,
    isNearLimit: file.size > maxSize * 0.9 && file.size <= maxSize,
    errors,
    warnings
  };
};

// Remove invalid files
const removeInvalidFiles = () => {
  const validFiles = validationResults.value
    .filter(result => result.isValid)
    .map(result => result.file);
  
  const removedCount = props.files.length - validFiles.length;
  
  emit('invalid-files-removed', {
    validFiles,
    removedCount
  });
  
  // Update validation results
  validateFiles(validFiles);
};

// Compress large images
const compressLargeImages = async () => {
  if (!hasCompressibleImages.value) return;

  isCompressing.value = true;
  compressionProgress.value = 0;

  const compressedFiles = [];
  const imagesToCompress = validationResults.value
    .filter(result => !result.isValid && result.file.type.startsWith('image/'))
    .map(result => result.file);

  try {
    for (let i = 0; i < imagesToCompress.length; i++) {
      const file = imagesToCompress[i];
      
      try {
        const compressedFile = await imageCompression(file, {
          maxSizeMB: 1.8,
          maxWidthOrHeight: 1920,
          useWebWorker: true,
          fileType: 'image/jpeg',
          initialQuality: 0.8
        });

        // Create new File object with original name
        const finalFile = new File([compressedFile], file.name, {
          type: compressedFile.type,
          lastModified: Date.now()
        });

        compressedFiles.push(finalFile);
      } catch (error) {
        console.warn(`Failed to compress ${file.name}:`, error);
        // Keep original file if compression fails
        compressedFiles.push(file);
      }

      compressionProgress.value = Math.round(((i + 1) / imagesToCompress.length) * 100);
    }

    // Combine compressed images with other valid files
    const otherFiles = validationResults.value
      .filter(result => result.isValid || !result.file.type.startsWith('image/'))
      .map(result => result.file);

    const allFiles = [...otherFiles, ...compressedFiles];

    emit('files-compressed', {
      originalFiles: props.files,
      compressedFiles: allFiles,
      compressionResults: {
        processed: imagesToCompress.length,
        successful: compressedFiles.length
      }
    });

    // Re-validate compressed files
    validateFiles(allFiles);

  } finally {
    isCompressing.value = false;
    compressionProgress.value = 0;
  }
};

// Watch for file changes
watch(() => props.files, (newFiles) => {
  validateFiles(newFiles);
}, { immediate: true });

// Expose validation function
defineExpose({
  validateFiles,
  removeInvalidFiles,
  compressLargeImages
});
</script>

<style scoped>
.file-size-checker {
  background: #f8f9fa;
  border: 1px solid #e9ecef;
  border-radius: 8px;
  padding: 12px;
  margin: 8px 0;
  font-size: 14px;
}

.size-limit-info {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  padding: 8px 12px;
  background: #e3f2fd;
  border-radius: 6px;
  color: #1976d2;
}

.limit-icon {
  font-size: 16px;
}

.limit-text {
  font-weight: 500;
}

.validation-results {
  margin-bottom: 12px;
}

.file-result {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 8px 12px;
  margin-bottom: 8px;
  border-radius: 6px;
  transition: all 0.2s;
}

.result-success {
  background: #e8f5e8;
  border-left: 4px solid #4caf50;
}

.result-warning {
  background: #fff3cd;
  border-left: 4px solid #ff9800;
}

.result-error {
  background: #ffebee;
  border-left: 4px solid #f44336;
}

.file-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.file-name {
  font-weight: 500;
  color: #333;
}

.file-size {
  font-size: 12px;
  color: #666;
}

.file-status {
  margin-left: 12px;
}

.error-messages,
.warning-messages {
  margin-top: 8px;
  flex-basis: 100%;
}

.error-message {
  color: #d32f2f;
  font-size: 12px;
  margin-bottom: 4px;
}

.warning-message {
  color: #f57c00;
  font-size: 12px;
  margin-bottom: 4px;
}

.action-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.btn-remove-invalid,
.btn-compress {
  padding: 6px 12px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.2s;
}

.btn-remove-invalid {
  background: #f44336;
  color: white;
}

.btn-remove-invalid:hover {
  background: #d32f2f;
}

.btn-compress {
  background: #2196f3;
  color: white;
}

.btn-compress:hover:not(:disabled) {
  background: #1976d2;
}

.btn-compress:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.compression-progress {
  margin-top: 12px;
}

.progress-bar {
  width: 100%;
  height: 6px;
  background: #e0e0e0;
  border-radius: 3px;
  overflow: hidden;
  margin-bottom: 6px;
}

.progress-fill {
  height: 100%;
  background: #2196f3;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 12px;
  color: #666;
}
</style>
