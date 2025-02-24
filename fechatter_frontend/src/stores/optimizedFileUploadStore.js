/**
 * 🔧 FILE UPLOAD DAG FIX: Optimized File Upload Store
 * 
 * 基于DAG结构优化的文件上传状态管理
 * 使用统一验证器和状态机模式
 */

import { defineStore } from 'pinia';
import { ref, reactive, computed } from 'vue';
import FileValidator, { FILE_STATES, ERROR_TYPES, RECOVERY_STRATEGIES } from '../services/FileValidator.js';

export const useOptimizedFileUploadStore = defineStore('optimizedFileUpload', () => {
  // ================================
  // Core State Management (Single Source of Truth)
  // ================================
  const files = ref(new Map()); // Map<fileId, fileData>
  const isProcessing = ref(false);
  const globalError = ref(null);
  const globalStatus = ref('idle'); // idle, validating, uploading, completed, error

  // ================================
  // Event-driven state updates
  // ================================
  const eventBus = reactive({
    lastEvent: null,
    timestamp: null
  });

  // Initialize FileValidator with event listeners
  const validator = FileValidator;
  
  // Setup DAG event listeners
  validator.on('validation:complete', (result) => {
    updateFilesFromValidation(result);
    emitEvent('validation:complete', result);
  });

  validator.on('compression:started', ({ fileId }) => {
    updateFileState(fileId, FILE_STATES.COMPRESSING);
    emitEvent('compression:started', { fileId });
  });

  validator.on('compression:success', ({ fileId, compressedFile }) => {
    const fileData = files.value.get(fileId);
    if (fileData) {
      fileData.file = compressedFile;
      fileData.state = FILE_STATES.COMPRESSED;
      fileData.isCompressed = true;
    }
    emitEvent('compression:success', { fileId, compressedFile });
  });

  validator.on('state:changed', ({ fileId, newState }) => {
    updateFileState(fileId, newState);
  });

  // ================================
  // Computed Properties (DAG Aggregation)
  // ================================
  const fileStats = computed(() => {
    const stats = {
      total: files.value.size,
      byState: {},
      totalSize: 0,
      validFiles: [],
      invalidFiles: [],
      uploadableFiles: []
    };

    // Initialize state counters
    Object.values(FILE_STATES).forEach(state => {
      stats.byState[state] = 0;
    });

    // Process all files
    for (const [fileId, fileData] of files.value) {
      stats.byState[fileData.state]++;
      stats.totalSize += fileData.file.size;

      if (fileData.state === FILE_STATES.VALID || fileData.state === FILE_STATES.COMPRESSED) {
        stats.validFiles.push(fileData);
        stats.uploadableFiles.push(fileData);
      } else if (fileData.state === FILE_STATES.INVALID || fileData.state === FILE_STATES.ERROR) {
        stats.invalidFiles.push(fileData);
      }
    }

    return stats;
  });

  const canUpload = computed(() => {
    return fileStats.value.uploadableFiles.length > 0 && !isProcessing.value;
  });

  const hasErrors = computed(() => {
    return fileStats.value.byState[FILE_STATES.INVALID] > 0 || 
           fileStats.value.byState[FILE_STATES.ERROR] > 0 ||
           globalError.value !== null;
  });

  // ================================
  // Core Actions (DAG Entry Points)
  // ================================

  /**
   * 🔧 DAG Entry: Add files with unified validation
   */
  const addFiles = async (newFiles) => {
    if (!newFiles || newFiles.length === 0) {
      return { success: 0, failed: 0, warnings: [] };
    }

    globalStatus.value = 'validating';
    isProcessing.value = true;
    globalError.value = null;

    try {
      // Use unified validator
      const validationResult = await validator.validateFiles(
        Array.from(newFiles), 
        files.value.size
      );

      // Process validation result
      const result = {
        success: validationResult.stats.validCount,
        failed: validationResult.stats.invalidCount,
        warnings: validationResult.warnings
      };

      globalStatus.value = hasErrors.value ? 'error' : 'idle';
      emitEvent('files:added', result);

      return result;
    } catch (error) {
      globalError.value = error.message;
      globalStatus.value = 'error';
      throw error;
    } finally {
      isProcessing.value = false;
    }
  };

  /**
   * 🔧 DAG Action: Smart file removal with recovery
   */
  const removeFile = (fileId) => {
    const fileData = files.value.get(fileId);
    if (!fileData) return false;

    // Cleanup resources
    if (fileData.preview) {
      URL.revokeObjectURL(fileData.preview);
    }

    files.value.delete(fileId);
    emitEvent('file:removed', { fileId, fileName: fileData.file.name });
    
    return true;
  };

  /**
   * 🔧 DAG Action: Auto-recovery for invalid files
   */
  const executeAutoRecovery = async () => {
    const invalidFiles = fileStats.value.invalidFiles;
    const recoveryResults = [];

    for (const fileData of invalidFiles) {
      try {
        const result = await validator.executeRecoveryStrategy(
          fileData, 
          fileData.recoveryStrategy
        );
        
        recoveryResults.push({
          fileId: fileData.id,
          fileName: fileData.file.name,
          strategy: fileData.recoveryStrategy,
          result
        });
      } catch (error) {
        recoveryResults.push({
          fileId: fileData.id,
          fileName: fileData.file.name,
          strategy: fileData.recoveryStrategy,
          result: { success: false, error: error.message }
        });
      }
    }

    emitEvent('recovery:completed', recoveryResults);
    return recoveryResults;
  };

  /**
   * 🔧 DAG Flow: Upload all valid files
   */
  const uploadAllFiles = async () => {
    const uploadableFiles = fileStats.value.uploadableFiles;
    
    if (uploadableFiles.length === 0) {
      throw new Error('No files ready for upload');
    }

    globalStatus.value = 'uploading';
    isProcessing.value = true;
    
    const uploadResults = [];
    const failedUploads = [];

    try {
      // Import ChatService dynamically to avoid circular dependency
      const { default: ChatService } = await import('../services/ChatService.ts');

      // Upload files in parallel with controlled concurrency
      const uploadPromises = uploadableFiles.map(async (fileData) => {
        try {
          updateFileState(fileData.id, FILE_STATES.UPLOADING);
          
          const uploadedFile = await ChatService.uploadFile(fileData.file, (progress) => {
            // Update progress
            fileData.progress = Math.min(progress, 99);
          });

          // Update state
          updateFileState(fileData.id, FILE_STATES.UPLOADED);
          fileData.progress = 100;
          fileData.uploadedFile = uploadedFile;

          uploadResults.push(uploadedFile.url);
          
          return { success: true, fileData, uploadedFile };
        } catch (error) {
          updateFileState(fileData.id, FILE_STATES.ERROR);
          fileData.error = error.message;
          fileData.progress = 0;
          
          failedUploads.push({ fileData, error });
          
          return { success: false, fileData, error };
        }
      });

      await Promise.allSettled(uploadPromises);

      // Handle results
      if (failedUploads.length > 0) {
        const errorMsg = `${failedUploads.length} file(s) failed to upload`;
        globalError.value = errorMsg;

        if (failedUploads.length === uploadableFiles.length) {
          globalStatus.value = 'error';
          throw new Error(errorMsg);
        } else {
          globalStatus.value = 'completed';
        }
      } else {
        globalStatus.value = 'completed';
      }

      // Remove successfully uploaded files
      const successfulFiles = uploadableFiles.filter(f => f.state === FILE_STATES.UPLOADED);
      successfulFiles.forEach(f => removeFile(f.id));

      emitEvent('upload:completed', {
        successful: uploadResults.length,
        failed: failedUploads.length,
        urls: uploadResults
      });

      return uploadResults;
    } finally {
      isProcessing.value = false;
    }
  };

  /**
   * 🔧 DAG Cleanup: Clear all files and reset state
   */
  const clearAll = () => {
    // Cleanup resources
    for (const [fileId, fileData] of files.value) {
      if (fileData.preview) {
        URL.revokeObjectURL(fileData.preview);
      }
    }

    files.value.clear();
    globalError.value = null;
    globalStatus.value = 'idle';
    isProcessing.value = false;
    
    emitEvent('files:cleared', {});
  };

  // ================================
  // Internal Helper Functions
  // ================================

  function updateFilesFromValidation(validationResult) {
    // Add valid files
    validationResult.valid.forEach(({ file, validation, state, id }) => {
      const fileData = createFileData(file, id, state);
      fileData.validation = validation;
      files.value.set(id, fileData);
    });

    // Add invalid files
    validationResult.invalid.forEach(({ file, validation, state, id, recoveryStrategy }) => {
      const fileData = createFileData(file, id, state);
      fileData.validation = validation;
      fileData.recoveryStrategy = recoveryStrategy;
      files.value.set(id, fileData);
    });
  }

  function createFileData(file, id, state) {
    const fileData = reactive({
      id,
      file,
      state,
      progress: 0,
      error: null,
      preview: null,
      validation: null,
      recoveryStrategy: null,
      uploadedFile: null,
      isCompressed: false,
      createdAt: Date.now()
    });

    // Generate preview for images
    if (file.type.startsWith('image/')) {
      try {
        fileData.preview = URL.createObjectURL(file);
      } catch (error) {
        console.warn('Failed to create preview:', error);
      }
    }

    return fileData;
  }

  function updateFileState(fileId, newState) {
    const fileData = files.value.get(fileId);
    if (fileData) {
      fileData.state = newState;
    }
  }

  function emitEvent(eventType, data) {
    eventBus.lastEvent = eventType;
    eventBus.timestamp = Date.now();
    
    // Emit to external listeners if needed
    if (typeof window !== 'undefined' && window.fileUploadEvents) {
      window.fileUploadEvents.emit(eventType, data);
    }
  }

  // ================================
  // Public API
  // ================================
  return {
    // State
    files: computed(() => Array.from(files.value.values())),
    isProcessing,
    globalError,
    globalStatus,
    eventBus,

    // Computed
    fileStats,
    canUpload,
    hasErrors,

    // Actions
    addFiles,
    removeFile,
    clearAll,
    uploadAllFiles,
    executeAutoRecovery,

    // Utilities
    getFileById: (id) => files.value.get(id),
    getFilesByState: (state) => Array.from(files.value.values()).filter(f => f.state === state),
    
    // Config
    getConfig: () => validator.getConfig(),
    updateConfig: (config) => validator.updateConfig(config)
  };
});
