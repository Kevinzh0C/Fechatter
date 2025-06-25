/**
 * Intelligent Type Inference System
 * Automatically infers and adapts API response formats to avoid hardcoding mismatches
 */

export interface BaseResponse {
  success?: boolean;
  data?: any;
  error?: {
    message?: string;
    code?: string;
    validation_errors?: any[];
  };
}

export interface UploadedFile {
  id: string | number;
  filename: string;
  url: string;
  mime_type: string;
  size: number;
  created_at: string;
}

/**
 * Smart Response Adapter - Automatically infers response format
 */
export class ResponseAdapter {
  /**
   * Intelligently parse upload response regardless of backend format
   */
  static parseUploadResponse(response: any, originalFile: File): UploadedFile {
    if (!response) {
      throw new Error('No response data received');
    }

    // Try to infer the response structure
    const responseData = this.extractDataFromResponse(response);
    
    if (!responseData) {
      throw new Error('Unable to extract valid data from response');
    }

    // Smart field mapping with auto-inference
    return {
      id: this.inferField(responseData, ['id', 'file_id', 'uuid'], Date.now()),
      filename: this.inferField(responseData, ['filename', 'file_name', 'name'], originalFile.name),
      url: this.inferField(responseData, ['url', 'file_url', 'path', 'download_url'], ''),
      mime_type: this.inferField(responseData, ['mime_type', 'content_type', 'type', 'file_type'], originalFile.type || 'application/octet-stream'),
      size: this.inferField(responseData, ['size', 'file_size', 'filesize'], originalFile.size),
      created_at: this.inferField(responseData, ['created_at', 'upload_time', 'timestamp'], new Date().toISOString())
    };
  }

  /**
   * Extract data from various response formats
   */
  private static extractDataFromResponse(response: any): any {
    // Standard format: { success: true, data: {...} }
    if (response.success && response.data) {
      return response.data;
    }
    
    // Error format: { success: false, error: {...} }
    if (response.success === false) {
      throw new Error(response.error?.message || 'Upload failed');
    }
    
    // Direct format: { file_url: '...', file_name: '...' }
    if (this.hasUploadFields(response)) {
      return response;
    }
    
    // Nested data format: { result: { data: {...} } }
    if (response.result?.data) {
      return response.result.data;
    }
    
    // Array format: [{ url: '...', filename: '...' }]
    if (Array.isArray(response) && response.length > 0) {
      return response[0];
    }
    
    return null;
  }

  /**
   * Check if response has typical upload fields
   */
  private static hasUploadFields(obj: any): boolean {
    const uploadFields = ['url', 'file_url', 'filename', 'file_name'];
    return uploadFields.some(field => field in obj);
  }

  /**
   * Intelligently infer field value from multiple possible field names
   */
  private static inferField<T>(
    obj: any, 
    possibleFields: string[], 
    fallback: T
  ): T {
    for (const field of possibleFields) {
      if (obj && obj[field] !== undefined && obj[field] !== null) {
        return obj[field];
      }
    }
    return fallback;
  }
}

/**
 * Request Configuration Auto-Inferrer
 */
export class RequestConfigInferrer {
  /**
   * Automatically infer optimal request configuration for file uploads
   */
  static inferUploadConfig(file: File): {
    timeout: number;
    maxRetries: number;
    retryDelay: number;
    chunkSize?: number;
  } {
    const sizeInMB = file.size / (1024 * 1024);
    
    // Adaptive configuration based on file size
    if (sizeInMB > 10) {
      return {
        timeout: 120000, // 2 minutes for large files
        maxRetries: 5,
        retryDelay: 2000,
        chunkSize: 1024 * 1024 // 1MB chunks for large files
      };
    } else if (sizeInMB > 2) {
      return {
        timeout: 60000, // 1 minute for medium files
        maxRetries: 3,
        retryDelay: 1000
      };
    } else {
      return {
        timeout: 30000, // 30 seconds for small files
        maxRetries: 3,
        retryDelay: 500
      };
    }
  }

  /**
   * Infer optimal headers (never include Content-Type for FormData!)
   */
  static inferHeaders(body: any): Record<string, string> {
    const headers: Record<string, string> = {};
    
    // CRITICAL: Never set Content-Type for FormData - browser auto-sets with boundary
    if (body instanceof FormData) {
      // Let browser handle Content-Type automatically
      return headers;
    }
    
    // For JSON data
    if (typeof body === 'object' && !(body instanceof FormData)) {
      headers['Content-Type'] = 'application/json';
    }
    
    return headers;
  }
}

export default {
  ResponseAdapter,
  RequestConfigInferrer
};
