/**
 * Chat Service
 * 处理聊天相关的API调用
 */
import api from './api';
import type {
  Chat,
  ChatMember,
  CreateChatRequest,
  UpdateChatRequest,
  AddChatMembersRequest,
  ChatMessage,
  SendMessageRequest,
  PaginatedResponse,
  PaginationParams,
  ApiResponse,
  UploadedFile,
  FileUploadResponse
} from '@/types/api';

class ChatService {
  /**
   * 获取工作区聊天列表
   */
  async getWorkspaceChats(): Promise<Chat[]> {
    try {
      const response = await api.get<ApiResponse<Chat[]>>('/workspace/chats');

      if (response.data.success && response.data.data) {
        return response.data.data;
      }

      throw new Error(response.data.error?.message || 'Failed to get workspace chats');
    } catch (error) {
      console.error('❌ [ChatService] Get workspace chats failed:', error);
      throw this.handleError(error);
    }
  }

  /**
   * 创建新聊天
   */
  async createChat(chatData: CreateChatRequest): Promise<Chat> {
    try {
      const response = await api.post<ApiResponse<Chat>>('/workspace/chats', chatData);

      if (response.data.success && response.data.data) {
        return response.data.data;
      }

      throw new Error(response.data.error?.message || 'Failed to create chat');
    } catch (error) {
      console.error('❌ [ChatService] Create chat failed:', error);
      throw this.handleError(error);
    }
  }

