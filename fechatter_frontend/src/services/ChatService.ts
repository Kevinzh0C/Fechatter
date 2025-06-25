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
  ApiResponse
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
      const response = await api.get<ApiResponse<Chat>>(`/api/chat/${chatId}`);
      
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
      const response = await api.put<ApiResponse<Chat>>(`/api/chat/${chatId}`, updates);
      
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
      const response = await api.delete<ApiResponse<void>>(`/api/chat/${chatId}`);
      
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
      const response = await api.get<ApiResponse<ChatMember[]>>(`/api/chat/${chatId}/members`);
      
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
      const response = await api.post<ApiResponse<ChatMember[]>>(`/api/chat/${chatId}/members`, memberData);
      
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
      const response = await api.delete<ApiResponse<void>>(`/api/chat/${chatId}/members/${userId}`);
      
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
      const response = await api.post<ApiResponse<ChatMember>>(`/api/chat/${chatId}/join`);
      
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
      const response = await api.post<ApiResponse<void>>(`/api/chat/${chatId}/leave`);
      
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

      const url = `/api/chat/${chatId}/messages${queryParams.toString() ? '?' + queryParams.toString() : ''}`;
      const response = await api.get<ApiResponse<ChatMessage[]>>(url);
      
      if (response.data.success && response.data.data) {
        // 从数据库返回的是消息数组，需要包装成分页响应格式
        const messages = response.data.data;
        
        return {
          data: messages,
          pagination: {
            page: Math.floor((params.offset || 0) / (params.limit || 50)) + 1,
            limit: params.limit || 50,
            total: messages.length, // 这里需要后端提供总数
            has_more: messages.length === (params.limit || 50) // 如果返回的数量等于限制，可能还有更多
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
      // 基于数据库结构的请求数据
      const requestPayload = {
        content: messageData.content || '',
        files: Array.isArray(messageData.files) ? messageData.files.map(f => 
          typeof f === 'string' ? f : f.name // 如果是文件对象，使用文件名
        ) : [],
        reply_to: messageData.reply_to, // 使用正确的字段名
        mentions: messageData.mentions || [],
        idempotency_key: messageData.idempotency_key || this.generateIdempotencyKey(),
        priority: messageData.priority || 'normal',
        is_important: messageData.is_important || false,
        scheduled_for: messageData.scheduled_for
      };

      // 如果包含 File 对象，需要先上传文件
      if (messageData.files && messageData.files.some(f => f instanceof File)) {
        const formData = new FormData();
        formData.append('content', requestPayload.content);
        
        if (requestPayload.reply_to) {
          formData.append('reply_to', requestPayload.reply_to.toString());
        }
        
        if (requestPayload.mentions.length > 0) {
          formData.append('mentions', JSON.stringify(requestPayload.mentions));
        }
        
        formData.append('idempotency_key', requestPayload.idempotency_key);
        formData.append('priority', requestPayload.priority);
        formData.append('is_important', requestPayload.is_important.toString());
        
        // 添加文件
        messageData.files.forEach((file, index) => {
          if (file instanceof File) {
            formData.append(`files[${index}]`, file);
          }
        });

        const response = await api.post<ApiResponse<ChatMessage>>(
          `/api/chat/${chatId}/messages`, 
          formData,
          { 
            headers: { 'Content-Type': 'multipart/form-data' }
          }
        );
        
        if (response.data.success && response.data.data) {
          return response.data.data;
        }
      } else {
        // 纯文本消息或已上传的文件路径
        const response = await api.post<ApiResponse<ChatMessage>>(
          `/api/chat/${chatId}/messages`, 
          requestPayload
        );
        
        if (response.data.success && response.data.data) {
          return response.data.data;
        }
      }
      
      throw new Error('Failed to send message');
    } catch (error) {
      console.error(`❌ [ChatService] Send message failed for chat ${chatId}:`, error);
      throw this.handleError(error);
    }
  }

  /**
   * 生成幂等性密钥
   */
  private generateIdempotencyKey(): string {
    return `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * 编辑消息
   */
  async editMessage(chatId: number, messageId: number, content: string): Promise<ChatMessage> {
    try {
      const response = await api.put<ApiResponse<ChatMessage>>(
        `/api/chat/${chatId}/messages/${messageId}`, 
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
      const response = await api.delete<ApiResponse<void>>(`/api/chat/${chatId}/messages/${messageId}`);
      
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
        ? `/api/chat/${chatId}/mark-read/${messageId}`
        : `/api/chat/${chatId}/mark-read`;
      
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
      const response = await api.post<ApiResponse<void>>(`/api/chat/${chatId}/typing`, {
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
        `/api/workspace/chats/search?q=${encodeURIComponent(query)}&limit=${limit}`
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
      const response = await api.post<ApiResponse<Chat>>('/workspace/chats', {
        name: '', // DM不需要名称
        chat_type: 'direct',
        is_public: false,
        member_ids: [userId]
      });
      
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
      const url = chatId ? `/api/chat/${chatId}/unread` : `/api/chats/unread`;
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
        `/api/chat/${chatId}/messages/read`,
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
        `/api/chat/${chatId}/messages/search?q=${encodeURIComponent(query)}&limit=${limit}&offset=${offset}&sort=${sort}`
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
        `/api/search/suggestions?q=${encodeURIComponent(query)}&limit=${limit}`
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
      await api.post<ApiResponse<void>>(`/api/chat/${chatId}/typing/start`);
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
      await api.post<ApiResponse<void>>(`/api/chat/${chatId}/typing/stop`);
    } catch (error) {
      console.warn(`⚠️ [ChatService] Stop typing failed for chat ${chatId}:`, error);
    }
  }

  /**
   * 获取正在输入的用户
   */
  async getTypingUsers(chatId: number): Promise<{ userId: number; userName: string; startedAt: number }[]> {
    try {
      const response = await api.get<ApiResponse<any>>(`/api/chat/${chatId}/typing/users`);
      
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
      const response = await api.get<ApiResponse<any>>(`/api/messages/${messageId}/receipts`);
      
      if (response.data.success && response.data.data) {
        return response.data.data;
      }
      
      return [];
    } catch (error) {
      console.warn(`⚠️ [ChatService] Get message receipts failed for message ${messageId}:`, error);
      return [];
    }
  }
}

// 导出单例
export default new ChatService();