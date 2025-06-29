/**
 * FILE UPLOAD DAG FIX: Unified File Validation System
 * 
 * 符合DAG结构的统一文件验证服务
 * 解决重复验证逻辑和状态不一致问题
 */

// File validation states following DAG pattern
export const FILE_STATES = {
  INITIAL: 'initial',
  VALIDATING: 'validating',
  VALID: 'valid',
  INVALID: 'invalid',
  COMPRESSING: 'compressing',
  COMPRESSED: 'compressed',
  UPLOADING: 'uploading',
  UPLOADED: 'uploaded',
  ERROR: 'error'
};

// Error types for structured error handling
export const ERROR_TYPES = {
  SIZE_EXCEEDED: 'size_exceeded',
  TYPE_NOT_SUPPORTED: 'type_not_supported',
  EMPTY_FILE: 'empty_file',
  LIMIT_EXCEEDED: 'limit_exceeded',
  COMPRESSION_FAILED: 'compression_failed',
  UPLOAD_FAILED: 'upload_failed',
  NETWORK_ERROR: 'network_error'
};

// Recovery strategies following DAG flow
export const RECOVERY_STRATEGIES = {
  COMPRESS: 'compress',
  REMOVE: 'remove',
  RETRY: 'retry',
  MANUAL: 'manual'
};

class FileValidator {
  constructor(config = {}) {
    this.config = {
      maxFileSize: 100 * 1024 * 1024, // 100MB
      maxFiles: 10,
      allowedTypes: [
        'image/jpeg', 'image/png', 'image/gif', 'image/webp', 'image/svg+xml',
        'video/mp4', 'video/avi', 'video/mov', 'video/wmv', 'video/webm',
        'audio/mp3', 'audio/wav', 'audio/ogg', 'audio/m4a', 'audio/aac',
        'application/pdf', 'application/msword',
        'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
        'text/plain', 'application/json', 'text/yaml', 'application/x-yaml'
      ],
      compressionOptions: {
        maxSizeMB: 1.8,
        maxWidthOrHeight: 1920,
        useWebWorker: true,
        fileType: 'image/jpeg',
        initialQuality: 0.8
      },
      ...config
    };

    // Event listeners for DAG state changes
    this.listeners = new Map();
    this.fileStates = new Map();
  }

  /**
   * DAG Entry Point: Unified file validation
   */
  async validateFiles(files, currentFileCount = 0) {
    const result = {
      valid: [],
      invalid: [],
      warnings: [],
      stats: {
        total: files.length,
        validCount: 0,
        invalidCount: 0,
        totalSize: 0
      }
    };

    for (const file of files) {
      const validation = await this.validateSingleFile(file, currentFileCount + result.valid.length);

      if (validation.isValid) {
        result.valid.push({
          file,
          validation,
          state: FILE_STATES.VALID,
          id: this.generateFileId()
        });
        result.stats.validCount++;
      } else {
        result.invalid.push({
          file,
          validation,
          state: FILE_STATES.INVALID,
          id: this.generateFileId(),
          recoveryStrategy: this.determineRecoveryStrategy(validation)
        });
        result.stats.invalidCount++;
      }

      result.stats.totalSize += file.size;
    }

    // Add warnings for files close to limit
    result.warnings = this.generateWarnings(result.valid);

    // Emit validation complete event
    this.emit('validation:complete', result);

    return result;
  }

