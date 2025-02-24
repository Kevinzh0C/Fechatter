/**
 * Mentions Service
 * 处理 @提及 相关的功能
 */
import api from './api';

class MentionsService {
  /**
   * 获取未读提及
   * @returns {Promise<Array>} 未读提及列表
   */
  async getUnreadMentions() {
    try {
      const response = await api.get('/mentions/unread');
      const data = response.data?.data || response.data;
      return Array.isArray(data) ? data : [];
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to fetch unread mentions:', error);
      return [];
    }

  /**
   * 获取消息的提及信息
   * @param {number} messageId - 消息ID
   * @returns {Promise<Array>} 提及列表
   */
  async getMessageMentions(messageId) {
    try {
      const response = await api.get(`/messages/${messageId}/mentions`);
      const data = response.data?.data || response.data;
      return Array.isArray(data) ? data : [];
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to fetch message mentions:', error);
      return [];
    }

  /**
   * 从消息内容中提取提及
   * @param {string} content - 消息内容
   * @returns {Array<Object>} 提及信息
   */
  extractMentions(content) {
    const mentions = [];
    
    // 匹配 @username 模式
    const userMentionRegex = /@(\w+)/g;
    let match;
    
    while ((match = userMentionRegex.exec(content)) !== null) {
      mentions.push({
        type: 'user',
        username: match[1],
        position: match.index,
        length: match[0].length
      });
    
    // 匹配 @everyone
    if (content.includes('@everyone')) {
      mentions.push({
        type: 'everyone',
        username: 'everyone',
        position: content.indexOf('@everyone'),
        length: 9
      });
    
    // 匹配 @here
    if (content.includes('@here')) {
      mentions.push({
        type: 'here',
        username: 'here',
        position: content.indexOf('@here'),
        length: 5
      });
    
    return mentions;
  }

  /**
   * 高亮消息中的提及
   * @param {string} content - 消息内容
   * @param {string} currentUsername - 当前用户名
   * @returns {string} 高亮后的HTML内容
   */
  highlightMentions(content, currentUsername = '') {
    let highlightedContent = content;
    
    // 高亮 @everyone
    highlightedContent = highlightedContent.replace(
      /@everyone/g,
      '<span class="mention mention-everyone">@everyone</span>'
    );
    
    // 高亮 @here
    highlightedContent = highlightedContent.replace(
      /@here/g,
      '<span class="mention mention-here">@here</span>'
    );
    
    // 高亮 @username
    highlightedContent = highlightedContent.replace(
      /@(\w+)/g,
      (match, username) => {
        const isCurrentUser = username === currentUsername;
        const className = isCurrentUser ? 'mention mention-self' : 'mention mention-user';
        return `<span class="${className}">@${username}</span>`;
      }
    );
    
    return highlightedContent;
  }

  /**
   * 检查消息是否提及了当前用户
   * @param {string} content - 消息内容
   * @param {string} currentUsername - 当前用户名
   * @returns {boolean} 是否被提及
   */
  isMentioned(content, currentUsername) {
    if (!content || !currentUsername) return false;
    
    // 检查 @everyone 或 @here
    if (content.includes('@everyone') || content.includes('@here')) {
      return true;
    }
    
    // 检查 @username
    const userMentionRegex = new RegExp(`@${currentUsername}\\b`, 'i');
    return userMentionRegex.test(content);
  }

  /**
   * 获取提及建议（用于输入时的自动完成）
   * @param {string} query - 搜索查询
   * @param {Array} availableUsers - 可用用户列表
   * @returns {Array} 提及建议
   */
  getMentionSuggestions(query, availableUsers = []) {
    const suggestions = [];
    
    // 添加特殊提及
    if ('everyone'.startsWith(query.toLowerCase())) {
      suggestions.push({
        type: 'everyone',
        username: 'everyone',
        display: '@everyone',
        description: 'Notify everyone in this chat'
      });
    
    if ('here'.startsWith(query.toLowerCase())) {
      suggestions.push({
        type: 'here',
        username: 'here',
        display: '@here',
        description: 'Notify online members'
      });
    
    // 添加用户提及
    const filteredUsers = availableUsers.filter(user => 
      user.username?.toLowerCase().includes(query.toLowerCase()) ||
      user.fullname?.toLowerCase().includes(query.toLowerCase())
    );
    
    filteredUsers.forEach(user => {
      suggestions.push({
        type: 'user',
        username: user.username,
        display: `@${user.username}`,
        fullname: user.fullname,
        avatar: user.avatar_url,
        description: user.fullname
      });
    });
    
    return suggestions.slice(0, 10); // 限制建议数量
  }

  /**
   * 处理提及输入
   * @param {string} text - 当前输入文本
   * @param {number} cursorPosition - 光标位置
   * @returns {Object|null} 提及上下文
   */
  getMentionContext(text, cursorPosition) {
    const beforeCursor = text.substring(0, cursorPosition);
    const mentionMatch = beforeCursor.match(/@(\w*)$/);
    
    if (mentionMatch) {
      return {
        query: mentionMatch[1],
        startIndex: mentionMatch.index,
        prefix: '@',
        isActive: true
      };
    }
    
    return null;
  }

  /**
   * 插入提及到文本中
   * @param {string} text - 原始文本
   * @param {Object} mention - 提及对象
   * @param {Object} context - 提及上下文
   * @returns {Object} 新文本和光标位置
   */
  insertMention(text, mention, context) {
    const beforeMention = text.substring(0, context.startIndex);
    const afterMention = text.substring(context.startIndex + context.query.length + 1);
    
    const mentionText = mention.display;
    const newText = beforeMention + mentionText + ' ' + afterMention;
    const newCursorPosition = beforeMention.length + mentionText.length + 1;
    
    return {
      text: newText,
      cursorPosition: newCursorPosition
    };
  }

export default new MentionsService();