import { defineStore } from 'pinia';
import { ref, reactive } from 'vue';
import imageCompression from 'browser-image-compression';

// Production-grade file upload store with comprehensive error handling
export const useFileUploadStore = defineStore('fileUpload', () => {
  // ================================
  // Core State Management
  // ================================
  const pendingFiles = ref([]);
  const isUploading = ref(false);
  const globalError = ref(null);

  // Upload configuration
  const uploadConfig = {
    maxFileSize: 100 * 1024 * 1024, // FILE UPLOAD FIX: 100MB max file size (matching nginx limit)
    maxFiles: 10, // Max 10 files at once
    allowedTypes: [
      'image/jpeg', 'image/png', 'image/gif', 'image/webp', 'image/svg+xml',
      'video/mp4', 'video/avi', 'video/mov', 'video/wmv', 'video/webm',
      'audio/mp3', 'audio/wav', 'audio/ogg', 'audio/m4a', 'audio/aac',
      'application/pdf', 'application/msword', 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
      'text/plain', 'application/json', 'text/yaml', 'application/x-yaml'
    ],
    compressionOptions: {
      maxSizeMB: 95, // Slightly less than 100MB to account for compression variations
      maxWidthOrHeight: 1920,
      useWebWorker: true,
      fileType: 'image/jpeg',
      initialQuality: 0.8
    }
  };

  // ================================
  // Enhanced File Size Formatting Utility
  // ================================
  const formatFileSize = (bytes) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  // ================================
  // Getters
  // ================================
  const hasPendingFiles = () => pendingFiles.value.length > 0;
  const getTotalSize = () => pendingFiles.value.reduce((total, file) => total + file.size, 0);
  const getFilesByStatus = (status) => pendingFiles.value.filter(f => f.status === status);
  const hasReadyFiles = () => pendingFiles.value.some(f => f.status === 'pending_upload' || f.status === 'completed');
  const getMaxFileSize = () => uploadConfig.maxFileSize;
  const getMaxFileSizeFormatted = () => formatFileSize(uploadConfig.maxFileSize);

  // ================================
  // Enhanced File Validation with User-Friendly Messages
  // ================================
  const validateFile = (file) => {
    const errors = [];
    const warnings = [];

    // Check file size with detailed messaging
    if (file.size > uploadConfig.maxFileSize) {
      const fileSize = formatFileSize(file.size);
      const maxSize = formatFileSize(uploadConfig.maxFileSize);
      errors.push(`File "${file.name}" (${fileSize}) exceeds the 100MB limit. Maximum allowed size is ${maxSize}.`);
    } else if (file.size > uploadConfig.maxFileSize * 0.9) {
      // Warning for files close to limit
      const fileSize = formatFileSize(file.size);
      warnings.push(`File "${file.name}" (${fileSize}) is close to the 100MB limit. Consider compressing if upload fails.`);
    }

    // Check file type with better error message
    if (!uploadConfig.allowedTypes.includes(file.type)) {
      errors.push(`File type "${file.type}" is not supported. Please use images, videos, audio, PDFs, or documents.`);
    }

    // Check total files limit
    if (pendingFiles.value.length >= uploadConfig.maxFiles) {
      errors.push(`Maximum ${uploadConfig.maxFiles} files allowed. Please remove some files before adding more.`);
    }

    // Check for empty files
    if (file.size === 0) {
      errors.push(`File "${file.name}" is empty and cannot be uploaded.`);
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
      fileSize: formatFileSize(file.size),
      isNearLimit: file.size > uploadConfig.maxFileSize * 0.9 && file.size <= uploadConfig.maxFileSize
    };
  };

  // ================================
  // File Processing
  // ================================
  const createFileEntry = (file) => {
    const fileEntry = reactive({
      id: `file-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      rawFile: file,
      name: file.name,
      size: file.size,
      type: file.type,
      preview: null,
      progress: 0,
      error: null,
      status: 'pending', // pending, compressing, pending_upload, uploading, completed, error
      uploadedFile: null,
      createdAt: new Date().toISOString(),
      retryCount: 0,
      maxRetries: 3
    });

    // Generate preview URL for images
    if (file.type.startsWith('image/')) {
      try {
        fileEntry.preview = URL.createObjectURL(file);
      } catch (error) {
        if (import.meta.env.DEV) {
          console.warn('Failed to create preview URL:', error);
        }
      }
    }

    return fileEntry;
  };

  const compressImageFile = async (fileEntry) => {
    if (!fileEntry.type.startsWith('image/')) {
      return fileEntry.rawFile;
    }

    try {
      fileEntry.status = 'compressing';
      if (import.meta.env.DEV) {
        console.log(`🗜️ Compressing ${fileEntry.name}...`);
      }

      const compressedFile = await imageCompression(fileEntry.rawFile, uploadConfig.compressionOptions);

      // Update file entry with compressed data
      const oldPreview = fileEntry.preview;
      fileEntry.rawFile = compressedFile;
      fileEntry.name = compressedFile.name || fileEntry.name;
      fileEntry.size = compressedFile.size;
      fileEntry.type = compressedFile.type;

      // Clean up old preview and create new one
      if (oldPreview) {
        URL.revokeObjectURL(oldPreview);
      }
      fileEntry.preview = URL.createObjectURL(compressedFile);

      if (import.meta.env.DEV) {
        console.log(`Compression successful: ${fileEntry.name} (${Math.round(fileEntry.size / 1024)}KB)`);
      }
      return compressedFile;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error(`ERROR: Compression failed for ${fileEntry.name}:`, error);
      }
      fileEntry.status = 'error';
      fileEntry.error = 'Compression failed';
      throw error;
    }
  };

  // ================================
  // Core Actions
  // ================================

  /**
   * Add files with comprehensive validation and processing
   * @param {File[]} files - Array of File objects
   * @returns {Promise<{success: number, failed: number, warnings: string[]}>}
   */
  const addFiles = async (files) => {
    if (!files || files.length === 0) {
      return { success: 0, failed: 0, warnings: [] };
    }

    globalError.value = null;
    const newFiles = [];
    const errors = [];
    const allWarnings = [];

    // Process each file
    for (const file of files) {
      const validation = validateFile(file);

      if (validation.isValid) {
        const fileEntry = createFileEntry(file);
        newFiles.push(fileEntry);
        pendingFiles.value.push(fileEntry);

        // Collect warnings for valid files
        if (validation.warnings.length > 0) {
          allWarnings.push(...validation.warnings);
        }
      } else {
        errors.push({
          fileName: file.name,
          fileSize: validation.fileSize,
          errors: validation.errors
        });
      }
    }

    // ENHANCED: Better error and warning reporting
    if (errors.length > 0) {
      const errorMessages = errors.map(e =>
        `ERROR: ${e.fileName} (${e.fileSize}): ${e.errors.join('; ')}`
      ).join('\n');

      globalError.value = errorMessages;

      if (import.meta.env.DEV) {
        console.warn('File validation errors:', errorMessages);
      }

      // USER INTERACTION: Show user-friendly error notifications
      if (typeof window !== 'undefined' && window.showFileUploadError) {
        window.showFileUploadError(errorMessages, errors.length);
      }
    }

    // ENHANCED: Show warnings to user
    if (allWarnings.length > 0 && typeof window !== 'undefined' && window.showFileUploadWarning) {
      window.showFileUploadWarning(allWarnings.join('\n'), allWarnings.length);
    }

    // Process valid files asynchronously
    const processingPromises = newFiles.map(async (fileEntry) => {
      try {
        await compressImageFile(fileEntry);
        fileEntry.status = 'pending_upload';
      } catch (error) {
        fileEntry.status = 'error';
        fileEntry.error = error.message || 'Processing failed';
      }
    });

    // Wait for all processing to complete
    await Promise.allSettled(processingPromises);

    const result = {
      success: newFiles.length,
      failed: errors.length,
      warnings: allWarnings
    };

    if (import.meta.env.DEV) {
      console.log(`📁 File upload result:`, result);
    }

    return result;
  };

  /**
   * Remove a file from pending list with cleanup
   * @param {string} fileId - File ID to remove
   */
  const removeFile = (fileId) => {
    const index = pendingFiles.value.findIndex(f => f.id === fileId);
    if (index !== -1) {
      const file = pendingFiles.value[index];

      // Clean up preview URL to prevent memory leaks
      if (file.preview) {
        URL.revokeObjectURL(file.preview);
      }

      pendingFiles.value.splice(index, 1);
      if (import.meta.env.DEV) {
        console.log(`🗑️ Removed file: ${file.name}`);
      }
    }
  };

  /**
   * Clear all pending files with proper cleanup
   */
  const clearAll = () => {
    // Clean up all preview URLs
    pendingFiles.value.forEach(file => {
      if (file.preview) {
        URL.revokeObjectURL(file.preview);
      }
    });

    pendingFiles.value = [];
    globalError.value = null;
    isUploading.value = false;
    if (import.meta.env.DEV) {
      console.log('🧹 Cleared all files');
    }
  };

  /**
   * Retry failed file processing
   * @param {string} fileId - File ID to retry
   */
  const retryFile = async (fileId) => {
    const file = pendingFiles.value.find(f => f.id === fileId);
    if (!file || file.retryCount >= file.maxRetries) {
      return false;
    }

    file.retryCount++;
    file.error = null;

    try {
      await compressImageFile(file);
      file.status = 'pending_upload';
      return true;
    } catch (error) {
      file.status = 'error';
      file.error = `Retry ${file.retryCount}/${file.maxRetries} failed: ${error.message}`;
      return false;
    }
  };

  /**
   * Upload all ready files to server - Enhanced with better error handling
   * @returns {Promise<string[]>} Array of uploaded file URLs
   */
  const uploadAll = async () => {
    const readyFiles = getFilesByStatus('pending_upload');
    if (readyFiles.length === 0) {
      if (import.meta.env.DEV) {
        console.warn('No files ready for upload');
      }
      return [];
    }

    if (isUploading.value) {
      throw new Error('Upload already in progress');
    }

    isUploading.value = true;
    globalError.value = null;
    const uploadedUrls = [];
    const failedUploads = [];

    try {
      if (import.meta.env.DEV) {
        console.log(`📤 Starting upload of ${readyFiles.length} files...`);
      }

      // Enhanced Upload Notification
      if (typeof window !== 'undefined' && window.showFileUploadNotification) {
        window.showFileUploadNotification(`Uploading ${readyFiles.length} file(s)...`, 'info');
      }

      // Upload files in parallel with controlled concurrency
      const uploadPromises = readyFiles.map(async (file, index) => {
        try {
          file.status = 'uploading';
          file.progress = 0;

          // 动态导入ChatService以避免循环依赖
          const { default: ChatService } = await import('../services/ChatService.ts');

          // Upload with progress tracking
          const uploadedFile = await ChatService.uploadFile(file.rawFile, (progress) => {
            file.progress = Math.min(progress, 99); // Keep at 99% until complete
          });

          // Mark as completed
          file.status = 'completed';
          file.progress = 100;
          file.uploadedFile = uploadedFile;
          file.error = null;

          // CRITICAL FIX: Map file_url to url for consistent interface
          const fileUrl = uploadedFile.file_url || uploadedFile.url;
          uploadedUrls.push(fileUrl);
          if (import.meta.env.DEV) {
            console.log(`Uploaded: ${file.name} -> ${fileUrl}`);
          }

          // Individual file success notification
          if (typeof window !== 'undefined' && window.showFileUploadNotification) {
            window.showFileUploadNotification(`${file.name} uploaded successfully`, 'success');
          }

        } catch (error) {
          file.status = 'error';
          file.progress = 0;
          failedUploads.push({ file, error });

          // Enhanced error handling with specific error types
          let errorMessage = 'Upload failed';
          let errorType = 'error';

          if (error.code === 'NETWORK_ERROR') {
            errorMessage = `ERROR: ${file.name}: Server connection failed. Please check your internet connection.`;
            errorType = 'network';
            file.error = 'Network connection failed. Server may be down.';
          } else if (error.message?.includes('exceeds')) {
            errorMessage = `ERROR: ${file.name}: File too large (max 100MB)`;
            errorType = 'size';
            file.error = 'File size exceeds 100MB limit';
          } else if (error.message?.includes('type')) {
            errorMessage = `ERROR: ${file.name}: File type not supported`;
            errorType = 'type';
            file.error = 'File type not supported';
          } else {
            errorMessage = `ERROR: ${file.name}: ${error.message || 'Unknown error'}`;
            file.error = error.message || 'Upload failed';
          }

          // User-friendly error notifications
          if (typeof window !== 'undefined' && window.showFileUploadNotification) {
            window.showFileUploadNotification(errorMessage, errorType);
          }

          if (import.meta.env.DEV) {
            console.error(`ERROR: Upload failed: ${file.name}:`, error);
          }
        }
      });

      // Wait for all uploads to complete
      await Promise.allSettled(uploadPromises);

      // Enhanced results handling
      if (failedUploads.length > 0) {
        const networkErrors = failedUploads.filter(f => f.error.code === 'NETWORK_ERROR');
        const sizeErrors = failedUploads.filter(f => f.error.message?.includes('exceeds'));
        const otherErrors = failedUploads.filter(f => !networkErrors.includes(f) && !sizeErrors.includes(f));

        let errorMsg = `${failedUploads.length} file(s) failed to upload`;
        let suggestions = [];

        // Provide specific suggestions based on error types
        if (networkErrors.length > 0) {
          suggestions.push('SUBSCRIPTION: Check your internet connection and try again');
          suggestions.push('🔄 Server may be temporarily down - please retry later');
        }
        if (sizeErrors.length > 0) {
          suggestions.push('📏 Compress large files or use files under 100MB');
        }
        if (otherErrors.length > 0) {
          suggestions.push('Check file format and try again');
        }

        globalError.value = errorMsg;

        // Enhanced user feedback with suggestions
        if (typeof window !== 'undefined' && window.showFileUploadError) {
          window.showFileUploadError(
            `${errorMsg}:\n${suggestions.join('\n')}`,
            failedUploads.length,
            suggestions
          );
        }

        // Only throw if ALL uploads failed
        if (failedUploads.length === readyFiles.length) {
          throw new Error(`${errorMsg}. ${suggestions.join(' ')}`);
        } else {
          if (import.meta.env.DEV) {
            console.warn(`WARNING: Partial upload failure: ${uploadedUrls.length} succeeded, ${failedUploads.length} failed`);
          }
        }
      }

      // Remove successfully uploaded files from pending list
      const successfulFiles = readyFiles.filter(f => f.status === 'completed');
      successfulFiles.forEach(file => removeFile(file.id));

      // Success notification
      if (uploadedUrls.length > 0 && typeof window !== 'undefined' && window.showFileUploadNotification) {
        window.showFileUploadNotification(
          `🎉 ${uploadedUrls.length} file(s) uploaded successfully!`,
          'success'
        );
      }

      if (import.meta.env.DEV) {
        console.log(`🎉 Upload completed: ${uploadedUrls.length} files uploaded successfully`);
      }
      return uploadedUrls;
    } finally {
      isUploading.value = false;
    }
  };

  /**
   * Get upload statistics
   */
  const getUploadStats = () => {
    return {
      total: pendingFiles.value.length,
      pending: getFilesByStatus('pending').length,
      compressing: getFilesByStatus('compressing').length,
      ready: getFilesByStatus('pending_upload').length,
      uploading: getFilesByStatus('uploading').length,
      completed: getFilesByStatus('completed').length,
      failed: getFilesByStatus('error').length,
      totalSize: getTotalSize(),
      isUploading: isUploading.value,
      hasErrors: globalError.value !== null
    };
  };

  /**
   * Cleanup on store destruction
   */
  const cleanup = () => {
    clearAll();
  };

  // ================================
  // Return Public API
  // ================================
  return {
    // State
    pendingFiles: pendingFiles,
    isUploading,
    globalError,
    uploadConfig,

    // Getters
    hasPendingFiles,
    getTotalSize,
    getFilesByStatus,
    hasReadyFiles,
    getUploadStats,
    getMaxFileSize,
    getMaxFileSizeFormatted,

    // Actions
    addFiles,
    removeFile,
    clearAll,
    retryFile,
    uploadAll,
    cleanup
  };
}); 