  /**
   * DAG Node: Single file validation
   */
  async validateSingleFile(file, currentIndex = 0) {
    const fileId = this.generateFileId();
    this.setFileState(fileId, FILE_STATES.VALIDATING);

    const validation = {
      fileId,
      fileName: file.name,
      fileSize: this.formatFileSize(file.size),
      fileSizeBytes: file.size,
      errors: [],
      warnings: [],
      isValid: false,
      isNearLimit: false,
      canCompress: false,
      timestamp: Date.now()
    };

    try {
      // Size validation
      if (file.size === 0) {
        validation.errors.push({
          type: ERROR_TYPES.EMPTY_FILE,
          message: `File "${file.name}" is empty and cannot be uploaded.`,
          severity: 'error'
        });
      } else if (file.size > this.config.maxFileSize) {
        validation.errors.push({
          type: ERROR_TYPES.SIZE_EXCEEDED,
          message: `File "${file.name}" (${validation.fileSize}) exceeds the 100MB limit.`,
          severity: 'error',
          details: {
            fileSize: file.size,
            maxSize: this.config.maxFileSize,
            canCompress: file.type.startsWith('image/')
          }
        });
      } else if (file.size > this.config.maxFileSize * 0.9) {
        validation.isNearLimit = true;
        validation.warnings.push({
          type: 'size_warning',
          message: `File "${file.name}" (${validation.fileSize}) is close to the 100MB limit.`,
          severity: 'warning'
        });
      }

      // Type validation
      if (!this.config.allowedTypes.includes(file.type)) {
        validation.errors.push({
          type: ERROR_TYPES.TYPE_NOT_SUPPORTED,
          message: `File type "${file.type}" is not supported.`,
          severity: 'error',
          details: {
            fileType: file.type,
            allowedTypes: this.config.allowedTypes
          }
        });
      }

      // Count validation
      if (currentIndex >= this.config.maxFiles) {
        validation.errors.push({
          type: ERROR_TYPES.LIMIT_EXCEEDED,
          message: `Maximum ${this.config.maxFiles} files allowed.`,
          severity: 'error'
        });
      }

      // Compression capability check
      validation.canCompress = file.type.startsWith('image/') && file.size > this.config.maxFileSize;

      // Final validation result
      validation.isValid = validation.errors.length === 0;

      // Update state based on validation result
      this.setFileState(fileId, validation.isValid ? FILE_STATES.VALID : FILE_STATES.INVALID);

      return validation;
    } catch (error) {
      validation.errors.push({
        type: ERROR_TYPES.NETWORK_ERROR,
        message: `Validation failed: ${error.message}`,
        severity: 'error'
      });

      this.setFileState(fileId, FILE_STATES.ERROR);
      return validation;
    }
  }

  /**
   * DAG Flow: Determine recovery strategy based on error type
   */
  determineRecoveryStrategy(validation) {
    const sizeErrors = validation.errors.filter(e => e.type === ERROR_TYPES.SIZE_EXCEEDED);
    const typeErrors = validation.errors.filter(e => e.type === ERROR_TYPES.TYPE_NOT_SUPPORTED);

    if (sizeErrors.length > 0) {
      const sizeError = sizeErrors[0];
      if (sizeError.details?.canCompress) {
        return RECOVERY_STRATEGIES.COMPRESS;
      }
      return RECOVERY_STRATEGIES.REMOVE;
    }

    if (typeErrors.length > 0) {
      return RECOVERY_STRATEGIES.REMOVE;
    }

    return RECOVERY_STRATEGIES.MANUAL;
  }

  /**
   * DAG State Management
   */
  setFileState(fileId, state) {
    const oldState = this.fileStates.get(fileId);
    this.fileStates.set(fileId, state);

    this.emit('state:changed', { fileId, oldState, newState: state });
  }

  getFileState(fileId) {
    return this.fileStates.get(fileId) || FILE_STATES.INITIAL;
  }

  /**
   * Event System for DAG communication
   */
  on(event, callback) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event).push(callback);
  }

  emit(event, data) {
    const callbacks = this.listeners.get(event) || [];
    callbacks.forEach(callback => {
      try {
        callback(data);
      } catch (error) {
        console.error(`Event callback error for ${event}:`, error);
      }
    });
  }

  /**
   * Utility Methods
   */
  generateFileId() {
    return `file-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  formatFileSize(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  generateWarnings(validFiles) {
    const warnings = [];
    const nearLimitFiles = validFiles.filter(f => f.validation.isNearLimit);

    if (nearLimitFiles.length > 0) {
      warnings.push({
        type: 'size_warning',
        message: `${nearLimitFiles.length} file(s) are close to the 100MB limit.`,
        files: nearLimitFiles.map(f => f.file.name)
      });
    }

    return warnings;
  }

  cleanup() {
    this.fileStates.clear();
    this.listeners.clear();
  }
}

// Export singleton instance
export default new FileValidator();

// Export class for custom instances
export { FileValidator };
