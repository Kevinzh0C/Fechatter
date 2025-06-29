/**
 * Read Receipts Service
 * 处理消息已读回执相关功能
 */
import api from './api';

class ReadReceiptsService {
  /**
   * 获取消息的详细已读回执
   * @param {number} messageId - 消息ID
   * @returns {Promise<Array>} 已读回执列表
   */
  async getDetailedReceipts(messageId) {
    try {
      const response = await api.get(`/messages/${messageId}/receipts/detailed`);
      const data = response.data?.data || response.data;
      return Array.isArray(data) ? data : [];
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to fetch detailed receipts:', error);
      return [];
    }

  /**
   * 获取消息的基础已读回执
   * @param {number} messageId - 消息ID
   * @returns {Promise<Array>} 已读回执列表
   */
  async getBasicReceipts(messageId) {
    try {
      const response = await api.get(`/messages/${messageId}/receipts`);
      const data = response.data?.data || response.data;
      return Array.isArray(data) ? data : [];
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to fetch basic receipts:', error);
      return [];
    }

  /**
   * 标记消息为已读（增强版）
   * @param {number} chatId - 聊天ID
   * @param {number} messageId - 消息ID
   * @returns {Promise<boolean>} 操作是否成功
   */
  async markMessageReadEnhanced(chatId, messageId) {
    try {
      await api.post(`/chat/${chatId}/messages/${messageId}/read/enhanced`);
      return true;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to mark message as read (enhanced):', error);
      return false;
    }

  /**
   * 批量标记消息为已读
   * @param {number} chatId - 聊天ID
   * @param {Array<number>} messageIds - 消息ID列表
   * @returns {Promise<boolean>} 操作是否成功
   */
  async markMessagesRead(chatId, messageIds) {
    try {
      await api.post(`/chat/${chatId}/messages/read`, {
        message_ids: messageIds
      });
      return true;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to mark messages as read:', error);
      return false;
    }

  /**
   * 格式化已读回执显示文本
   * @param {Array} receipts - 已读回执列表
   * @param {number} totalMembers - 聊天总成员数
   * @returns {Object} 格式化后的显示信息
   */
  formatReceiptStatus(receipts, totalMembers) {
    if (!receipts || receipts.length === 0) {
      return {
        text: 'Sent',
        detail: 'Message sent',
        icon: 'sent',
        count: 0
      };
    }

    const readReceipts = receipts.filter(r => r.status === 'read');
    const deliveredReceipts = receipts.filter(r => r.status === 'delivered');

    if (readReceipts.length === totalMembers - 1) { // 排除发送者
      return {
        text: 'Read by all',
        detail: `Read by all ${readReceipts.length} members`,
        icon: 'read-all',
        count: readReceipts.length
      };
    } else if (readReceipts.length > 0) {
      return {
        text: `Read by ${readReceipts.length}`,
        detail: `Read by ${readReceipts.length} of ${totalMembers - 1} members`,
        icon: 'read-some',
        count: readReceipts.length
      };
    } else if (deliveredReceipts.length > 0) {
      return {
        text: 'Delivered',
        detail: `Delivered to ${deliveredReceipts.length} members`,
        icon: 'delivered',
        count: deliveredReceipts.length
      };
    } else {
      return {
        text: 'Sent',
        detail: 'Message sent',
        icon: 'sent',
        count: 0
      };
    }

  /**
   * 生成已读回执的详细信息
   * @param {Array} receipts - 已读回执列表
   * @returns {Object} 详细信息
   */
  generateReceiptDetails(receipts) {
    const readUsers = [];
    const deliveredUsers = [];
    const unreadUsers = [];

    receipts.forEach(receipt => {
      const userInfo = {
        id: receipt.user_id,
        username: receipt.username,
        fullname: receipt.fullname,
        timestamp: receipt.timestamp
      };

      if (receipt.status === 'read') {
        readUsers.push(userInfo);
      } else if (receipt.status === 'delivered') {
        deliveredUsers.push(userInfo);
      }
    });

    // 按时间排序
    readUsers.sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp));
    deliveredUsers.sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp));

    return {
      read: readUsers,
      delivered: deliveredUsers,
      unread: unreadUsers,
      summary: {
        readCount: readUsers.length,
        deliveredCount: deliveredUsers.length,
        unreadCount: unreadUsers.length
      }
    };
  }

  /**
   * 检查消息是否需要显示已读回执
   * @param {Object} message - 消息对象
   * @param {Object} currentUser - 当前用户
   * @returns {boolean} 是否显示已读回执
   */
  shouldShowReceipts(message, currentUser) {
    // 只有发送者才能看到已读回执
    return message.sender_id === currentUser?.id;
  }

  /**
   * 获取已读回执的图标
   * @param {string} status - 状态
   * @returns {string} 图标名称
   */
  getReceiptIcon(status) {
    const iconMap = {
      'sent': '📤',
      'delivered': '📩',
      'read-some': '👁️',
      'read-all': ''
    };
    return iconMap[status] || '📤';
  }

  /**
   * 实时更新已读回执
   * @param {number} messageId - 消息ID
   * @param {Object} receipt - 新的已读回执
   * @param {Function} callback - 更新回调
   */
  updateReceiptRealtime(messageId, receipt, callback) {
    // 这个方法会被 SSE 事件调用
    if (typeof callback === 'function') {
      callback(messageId, receipt);
    }

  /**
   * 格式化时间显示
   * @param {string} timestamp - 时间戳
   * @returns {string} 格式化后的时间
   */
  formatReceiptTime(timestamp) {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now - date;
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) {
      return 'Just now';
    } else if (diffMins < 60) {
      return `${diffMins}m ago`;
    } else if (diffHours < 24) {
      return `${diffHours}h ago`;
    } else if (diffDays < 7) {
      return `${diffDays}d ago`;
    } else {
      return date.toLocaleDateString();
    }

export default new ReadReceiptsService();