  /**
   * 获取聊天详情
   */
  async getChatDetails(chatId: number): Promise<Chat> {
    try {
      const response = await api.get<ApiResponse<Chat>>(`/chat/${chatId}`);

      if (response.data.success && response.data.data) {
        return response.data.data;
      }

      throw new Error(response.data.error?.message || 'Failed to get chat details');
    } catch (error) {
      console.error(`❌ [ChatService] Get chat details failed for chat ${chatId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 更新聊天信息
   */
  async updateChat(chatId: number, updates: UpdateChatRequest): Promise<Chat> {
    try {
      const response = await api.put<ApiResponse<Chat>>(`/chat/${chatId}`, updates);

      if (response.data.success && response.data.data) {
        return response.data.data;
      }

      throw new Error(response.data.error?.message || 'Failed to update chat');
    } catch (error) {
      console.error(`❌ [ChatService] Update chat failed for chat ${chatId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 删除聊天
   */
  async deleteChat(chatId: number): Promise<void> {
    try {
      const response = await api.delete<ApiResponse<void>>(`/chat/${chatId}`);

      if (!response.data.success) {
        throw new Error(response.data.error?.message || 'Failed to delete chat');
      }
    } catch (error) {
      console.error(`❌ [ChatService] Delete chat failed for chat ${chatId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 获取聊天成员列表
   */
  async getChatMembers(chatId: number): Promise<ChatMember[]> {
    try {
      const response = await api.get<ApiResponse<ChatMember[]>>(`/chat/${chatId}/members`);

      if (response.data.success && response.data.data) {
        return response.data.data;
      }

      throw new Error(response.data.error?.message || 'Failed to get chat members');
    } catch (error) {
      console.error(`❌ [ChatService] Get chat members failed for chat ${chatId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 添加聊天成员
   */
  async addChatMembers(chatId: number, memberData: AddChatMembersRequest): Promise<ChatMember[]> {
    try {
      const response = await api.post<ApiResponse<ChatMember[]>>(`/chat/${chatId}/members`, memberData);

      if (response.data.success && response.data.data) {
        return response.data.data;
      }

      throw new Error(response.data.error?.message || 'Failed to add chat members');
    } catch (error) {
      console.error(`❌ [ChatService] Add chat members failed for chat ${chatId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 移除聊天成员
   */
  async removeChatMember(chatId: number, userId: number): Promise<void> {
    try {
      const response = await api.delete<ApiResponse<void>>(`/chat/${chatId}/members/${userId}`);

      if (!response.data.success) {
        throw new Error(response.data.error?.message || 'Failed to remove chat member');
      }
    } catch (error) {
      console.error(`❌ [ChatService] Remove chat member failed for chat ${chatId}, user ${userId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 加入聊天
   */
  async joinChat(chatId: number): Promise<ChatMember> {
    try {
      const response = await api.post<ApiResponse<ChatMember>>(`/chat/${chatId}/join`);

      if (response.data.success && response.data.data) {
        return response.data.data;
      }

      throw new Error(response.data.error?.message || 'Failed to join chat');
    } catch (error) {
      console.error(`❌ [ChatService] Join chat failed for chat ${chatId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 离开聊天
   */
  async leaveChat(chatId: number): Promise<void> {
    try {
      const response = await api.post<ApiResponse<void>>(`/chat/${chatId}/leave`);

      if (!response.data.success) {
        throw new Error(response.data.error?.message || 'Failed to leave chat');
      }
    } catch (error) {
      console.error(`❌ [ChatService] Leave chat failed for chat ${chatId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 获取聊天消息列表
   */
  async getChatMessages(
    chatId: number,
    params: PaginationParams = {}
  ): Promise<PaginatedResponse<ChatMessage>> {
    try {
      const queryParams = new URLSearchParams();

      // 基于数据库的分页参数
      if (params.limit) queryParams.append('limit', params.limit.toString());
      if (params.before) queryParams.append('before', params.before.toString()); // 基于消息ID的分页
      if (params.offset) queryParams.append('offset', params.offset.toString());

      // 排序参数 - 消息通常按创建时间倒序
      queryParams.append('order', 'desc');

      const url = `/chat/${chatId}/messages${queryParams.toString() ? '?' + queryParams.toString() : ''}`;
      const response = await api.get<ApiResponse<ChatMessage[]>>(url);

      if (response.data.success && response.data.data) {
        // 从数据库返回的是消息数组，需要包装成分页响应格式
        const messages = response.data.data;
        const limit = params.limit || 50;

        return {
          data: messages,
          pagination: {
            page: Math.floor((params.offset || 0) / limit) + 1,
            limit: limit,
            total: messages.length, // 注意：这只是当前页的数量，并非总数
            total_pages: 1, // 同样，无法从当前信息中得知
            has_next: messages.length === limit,
            has_prev: (params.offset || 0) > 0,
            has_more: messages.length === limit
          }
        };
      }

      throw new Error(response.data.error?.message || 'Failed to get chat messages');
    } catch (error) {
      console.error(`❌ [ChatService] Get chat messages failed for chat ${chatId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 发送消息
   */
  async sendMessage(chatId: number, messageData: SendMessageRequest): Promise<ChatMessage> {
    try {
      if (import.meta.env.DEV) {
        console.log(`📤 [ChatService] Processing sendMessage for chat ${chatId}:`, messageData);
      }

      // 1. Handle file uploads - 支持文件对象和文件URL
      let uploadedFiles: UploadedFile[] = [];
      const filesToUpload = messageData.files?.filter(f => f instanceof File) as File[] || [];

      // 如果有File对象需要上传
      if (filesToUpload.length > 0) {
        const uploadPromises = filesToUpload.map(file => this.uploadFile(file as File));
        uploadedFiles = await Promise.all(uploadPromises);
      }

      // 2. 整理文件URL - 支持已上传的URL和新上传的文件
      const newFileUrls = uploadedFiles.map(f => f.url || f.file_url); // 支持两种格式
      const existingFileUrls = messageData.files?.filter(f => typeof f === 'string') as string[] || [];
      const allFileUrls = [...existingFileUrls, ...newFileUrls];

      if (import.meta.env.DEV) {
        console.log(`📎 [ChatService] File processing complete:`, {
          newFiles: newFileUrls,
          existingFiles: existingFileUrls,
          totalFiles: allFileUrls
        });
      }

      // 3. 构建请求内容
      const content = (messageData.content || '').trim();
      // 如果没有文本内容但有文件，使用默认的文件图标
      const finalContent = content.length > 0 ? content : (allFileUrls.length > 0 ? '📎 File attachment' : '');

      if (!finalContent) {
        throw new Error("Message content cannot be empty unless files are present.");
      }

      // 4. 构建符合后端API的请求载荷
      const requestPayload = {
        content: finalContent,
        files: allFileUrls.length > 0 ? allFileUrls : undefined,
        reply_to: messageData.reply_to || undefined,
        mentions: (messageData.mentions && messageData.mentions.length > 0) ? messageData.mentions : undefined,
        idempotency_key: messageData.idempotency_key || this.generateIdempotencyKey()
      };

      if (import.meta.env.DEV) {
        console.log(`🚀 [ChatService] Sending request payload:`, requestPayload);
      }

      // 5. 发送到后端API
      const response = await api.post(`/chat/${chatId}/messages`, requestPayload);

      if (response.data.success && response.data.data) {
        const sentMessage = response.data.data;

        if (import.meta.env.DEV) {
          console.log(`✅ [ChatService] Message sent successfully:`, sentMessage);
        }

        return sentMessage;
      }

      // 处理API错误响应
      const errorMessage = response.data.error?.message || 'Failed to send message';
      if (import.meta.env.DEV) {
        console.error(`❌ [ChatService] API error response:`, response.data);
      }

      throw new Error(errorMessage);

    } catch (error) {
      console.error(`❌ [ChatService] Send message failed for chat ${chatId}:`, error);

      // 增强错误处理，提供用户友好的错误信息
      if (error.response?.status === 401) {
        throw new Error('Authentication failed. Please login again.');
      } else if (error.response?.status === 403) {
        throw new Error('You do not have permission to send messages in this chat.');
      } else if (error.response?.status === 413) {
        throw new Error('Message or files are too large. Please reduce the size and try again.');
      } else if (error.response?.status === 422) {
        throw new Error('Invalid message format. Please check your content and try again.');
      }

      throw this.handleError(error);
    }
  }

  /**
   * 生成幂等性密钥
   */
  private generateIdempotencyKey(): string {
    // Generate a valid UUID v4
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
      const r = Math.random() * 16 | 0;
      const v = c == 'x' ? r : (r & 0x3 | 0x8);
      return v.toString(16);
    });
  }

  /**
   * 编辑消息
   */
  async editMessage(chatId: number, messageId: number, content: string): Promise<ChatMessage> {
    try {
      const response = await api.put<ApiResponse<ChatMessage>>(
        `/chat/${chatId}/messages/${messageId}`,
        { content }
      );

      if (response.data.success && response.data.data) {
        return response.data.data;
      }

      throw new Error(response.data.error?.message || 'Failed to edit message');
    } catch (error) {
      console.error(`❌ [ChatService] Edit message failed for chat ${chatId}, message ${messageId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 删除消息
   */
  async deleteMessage(chatId: number, messageId: number): Promise<void> {
    try {
      const response = await api.delete<ApiResponse<void>>(`/chat/${chatId}/messages/${messageId}`);

      if (!response.data.success) {
        throw new Error(response.data.error?.message || 'Failed to delete message');
      }
    } catch (error) {
      console.error(`❌ [ChatService] Delete message failed for chat ${chatId}, message ${messageId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 标记消息为已读
   */
  async markAsRead(chatId: number, messageId?: number): Promise<void> {
    try {
      const url = messageId
        ? `/chat/${chatId}/mark-read/${messageId}`
        : `/chat/${chatId}/mark-read`;

      const response = await api.post<ApiResponse<void>>(url);

      if (!response.data.success) {
        throw new Error(response.data.error?.message || 'Failed to mark as read');
      }
    } catch (error) {
      console.error(`❌ [ChatService] Mark as read failed for chat ${chatId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 发送打字状态
   */
  async sendTypingStatus(chatId: number, isTyping: boolean): Promise<void> {
    try {
      const response = await api.post<ApiResponse<void>>(`/chat/${chatId}/typing`, {
        is_typing: isTyping
      });

      if (!response.data.success) {
        throw new Error(response.data.error?.message || 'Failed to send typing status');
      }
    } catch (error) {
      // 打字状态失败不需要抛出错误，只记录日志
      console.warn(`⚠️ [ChatService] Send typing status failed for chat ${chatId}:`, error);
    }
  }

  /**
   * 搜索工作区内的聊天
   */
  async searchChats(query: string, limit: number = 10): Promise<Chat[]> {
    try {
      const response = await api.get<ApiResponse<Chat[]>>(
        `/workspace/chats/search?q=${encodeURIComponent(query)}&limit=${limit}`
      );

      if (response.data.success && response.data.data) {
        return response.data.data;
      }

      return [];
    } catch (error) {
      console.warn('⚠️ [ChatService] Search chats failed:', error);
      return [];
    }
  }

  /**
   * 创建直接消息聊天
   */
  async createDirectMessage(userId: number): Promise<Chat> {
    try {
      const response = await api.post<ApiResponse<Chat>>(
        '/workspace/chats',
        {
          name: 'Direct Message', // Non-empty name
          chat_type: 'Single',    // Correct ChatType
          members: [userId],      // Single member
          description: null       // Optional
        }
      );

      if (response.data.success && response.data.data) {
        return response.data.data;
      }

      throw new Error(response.data.error?.message || 'Failed to create direct message');
    } catch (error) {
      console.error(`❌ [ChatService] Create direct message failed for user ${userId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 错误处理
   */
  private handleError(error: any): Error {
    // 网络错误
    if (!error.response) {
      const networkError = new Error('Network error. Please check your connection.');
      (networkError as any).code = 'NETWORK_ERROR';
      return networkError;
    }

    const status = error.response?.status;
    const data = error.response?.data;

    // 创建统一的错误对象
    const chatError = new Error();
    (chatError as any).code = data?.error?.code || 'UNKNOWN_ERROR';
    (chatError as any).status = status;
    (chatError as any).originalError = error;

    // 根据状态码设置用户友好的错误信息
    switch (status) {
      case 400:
        chatError.message = data?.error?.message || 'Invalid request. Please check your input.';
        break;

      case 401:
        chatError.message = 'You are not authorized to perform this action. Please log in again.';
        break;

      case 403:
        chatError.message = 'You do not have permission to access this chat.';
        break;

      case 404:
        chatError.message = 'The requested chat was not found.';
        break;

      case 422:
        chatError.message = data?.error?.message || 'Validation failed. Please check your input.';
        (chatError as any).validationErrors = data?.error?.validation_errors || [];
        break;

      case 429:
        chatError.message = 'Too many requests. Please wait a moment and try again.';
        break;

      case 500:
        chatError.message = 'Internal server error. Please try again later.';
        break;

      default:
        chatError.message = data?.error?.message || 'An unexpected error occurred.';
    }

    return chatError;
  }

  // ================================
  // 新增的消息系统功能
  // ================================

  /**
   * 获取未读消息数量
   */
  async getUnreadCount(chatId?: number): Promise<{ chatId?: number; unreadCount: number }[]> {
    try {
      const url = chatId ? `/chat/${chatId}/unread` : `/chats/unread`;
      const response = await api.get<ApiResponse<any>>(url);

      if (response.data.success) {
        // 如果是单个聊天的未读数量
        if (chatId) {
          return [{ chatId, unreadCount: response.data.data?.unread_count || 0 }];
        }
        // 如果是所有聊天的未读数量
        return response.data.data || [];
      }

      throw new Error(response.data.error?.message || 'Failed to get unread count');
    } catch (error) {
      console.error(`❌ [ChatService] Get unread count failed:`, error);
      // 优雅降级：返回空数组而不是抛出错误
      return [];
    }
  }

  /**
   * 批量标记消息为已读
   */
  async markMessagesAsRead(chatId: number, messageIds: number[]): Promise<void> {
    try {
      const response = await api.post<ApiResponse<void>>(
        `/chat/${chatId}/messages/read`,
        { message_ids: messageIds }
      );

      if (!response.data.success) {
        throw new Error(response.data.error?.message || 'Failed to mark messages as read');
      }
    } catch (error) {
      console.error(`❌ [ChatService] Batch mark as read failed for chat ${chatId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 在聊天中搜索消息
   */
  async searchMessages(
    chatId: number,
    query: string,
    options: {
      limit?: number;
      offset?: number;
      sort?: 'relevance' | 'date_desc' | 'date_asc';
    } = {}
  ): Promise<{ hits: ChatMessage[]; total: number; took_ms: number }> {
    try {
      const { limit = 20, offset = 0, sort = 'relevance' } = options;

      const response = await api.get<ApiResponse<any>>(
        `/chat/${chatId}/messages/search?q=${encodeURIComponent(query)}&limit=${limit}&offset=${offset}&sort=${sort}`
      );

      if (response.data.success && response.data.data) {
        return {
          hits: response.data.data.hits || [],
          total: response.data.data.total || 0,
          took_ms: response.data.data.took_ms || 0
        };
      }

      throw new Error(response.data.error?.message || 'Failed to search messages');
    } catch (error) {
      console.error(`❌ [ChatService] Search messages failed for chat ${chatId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 获取全局消息搜索建议
   */
  async getSearchSuggestions(query: string, limit: number = 5): Promise<string[]> {
    try {
      const response = await api.get<ApiResponse<{ suggestions: string[] }>>(
        `/search/suggestions?q=${encodeURIComponent(query)}&limit=${limit}`
      );

      if (response.data.success && response.data.data) {
        return response.data.data.suggestions || [];
      }

      return [];
    } catch (error) {
      console.warn(`⚠️ [ChatService] Get search suggestions failed:`, error);
      // 优雅降级：返回空数组
      return [];
    }
  }

  /**
   * 开始输入状态
   */
  async startTyping(chatId: number): Promise<void> {
    try {
      await api.post<ApiResponse<void>>(`/chat/${chatId}/typing/start`);
    } catch (error) {
      // 输入状态失败不影响用户体验，只记录警告
      console.warn(`⚠️ [ChatService] Start typing failed for chat ${chatId}:`, error);
    }
  }

  /**
   * 停止输入状态
   */
  async stopTyping(chatId: number): Promise<void> {
    try {
      await api.post<ApiResponse<void>>(`/chat/${chatId}/typing/stop`);
    } catch (error) {
      console.warn(`⚠️ [ChatService] Stop typing failed for chat ${chatId}:`, error);
    }
  }

  /**
   * 获取正在输入的用户
   */
  async getTypingUsers(chatId: number): Promise<{ userId: number; userName: string; startedAt: number }[]> {
    try {
      const response = await api.get<ApiResponse<any>>(`/chat/${chatId}/typing/users`);

      if (response.data.success && response.data.data) {
        return response.data.data.typing_users || [];
      }

      return [];
    } catch (error) {
      console.warn(`⚠️ [ChatService] Get typing users failed for chat ${chatId}:`, error);
      return [];
    }
  }

  /**
   * 获取消息回执状态
   */
  async getMessageReceipts(messageId: number): Promise<any[]> {
    try {
      const response = await api.get<ApiResponse<any>>(`/messages/${messageId}/receipts`);

      if (response.data.success && response.data.data) {
        return response.data.data;
      }

      return [];
    } catch (error) {
      console.warn(`⚠️ [ChatService] Get message receipts failed for message ${messageId}:`, error);
      return [];
    }
  }

  /**
   * 上传文件 - 增强版带重试机制和网络诊断
   */
  async uploadFile(file: File, onProgress?: (progress: number) => void): Promise<UploadedFile> {
    const formData = new FormData();
    formData.append('file', file);

    // 🔧 Upload configuration
    const uploadConfig = {
      maxRetries: 3,
      retryDelay: 1000, // Start with 1 second
      timeout: 30000,
      maxFileSize: 2 * 1024 * 1024 // 2MB
    };

    // 🔧 Pre-upload validation
    if (file.size > uploadConfig.maxFileSize) {
      throw new Error(`File size ${Math.round(file.size / 1024 / 1024 * 100) / 100}MB exceeds 2MB limit`);
    }

    // 🔧 Network diagnostic function
    const checkNetworkHealth = async (): Promise<boolean> => {
      try {
        // 🔧 CRITICAL FIX: Use direct fetch to avoid /api prefix
        // Health check should access infrastructure endpoint directly
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 5000);

        const response = await fetch('/health', {
          method: 'GET',
          signal: controller.signal,
          headers: {
            'Content-Type': 'application/json'
          }
        });

        clearTimeout(timeoutId);
        return response.status === 200;
      } catch (error: any) {
        console.warn('🔍 Network health check failed:', error.message);
        return false;
      }
    };

    // 🔧 Enhanced retry logic with exponential backoff
    for (let attempt = 1; attempt <= uploadConfig.maxRetries; attempt++) {
      try {
        console.log(`📤 Upload attempt ${attempt}/${uploadConfig.maxRetries} for ${file.name}`);

        const response = await api.post('/files/single', formData, {
          headers: {
            'Content-Type': 'multipart/form-data',
          },
          timeout: uploadConfig.timeout,
          onUploadProgress: (progressEvent) => {
            if (onProgress && progressEvent.total) {
              const progress = Math.round((progressEvent.loaded * 100) / progressEvent.total);
              onProgress(progress);
            }
          },
        });

        // 🔧 ENHANCED: 处理后端返回的数据结构 - 增强版带详细诊断
        if (response.data) {
          if (import.meta.env.DEV) {
            console.log('🔍 [ChatService] Upload response analysis:', {
              hasData: !!response.data,
              hasSuccess: 'success' in response.data,
              successValue: response.data.success,
              hasDataField: 'data' in response.data,
              dataFieldValue: response.data.data,
              hasError: 'error' in response.data,
              fullResponse: response.data
            });
          }

          // 🔧 CRITICAL FIX: 更宽松的响应格式检查
          // 检查标准格式: { success: true, data: {...} }
          if (response.data.success && response.data.data) {
            const uploadData = response.data.data;
            console.log(`✅ Upload successful: ${file.name} -> ${uploadData.url}`);

            // 转换为前端期望的 UploadedFile 格式
            return {
              id: uploadData.id,
              filename: uploadData.filename,
              url: uploadData.url,
              mime_type: uploadData.mime_type,
              size: uploadData.size,
              created_at: uploadData.created_at
            };
          }

          // 🔧 COMPATIBILITY FIX: 检查直接返回格式 (兼容不同后端版本)
          else if (response.data.file_url && response.data.file_name) {
            if (import.meta.env.DEV) {
              console.warn('⚠️ [ChatService] Non-standard response format detected, using compatibility mode');
              console.log('📊 [ChatService] Direct format response:', response.data);
            }

            const uploadData = response.data;
            return {
              id: uploadData.id || Date.now(),
              filename: uploadData.file_name,  // 使用 file_name
              url: uploadData.file_url,        // 使用 file_url
              mime_type: uploadData.mime_type || uploadData.file_type || 'application/octet-stream',
              size: uploadData.file_size || file.size,  // 使用 file_size
              created_at: uploadData.created_at || uploadData.upload_time || new Date().toISOString()
            };
          }

          // 🔧 BACKWARD COMPATIBILITY: 支持旧的字段名格式
          else if (response.data.url && response.data.filename) {
            if (import.meta.env.DEV) {
              console.warn('⚠️ [ChatService] Legacy response format detected');
              console.log('📊 [ChatService] Legacy format response:', response.data);
            }

            const uploadData = response.data;
            return {
              id: uploadData.id || Date.now(),
              filename: uploadData.filename,
              url: uploadData.url,
              mime_type: uploadData.mime_type || uploadData.type || 'application/octet-stream',
              size: uploadData.size || file.size,
              created_at: uploadData.created_at || new Date().toISOString()
            };
          }

          // 🔧 ERROR ANALYSIS: 详细的错误分析和建议
          else {
            const errorDetails = {
              responseStructure: Object.keys(response.data),
              hasSuccess: 'success' in response.data,
              successValue: response.data.success,
              hasData: 'data' in response.data,
              dataValue: response.data.data,
              hasError: 'error' in response.data,
              errorValue: response.data.error,
              suggestion: 'Check backend response format'
            };

            if (import.meta.env.DEV) {
              console.error('❌ [ChatService] Upload response format analysis:', errorDetails);
            }

            // 构建详细的错误信息
            let detailedError = response.data.error?.message || 'File upload failed';

            if (import.meta.env.DEV) {
              detailedError += ` (Response analysis: ${JSON.stringify(errorDetails)})`;
            }

            throw new Error(detailedError);
          }
        }

        // 🔧 FALLBACK: 如果完全没有response.data
        else {
          const noDataError = new Error('Server returned no data');
          if (import.meta.env.DEV) {
            console.error('❌ [ChatService] No response.data received:', response);
          }
          throw noDataError;
        }
      } catch (error: any) {
        console.error(`❌ Upload attempt ${attempt} failed for ${file.name}:`, error);

        // 🔧 ENHANCED: 详细的错误类型识别
        if (error.response) {
          const status = error.response.status;

          // 🔧 明确处理认证错误
          if (status === 401) {
            const authError = new Error(
              'Authentication required. Please login first to upload files.'
            );
            (authError as any).code = 'AUTH_REQUIRED';
            (authError as any).status = 401;
            throw authError;
          }

          // 🔧 明确处理其他HTTP错误
          if (status === 403) {
            throw new Error('You do not have permission to upload files.');
          }

          if (status === 413) {
            throw new Error('File size exceeds server limit.');
          }

          if (status === 422) {
            const validationError = new Error(
              error.response.data?.error?.message || 'File validation failed.'
            );
            (validationError as any).validationErrors = error.response.data?.error?.validation_errors;
            throw validationError;
          }

          // Don't retry on client errors (4xx) except for specific cases
          if (status >= 400 && status < 500 && status !== 429) {
            throw this.handleError(error);
          }

          // Retry on server errors (5xx) and rate limiting (429)
          if (attempt < uploadConfig.maxRetries && (status >= 500 || status === 429)) {
            const delay = uploadConfig.retryDelay * Math.pow(2, attempt - 1);
            console.log(`⏱️ Retrying in ${delay}ms... (Server error ${status})`);
            await new Promise(resolve => setTimeout(resolve, delay));
            continue;
          }
        }

        // 🔧 Network error handling with diagnostics
        if (!error.response) {
          const isNetworkHealthy = await checkNetworkHealth();

          if (!isNetworkHealthy) {
            console.warn(`🔍 Network diagnostic: API server appears to be down`);

            // 🔧 Progressive retry strategy
            if (attempt < uploadConfig.maxRetries) {
              const delay = uploadConfig.retryDelay * Math.pow(2, attempt - 1); // Exponential backoff
              console.log(`⏱️ Retrying in ${delay}ms... (Network issue detected)`);
              await new Promise(resolve => setTimeout(resolve, delay));
              continue;
            } else {
              // 🔧 Enhanced error message for network issues
              const networkError = new Error(
                `File upload failed: Unable to connect to server after ${uploadConfig.maxRetries} attempts. ` +
                `Please check your internet connection or try again later.`
              );
              (networkError as any).code = 'NETWORK_ERROR';
              (networkError as any).attempts = attempt;
              throw networkError;
            }
          }
        }

        // Final attempt failed
        if (attempt === uploadConfig.maxRetries) {
          throw this.handleError(error);
        }
      }
    }

    // This should never be reached, but TypeScript requires it
    throw new Error('Upload failed after all retry attempts');
  }
}

// 导出单例
export default new ChatService();