/**
 * Minimalist Message Persistence Solution
 * 符合奥卡姆剃刀原则：简单、有效、可维护
 * 
 * 解决消息刷新后丢失的核心问题，用最少的代码实现最大的效果
 */

class MinimalMessagePersistence {
  constructor() {
    this.STORAGE_KEY = 'fechatter_messages';
    this.MAX_CACHE_SIZE = 50; // 每个聊天最多缓存50条消息
    this.MAX_CHATS = 10;      // 最多缓存10个聊天

    console.log('💾 Minimal Message Persistence initialized');
  }

  /**
   * Save message to localStorage
   * 保存单条消息到localStorage
   */
  saveMessage(chatId, message) {
    if (!chatId || !message) return;

    try {
      const allMessages = this.getAllMessages();
      if (!allMessages[chatId]) allMessages[chatId] = [];

      // 避免重复保存相同消息
      const existingIndex = allMessages[chatId].findIndex(m =>
        m.id === message.id ||
        (m.temp_id && m.temp_id === message.temp_id)
      );

      if (existingIndex >= 0) {
        // 更新现有消息（处理乐观更新替换）
        allMessages[chatId][existingIndex] = message;
      } else {
        // 添加新消息
        allMessages[chatId].push(message);
      }

      // 保持缓存大小在合理范围
      if (allMessages[chatId].length > this.MAX_CACHE_SIZE) {
        allMessages[chatId] = allMessages[chatId].slice(-this.MAX_CACHE_SIZE);
      }

      // 保存到localStorage
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(allMessages));

    } catch (error) {
      // 静默失败，不影响正常功能
      console.warn('Failed to save message to localStorage:', error);
    }
  }

  /**
   * Save multiple messages for a chat
   * 批量保存聊天消息
   */
  saveMessages(chatId, messages) {
    if (!chatId || !Array.isArray(messages) || messages.length === 0) return;

    try {
      const allMessages = this.getAllMessages();

      // 替换整个聊天的消息
      allMessages[chatId] = messages.slice(-this.MAX_CACHE_SIZE);

      // 保存到localStorage
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(allMessages));

      console.log(`💾 Saved ${messages.length} messages for chat ${chatId}`);

    } catch (error) {
      console.warn('Failed to save messages to localStorage:', error);
    }
  }

  /**
   * Get messages for a specific chat
   * 获取特定聊天的消息
   */
  getMessages(chatId) {
    if (!chatId) return [];

    try {
      const allMessages = this.getAllMessages();
      const messages = allMessages[chatId] || [];

      if (messages.length > 0) {
        console.log(`📦 Restored ${messages.length} messages for chat ${chatId}`);
      }

      return messages;
    } catch (error) {
      console.warn('Failed to get messages from localStorage:', error);
      return [];
    }
  }

  /**
   * Get all stored messages
   * 获取所有存储的消息
   */
  getAllMessages() {
    try {
      const stored = localStorage.getItem(this.STORAGE_KEY);
      return stored ? JSON.parse(stored) : {};
    } catch (error) {
      console.warn('Failed to parse stored messages:', error);
      return {};
    }
  }

  /**
   * Remove a specific message
   * 删除特定消息（用于失败的乐观更新）
   */
  removeMessage(chatId, messageId) {
    if (!chatId || !messageId) return;

    try {
      const allMessages = this.getAllMessages();
      if (!allMessages[chatId]) return;

      allMessages[chatId] = allMessages[chatId].filter(m =>
        m.id !== messageId && m.temp_id !== messageId
      );

      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(allMessages));

    } catch (error) {
      console.warn('Failed to remove message from localStorage:', error);
    }
  }

  /**
   * Clean up old and excessive cache
   * 清理过期和过多的缓存
   */
  cleanup() {
    try {
      const allMessages = this.getAllMessages();
      const chatIds = Object.keys(allMessages);

      // 如果缓存的聊天数量超过限制，只保留最近的
      if (chatIds.length > this.MAX_CHATS) {
        console.log(`🧹 Cleaning up message cache: ${chatIds.length} chats -> ${this.MAX_CHATS} chats`);

        // 简单策略：保留前MAX_CHATS个聊天
        const toKeep = chatIds.slice(0, this.MAX_CHATS);
        const cleaned = {};

        toKeep.forEach(chatId => {
          cleaned[chatId] = allMessages[chatId];
        });

        localStorage.setItem(this.STORAGE_KEY, JSON.stringify(cleaned));
      }

      // 清理每个聊天中过多的消息
      let hasChanges = false;
      chatIds.forEach(chatId => {
        if (allMessages[chatId] && allMessages[chatId].length > this.MAX_CACHE_SIZE) {
          allMessages[chatId] = allMessages[chatId].slice(-this.MAX_CACHE_SIZE);
          hasChanges = true;
        }
      });

      if (hasChanges) {
        localStorage.setItem(this.STORAGE_KEY, JSON.stringify(allMessages));
      }

    } catch (error) {
      console.warn('Failed to cleanup message cache:', error);
    }
  }

  /**
   * Clear all cached messages
   * 清空所有缓存消息
   */
  clear() {
    try {
      localStorage.removeItem(this.STORAGE_KEY);
      console.log('🗑️ All cached messages cleared');
    } catch (error) {
      console.warn('Failed to clear message cache:', error);
    }
  }

  /**
   * Get cache statistics
   * 获取缓存统计信息
   */
  getStats() {
    try {
      const allMessages = this.getAllMessages();
      const chats = Object.keys(allMessages);
      const totalMessages = chats.reduce((sum, chatId) => {
        return sum + (allMessages[chatId]?.length || 0);
      }, 0);

      // 计算存储大小（粗略估算）
      const storageSize = JSON.stringify(allMessages).length;

      return {
        totalChats: chats.length,
        totalMessages,
        storageSize: `${Math.round(storageSize / 1024)}KB`,
        chats: chats.map(chatId => ({
          chatId,
          messageCount: allMessages[chatId]?.length || 0
        }))
      };
    } catch (error) {
      return {
        totalChats: 0,
        totalMessages: 0,
        storageSize: '0KB',
        chats: []
      };
    }
  }
}

// 创建单例实例
const minimalMessagePersistence = new MinimalMessagePersistence();

// 暴露到window对象用于调试
if (typeof window !== 'undefined') {
  window.msgPersist = {
    save: (chatId, message) => minimalMessagePersistence.saveMessage(chatId, message),
    get: (chatId) => minimalMessagePersistence.getMessages(chatId),
    cleanup: () => minimalMessagePersistence.cleanup(),
    clear: () => minimalMessagePersistence.clear(),
    stats: () => minimalMessagePersistence.getStats()
  };

  console.log('💾 Message Persistence available at window.msgPersist');
}

export default minimalMessagePersistence; 