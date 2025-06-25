/**
 * 消息系统 Composable
 * 提供完整的消息管理功能
 */
import { reactive, computed, onMounted, onUnmounted, watch } from 'vue';
import ChatService from '@/services/ChatService';
import { useMessageBatcher } from './useMessageBatcher';
import type { ChatMessage, SendMessageRequest, UploadedFile, User } from '@/types/api';
import authStateManager from '@/utils/authStateManager';

export interface MessageSystemState {
  messages: ChatMessage[];
  loading: boolean;
  sending: boolean;
  editing: { messageId: number | null; content: string };
  hasMore: boolean;
  unreadCount: number;
  searchResults: {
    hits: ChatMessage[];
    total: number;
    loading: boolean;
    query: string;
  };
  typing: {
    users: { userId: number; userName: string; startedAt: number }[];
    lastUpdate: number;
  };
}

export function useMessageSystem(chatId: number | string) {
  // 确保 chatId 是数字类型
  const numericChatId = computed(() => typeof chatId === 'string' ? parseInt(chatId, 10) : chatId);
  // ================================
  // 状态管理
  // ================================

  const state = reactive<MessageSystemState>({
    messages: [],
    loading: false,
    sending: false,
    editing: { messageId: null, content: '' },
    hasMore: true,
    unreadCount: 0,
    searchResults: {
      hits: [],
      total: 0,
      loading: false,
      query: ''
    },
    typing: {
      users: [],
      lastUpdate: 0
    }
  });

  // 使用消息批处理器
  const messageBatcher = useMessageBatcher();

  // ================================
  // 计算属性
  // ================================

  const sortedMessages = computed(() => {
    return [...state.messages].sort((a, b) =>
      new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
    );
  });

  const hasUnreadMessages = computed(() => state.unreadCount > 0);

  const isTypingDisplayText = computed(() => {
    const users = state.typing.users;
    if (users.length === 0) return '';
    if (users.length === 1) return `${users[0].userName} is typing...`;
    if (users.length === 2) return `${users[0].userName} and ${users[1].userName} are typing...`;
    return `${users[0].userName} and ${users.length - 1} others are typing...`;
  });

  // ================================
  // 消息加载
  // ================================

  // 请求序列号，用于防止过期请求覆盖新请求
  let loadingRequestId = 0;

  /**
   * 加载聊天消息
   */
  const loadMessages = async (params: { limit?: number; before?: number } = {}) => {
    if (state.loading) {
      console.warn('⚠️ [useMessageSystem] Already loading messages, ignoring request');
      return;
    }

    const currentRequestId = ++loadingRequestId;
    state.loading = true;

    try {
      const { limit = 50, before } = params;
      const response = await ChatService.getChatMessages(numericChatId.value, {
        limit,
        offset: before ? undefined : 0,
        // 如果有before参数，添加到查询中
        ...(before && { before })
      });

      // 检查是否是最新的请求（防止过期响应覆盖新数据）
      if (currentRequestId !== loadingRequestId) {
        console.warn('⚠️ [useMessageSystem] Discarding outdated load response');
        return;
      }

      if (before) {
        // 向上加载更多（历史消息）- 防重复
        const newMessages = response.data.filter(newMsg =>
          !state.messages.some(existingMsg => existingMsg.id === newMsg.id)
        );
        state.messages.unshift(...newMessages);
      } else {
        // 初始加载或刷新
        state.messages = response.data;
      }

      state.hasMore = response.data.length === limit;

      // 添加到批处理器
      messageBatcher.addMessages(response.data);

    } catch (error) {
      // 只有当前请求才报告错误
      if (currentRequestId === loadingRequestId) {
        console.error('❌ [useMessageSystem] Load messages failed:', error);
        throw error;
      }
    } finally {
      // 只有当前请求才清除loading状态
      if (currentRequestId === loadingRequestId) {
        state.loading = false;
      }
    }
  };

  /**
   * 加载更多历史消息
   */
  const loadMoreMessages = async () => {
    if (!state.hasMore || state.loading) return;

    const oldestMessage = state.messages[0];
    if (oldestMessage && typeof oldestMessage.id === 'number') {
      await loadMessages({ before: oldestMessage.id });
    }
  };

  // ================================
  // 消息发送
  // ================================

  /**
   * 发送消息
   */
  const sendMessage = async (messageData: SendMessageRequest): Promise<ChatMessage | null> => {
    // 防止重复发送 - 使用返回而非抛出错误
    if (state.sending) {
      console.warn('⚠️ [useMessageSystem] Message already being sent, ignoring duplicate request');
      return null;
    }

    state.sending = true;
    try {
      const sentMessage = await ChatService.sendMessage(numericChatId.value, messageData);

      // 检查消息是否已存在（防止重复添加）
      const existingMessage = state.messages.find(m => m.id === sentMessage.id);
      if (!existingMessage) {
        state.messages.push(sentMessage);
        messageBatcher.addMessage(sentMessage);
      }

      return sentMessage;
    } catch (error) {
      console.error('❌ [useMessageSystem] Send message failed:', error);
      throw error;
    } finally {
      state.sending = false;
    }
  };

  /**
   * 快速发送文本消息
   */
  const sendTextMessage = async (content: string, options: {
    reply_to?: number;
    mentions?: number[];
    priority?: 'low' | 'normal' | 'high' | 'urgent';
    is_important?: boolean;
  } = {}): Promise<ChatMessage | null> => {
    const result = await sendMessage({
      content: content.trim(),
      reply_to: options.reply_to,
      mentions: options.mentions || [],
      priority: options.priority || 'normal',
      is_important: options.is_important || false
    });

    return result;
  };

  /**
   * 上传并发送文件
   */
  const sendFiles = async (files: { file: File, url: string, name: string, size: number, type: string }[], options: {
    reply_to?: number;
    mentions?: number[];
  } = {}) => {
    const currentUser = authStateManager.getAuthState().user as User | null;

    // 1. 创建乐观的临时消息
    const tempMessages: ChatMessage[] = files.map(fileInfo => {
      const tempId = `temp_${Date.now()}_${Math.random()}`;
      const tempFile: UploadedFile = {
        id: tempId,
        filename: fileInfo.name,
        url: fileInfo.url, // 本地 blob URL
        mime_type: fileInfo.type,
        size: fileInfo.size,
        created_at: new Date().toISOString()
      };

      const tempMsg: ChatMessage = {
        id: tempId,
        temp_id: tempId,
        chat_id: numericChatId.value,
        sender_id: currentUser?.id || 0,
        sender: currentUser || undefined,
        content: '',
        files: [tempFile],
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        message_type: 'file',
        status: 'sending',
        // 补全缺失的属性
        mentions: [],
        priority: 'normal',
        is_important: false,
        is_scheduled: false,
      };
      return tempMsg;
    });

    state.messages.push(...tempMessages);

    // 2. 依次上传每个文件并更新消息
    for (let i = 0; i < tempMessages.length; i++) {
      const tempMessage = tempMessages[i];
      const fileInfo = files[i];

      try {
        const uploadedFile: UploadedFile = await ChatService.uploadFile(fileInfo.file, (_progress: number) => {
          // TODO: 更新上传进度
        });

        const finalMessage = await sendMessage({
          content: '',
          files: [uploadedFile.id],
          reply_to: options.reply_to,
          mentions: options.mentions,
        });

        // 4. 用最终消息替换临时消息，但保留前端更丰富的文件信息
        if (finalMessage) {
          const index = state.messages.findIndex(m => m.id === tempMessage.id);
          if (index !== -1) {
            // "Enrich" the final message with the optimistic file data
            const enrichedMessage = {
              ...finalMessage,
              files: tempMessage.files, // Use the complete file object from the temp message
            };
            state.messages.splice(index, 1, enrichedMessage);
          }
        }
      } catch (error) {
        console.error('❌ [useMessageSystem] File send failed:', error);
        const index = state.messages.findIndex(m => m.id === tempMessage.id);
        if (index !== -1) {
          state.messages[index].status = 'failed';
        }
      }
    }
  };

  // ================================
  // 消息编辑
  // ================================

  /**
   * 开始编辑消息
   */
  const startEditMessage = (messageId: number, currentContent: string) => {
    state.editing.messageId = messageId;
    state.editing.content = currentContent;
  };

  /**
   * 取消编辑
   */
  const cancelEditMessage = () => {
    state.editing.messageId = null;
    state.editing.content = '';
  };

  /**
   * 保存编辑的消息
   */
  const saveEditMessage = async (): Promise<void> => {
    if (!state.editing.messageId || !state.editing.content.trim()) return;

    try {
      const updatedMessage = await ChatService.editMessage(
        numericChatId.value,
        state.editing.messageId,
        state.editing.content.trim()
      );

      // 更新本地消息
      const index = state.messages.findIndex(m => m.id === state.editing.messageId);
      if (index !== -1) {
        state.messages[index] = updatedMessage;
      }

      cancelEditMessage();
    } catch (error) {
      console.error('❌ [useMessageSystem] Edit message failed:', error);
      throw error;
    }
  };

  // ================================
  // 消息删除
  // ================================

  /**
   * 删除消息
   */
  const deleteMessage = async (messageId: number): Promise<void> => {
    try {
      await ChatService.deleteMessage(numericChatId.value, messageId);

      // 从本地移除
      const index = state.messages.findIndex(m => m.id === messageId);
      if (index !== -1) {
        state.messages.splice(index, 1);
      }
    } catch (error) {
      console.error('❌ [useMessageSystem] Delete message failed:', error);
      throw error;
    }
  };

  // ================================
  // 未读消息管理
  // ================================

  /**
   * 更新未读计数
   */
  const updateUnreadCount = async () => {
    try {
      const unreadData = await ChatService.getUnreadCount(numericChatId.value);
      if (unreadData.length > 0) {
        const newCount = unreadData[0].unreadCount || 0;
        // 验证未读计数的合理性
        if (newCount >= 0 && newCount !== state.unreadCount) {
          state.unreadCount = newCount;
        }
      }
    } catch (error) {
      console.warn('⚠️ [useMessageSystem] Update unread count failed:', error);
      // 可以考虑触发错误恢复机制或用户通知
    }
  };

  /**
   * 标记消息为已读
   */
  const markAsRead = async (messageId?: number) => {
    try {
      await ChatService.markAsRead(numericChatId.value, messageId);
      if (!messageId) {
        state.unreadCount = 0;
      }
    } catch (error) {
      console.warn('⚠️ [useMessageSystem] Mark as read failed:', error);
    }
  };

  /**
   * 批量标记消息为已读
   */
  const markMessagesAsRead = async (messageIds: number[]) => {
    try {
      await ChatService.markMessagesAsRead(numericChatId.value, messageIds);
      state.unreadCount = Math.max(0, state.unreadCount - messageIds.length);
    } catch (error) {
      console.warn('⚠️ [useMessageSystem] Batch mark as read failed:', error);
    }
  };

  // ================================
  // 消息搜索
  // ================================

  /**
   * 搜索消息
   */
  const searchMessages = async (query: string, options: {
    limit?: number;
    offset?: number;
    sort?: 'relevance' | 'date_desc' | 'date_asc';
  } = {}) => {
    if (!query.trim()) {
      state.searchResults = { hits: [], total: 0, loading: false, query: '' };
      return;
    }

    state.searchResults.loading = true;
    state.searchResults.query = query;

    try {
      const results = await ChatService.searchMessages(numericChatId.value, query, options);
      state.searchResults.hits = results.hits;
      state.searchResults.total = results.total;
    } catch (error) {
      console.error('❌ [useMessageSystem] Search messages failed:', error);
      state.searchResults.hits = [];
      state.searchResults.total = 0;
    } finally {
      state.searchResults.loading = false;
    }
  };

  /**
   * 清空搜索结果
   */
  const clearSearchResults = () => {
    state.searchResults = { hits: [], total: 0, loading: false, query: '' };
  };

  // ================================
  // 输入状态管理
  // ================================

  let typingTimer: number | null = null;

  /**
   * 发送开始输入状态
   */
  const startTyping = async () => {
    try {
      await ChatService.startTyping(numericChatId.value);

      // 3秒后自动停止
      if (typingTimer) clearTimeout(typingTimer);
      typingTimer = window.setTimeout(() => {
        stopTyping();
      }, 3000);
    } catch (error) {
      console.warn('⚠️ [useMessageSystem] Start typing failed:', error);
    }
  };

  /**
   * 停止输入状态
   */
  const stopTyping = async () => {
    try {
      if (typingTimer) {
        clearTimeout(typingTimer);
        typingTimer = null;
      }
      await ChatService.stopTyping(numericChatId.value);
    } catch (error) {
      console.warn('⚠️ [useMessageSystem] Stop typing failed:', error);
    }
  };

  /**
   * 更新正在输入的用户列表
   */
  const updateTypingUsers = async () => {
    try {
      const users = await ChatService.getTypingUsers(numericChatId.value);
      state.typing.users = users;
      state.typing.lastUpdate = Date.now();
    } catch (error) {
      console.warn('⚠️ [useMessageSystem] Update typing users failed:', error);
    }
  };

  // ================================
  // 生命周期管理
  // ================================

  let typingUsersInterval: number | null = null;

  onMounted(() => {
    // 定期更新正在输入的用户 - 减少频率以降低服务器负载
    typingUsersInterval = window.setInterval(updateTypingUsers, 15000); // 从2秒改为15秒
  });

  // 清理函数
  const cleanup = () => {
    // 清理所有计时器
    if (typingTimer) {
      clearTimeout(typingTimer);
      typingTimer = null;
    }
    if (typingUsersInterval) {
      clearInterval(typingUsersInterval);
      typingUsersInterval = null;
    }

    // 重置请求序列号
    loadingRequestId = 0;

    // 清理批处理器
    if (messageBatcher.cleanup) {
      messageBatcher.cleanup();
    }

    // 停止任何正在进行的输入状态
    if (state.typing.users.length > 0) {
      stopTyping().catch(console.warn);
    }
  };

  onUnmounted(cleanup);

  // ================================
  // 响应式监听
  // ================================

  // 监听聊天ID变化，重新加载消息
  watch(() => numericChatId.value, async (newChatId) => {
    if (newChatId && !isNaN(newChatId)) {
      state.messages = [];
      state.unreadCount = 0;
      clearSearchResults();
      await loadMessages();
      await updateUnreadCount();
    }
  }, { immediate: true });

  // ================================
  // 返回接口
  // ================================

  return {
    // 状态
    state: state as Readonly<MessageSystemState>,

    // 计算属性
    sortedMessages,
    hasUnreadMessages,
    isTypingDisplayText,

    // 消息加载
    loadMessages,
    loadMoreMessages,

    // 消息发送
    sendMessage,
    sendTextMessage,
    sendFiles,

    // 消息编辑
    startEditMessage,
    cancelEditMessage,
    saveEditMessage,

    // 消息删除
    deleteMessage,

    // 未读管理
    updateUnreadCount,
    markAsRead,
    markMessagesAsRead,

    // 搜索
    searchMessages,
    clearSearchResults,

    // 输入状态
    startTyping,
    stopTyping,
    updateTypingUsers,

    // 批处理器访问
    messageBatcher
  };
}

export type UseMessageSystemReturn = ReturnType<typeof useMessageSystem>;