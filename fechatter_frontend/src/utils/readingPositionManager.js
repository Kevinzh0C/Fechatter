/**
 * Reading Position Manager
 * 管理用户在每个频道的阅读位置和访问历史
 */

export class ReadingPositionManager {
  constructor() {
    this.STORAGE_KEY = 'fechatter_reading_positions';
    this.VISIT_HISTORY_KEY = 'fechatter_channel_visits';
    this.SESSION_KEY = 'fechatter_session_channels';
    
    // 当前会话访问的频道（内存中）
    this.sessionChannels = new Set();
    
    // 从 sessionStorage 恢复会话数据
    this.restoreSessionData();
  }

  /**
   * 恢复会话数据
   */
  restoreSessionData() {
    try {
      const sessionData = sessionStorage.getItem(this.SESSION_KEY);
      if (sessionData) {
        const channels = JSON.parse(sessionData);
        this.sessionChannels = new Set(channels);
      }
    } catch (error) {
      console.warn('Failed to restore session data:', error);
    }
  }

  /**
   * 保存会话数据
   */
  saveSessionData() {
    try {
      sessionStorage.setItem(this.SESSION_KEY, JSON.stringify([...this.sessionChannels]));
    } catch (error) {
      console.warn('Failed to save session data:', error);
    }
  }

  /**
   * 检查频道是否在当前会话中已访问过
   * @param {number|string} channelId - 频道ID
   * @returns {boolean} 是否已访问过
   */
  isChannelVisitedInSession(channelId) {
    return this.sessionChannels.has(String(channelId));
  }

  /**
   * 标记频道为已访问
   * @param {number|string} channelId - 频道ID
   */
  markChannelAsVisited(channelId) {
    this.sessionChannels.add(String(channelId));
    this.saveSessionData();
    
    // 同时更新持久化的访问历史
    this.updateVisitHistory(channelId);
  }

  /**
   * 更新访问历史（localStorage，跨会话持久化）
   * @param {number|string} channelId - 频道ID
   */
  updateVisitHistory(channelId) {
    try {
      let history = this.getVisitHistory();
      history[String(channelId)] = {
        lastVisit: Date.now(),
        visitCount: (history[String(channelId)]?.visitCount || 0) + 1
      };
      
      localStorage.setItem(this.VISIT_HISTORY_KEY, JSON.stringify(history));
    } catch (error) {
      console.warn('Failed to update visit history:', error);
    }
  }

  /**
   * 获取访问历史
   * @returns {Object} 访问历史对象
   */
  getVisitHistory() {
    try {
      const history = localStorage.getItem(this.VISIT_HISTORY_KEY);
      return history ? JSON.parse(history) : {};
    } catch (error) {
      console.warn('Failed to get visit history:', error);
      return {};
    }
  }

  /**
   * 保存阅读位置
   * @param {number|string} channelId - 频道ID
   * @param {Object} position - 位置信息
   */
  saveReadingPosition(channelId, position) {
    try {
      let positions = this.getReadingPositions();
      positions[String(channelId)] = {
        ...position,
        timestamp: Date.now()
      };
      
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(positions));
      console.log(`📖 [ReadingPosition] Saved position for channel ${channelId}:`, position);
    } catch (error) {
      console.warn('Failed to save reading position:', error);
    }
  }

  /**
   * 获取阅读位置
   * @param {number|string} channelId - 频道ID
   * @returns {Object|null} 阅读位置或null
   */
  getReadingPosition(channelId) {
    try {
      const positions = this.getReadingPositions();
      const position = positions[String(channelId)];
      
      // 如果位置超过7天，认为过期
      if (position && (Date.now() - position.timestamp) > 7 * 24 * 60 * 60 * 1000) {
        this.clearReadingPosition(channelId);
        return null;
      }
      
      return position || null;
    } catch (error) {
      console.warn('Failed to get reading position:', error);
      return null;
    }
  }

  /**
   * 获取所有阅读位置
   * @returns {Object} 所有阅读位置
   */
  getReadingPositions() {
    try {
      const positions = localStorage.getItem(this.STORAGE_KEY);
      return positions ? JSON.parse(positions) : {};
    } catch (error) {
      console.warn('Failed to get reading positions:', error);
      return {};
    }
  }

  /**
   * 清除频道的阅读位置
   * @param {number|string} channelId - 频道ID
   */
  clearReadingPosition(channelId) {
    try {
      let positions = this.getReadingPositions();
      delete positions[String(channelId)];
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(positions));
    } catch (error) {
      console.warn('Failed to clear reading position:', error);
    }
  }

  /**
   * 确定频道的加载策略
   * @param {number|string} channelId - 频道ID
   * @returns {Object} 加载策略
   */
  getLoadingStrategy(channelId) {
    const isVisitedInSession = this.isChannelVisitedInSession(channelId);
    const savedPosition = this.getReadingPosition(channelId);
    const visitHistory = this.getVisitHistory()[String(channelId)];

    console.log(`📖 [ReadingPosition] Strategy for channel ${channelId}:`, {
      visitedInSession: isVisitedInSession,
      hasSavedPosition: !!savedPosition,
      hasVisitHistory: !!visitHistory
    });

    if (!isVisitedInSession) {
      // 首次访问：加载最新消息
      return {
        type: 'latest',
        scrollToBottom: true,
        reason: 'first_visit_in_session'
      };
    } else if (savedPosition) {
      // 有保存的阅读位置：恢复到该位置
      return {
        type: 'resume',
        scrollToMessage: savedPosition.messageId,
        scrollOffset: savedPosition.scrollOffset,
        reason: 'resume_last_position'
      };
    } else {
      // 访问过但没有保存位置：加载最新消息
      return {
        type: 'latest',
        scrollToBottom: true,
        reason: 'no_saved_position'
      };
    }
  }

  /**
   * 清理过期数据
   */
  cleanup() {
    try {
      // 清理过期的阅读位置（超过7天）
      const positions = this.getReadingPositions();
      const now = Date.now();
      const sevenDays = 7 * 24 * 60 * 60 * 1000;
      
      Object.keys(positions).forEach(channelId => {
        if (now - positions[channelId].timestamp > sevenDays) {
          delete positions[channelId];
        }
      });
      
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(positions));
      
      // 清理过期的访问历史（超过30天）
      const history = this.getVisitHistory();
      const thirtyDays = 30 * 24 * 60 * 60 * 1000;
      
      Object.keys(history).forEach(channelId => {
        if (now - history[channelId].lastVisit > thirtyDays) {
          delete history[channelId];
        }
      });
      
      localStorage.setItem(this.VISIT_HISTORY_KEY, JSON.stringify(history));
      
      console.log('📖 [ReadingPosition] Cleanup completed');
    } catch (error) {
      console.warn('Failed to cleanup reading positions:', error);
    }
  }

  /**
   * 获取调试信息
   * @returns {Object} 调试信息
   */
  getDebugInfo() {
    return {
      sessionChannels: [...this.sessionChannels],
      readingPositions: this.getReadingPositions(),
      visitHistory: this.getVisitHistory()
    };
  }
}

// 创建单例实例
export const readingPositionManager = new ReadingPositionManager();

// 在页面卸载时清理
if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    readingPositionManager.saveSessionData();
  });
  
  // 开发模式下暴露到全局
  if (import.meta.env.DEV) {
    window.readingPositionManager = readingPositionManager;
  }
}

export default readingPositionManager;