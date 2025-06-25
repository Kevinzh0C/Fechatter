/**
 * 📅 MessageSessionGrouper - 消息时间分组器
 * 
 * 解决问题：
 * 1. 消息缺少分割线，用户认知负担高
 * 2. 需要按时间session自动分组，降低信息处理难度
 * 
 * 核心特性：
 * - 智能时间间隔分析
 * - 会话自动分组
 * - 日期分割线插入
 * - 认知负担优化
 */

export class MessageSessionGrouper {
  constructor() {
    // 时间分组配置
    this.groupingRules = {
      // 时间间隔阈值 (毫秒)
      intervals: {
        newSession: 30 * 60 * 1000,    // 30分钟 - 新会话
        newDay: 24 * 60 * 60 * 1000,   // 24小时 - 新日期
        shortBreak: 5 * 60 * 1000,     // 5分钟 - 短暂休息
        longBreak: 2 * 60 * 60 * 1000  // 2小时 - 长时间休息
      },

      // 分组条件
      conditions: {
        minMessagesInSession: 2,        // 会话最小消息数
        maxSessionDuration: 4 * 60 * 60 * 1000, // 4小时 - 最大会话时长
        sameUserContinuityThreshold: 2 * 60 * 1000, // 2分钟 - 同用户连续性
        differentUserBreakThreshold: 10 * 60 * 1000 // 10分钟 - 不同用户间隔
      },

      // 特殊规则
      special: {
        weekendSessionExtension: 1.5,   // 周末会话时间延长倍数
        nightTimeSessionReduction: 0.7, // 夜间会话时间缩短倍数
        holidayDetection: true,         // 节假日检测
        workHoursAdjustment: true       // 工作时间调整
      }
    };

    // 缓存和状态
    this.sessionCache = new Map();
    this.dividerCache = new Map();
    this.analysisCache = new Map();

    // 性能配置
    this.performanceConfig = {
      maxCacheSize: 100,      // 最大缓存条目数
      cacheExpiration: 30 * 60 * 1000, // 30分钟缓存过期
      batchSize: 50,          // 批处理大小
      enableDebounce: true    // 启用防抖
    };

    if (import.meta.env.DEV) {
      console.log('📅 [MessageSessionGrouper] 消息时间分组器已初始化');
    }
  }

  /**
   * 🔍 分析和分组消息列表
   */
  analyzeAndGroupMessages(messages, chatId) {
    if (!messages || messages.length === 0) {
      return { groupedMessages: [], dividers: [], sessions: [] };
    }

    // 检查缓存
    const cacheKey = this.generateCacheKey(messages, chatId);
    if (this.analysisCache.has(cacheKey)) {
      const cached = this.analysisCache.get(cacheKey);
      if (Date.now() - cached.timestamp < this.performanceConfig.cacheExpiration) {
        return cached.result;
      }
    }

    // 按时间排序消息 (确保正确顺序)
    const sortedMessages = [...messages].sort((a, b) =>
      new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
    );

    // 执行分组分析
    const result = this.performGroupingAnalysis(sortedMessages, chatId);

    // 缓存结果
    this.cacheAnalysisResult(cacheKey, result);

    return result;
  }

  /**
   * 🧠 执行分组分析核心算法
   */
  performGroupingAnalysis(messages, chatId) {
    const sessions = [];
    const dividers = [];
    const groupedMessages = [];

    let currentSession = null;
    let lastMessage = null;
    let sessionId = 0;

    for (let i = 0; i < messages.length; i++) {
      const message = messages[i];
      const messageTime = new Date(message.created_at);

      // 第一条消息 - 开始新会话
      if (!lastMessage) {
        currentSession = this.createNewSession(sessionId++, message, messageTime);
        sessions.push(currentSession);

        // 添加日期分割线 (如果是当天第一条消息)
        const dateDivider = this.createDateDivider(messageTime, 'session-start');
        if (dateDivider) {
          dividers.push(dateDivider);
          groupedMessages.push(dateDivider);

          // 🔧 CRITICAL FIX: 每个主日期分割线下方必须显示一次副日期分割线
          const subDateDivider = this.createSubDateDivider(
            messageTime, // 首条消息使用相同时间
            messageTime,
            'session-start'
          );
          dividers.push(subDateDivider);
          groupedMessages.push(subDateDivider);
        }

        groupedMessages.push(this.enhanceMessageWithSession(message, currentSession));
        currentSession.messages.push(message);
        lastMessage = message;
        continue;
      }

      const lastMessageTime = new Date(lastMessage.created_at);
      const timeGap = messageTime.getTime() - lastMessageTime.getTime();

      // 分析是否需要新会话或分割线
      const breakAnalysis = this.analyzeTimeBreak(lastMessage, message, timeGap);

      if (breakAnalysis.needsNewSession) {
        // 结束当前会话
        this.finalizeSession(currentSession);

        // 🔧 CRITICAL FIX: 主日期分界线下面必须跟一个副日期分割线
        // 先添加日期分割线 (如果跨日) - 主日期分界线
        if (breakAnalysis.needsDateDivider) {
          const dateDivider = this.createDateDivider(messageTime, 'new-day');
          dividers.push(dateDivider);
          groupedMessages.push(dateDivider);

          // ✅ 强制添加副日期分割线 - 必须跟在主日期分界线后面
          const subDateDivider = this.createSubDateDivider(
            lastMessageTime,
            messageTime,
            'new-day-session'
          );
          dividers.push(subDateDivider);
          groupedMessages.push(subDateDivider);
        } else if (breakAnalysis.needsSessionDivider) {
          // 添加普通会话间分割线 (非跨日情况)
          const sessionDivider = this.createSessionDivider(
            lastMessageTime,
            messageTime,
            breakAnalysis.breakType
          );
          dividers.push(sessionDivider);
          groupedMessages.push(sessionDivider);
        }

        // 创建新会话
        currentSession = this.createNewSession(sessionId++, message, messageTime);
        sessions.push(currentSession);
      }

      // 添加消息到当前会话
      groupedMessages.push(this.enhanceMessageWithSession(message, currentSession));
      currentSession.messages.push(message);
      this.updateSessionMetrics(currentSession, message, messageTime);

      lastMessage = message;
    }

    // 完成最后一个会话
    if (currentSession) {
      this.finalizeSession(currentSession);
    }

    if (import.meta.env.DEV) {
      console.log(`📅 [MessageSessionGrouper] 分组完成:`, {
        chatId,
        messagesCount: messages.length,
        sessionsCount: sessions.length,
        dividersCount: dividers.length
      });
    }

    // 🔧 CRITICAL VERIFICATION: 确保每个主日期分割线的相邻下方都有副日期分割线
    const verifiedGroupedMessages = this.ensureSubDateDividersFollowMainDividers(groupedMessages);

    return {
      groupedMessages: verifiedGroupedMessages,
      dividers,
      sessions,
      metadata: {
        chatId,
        totalMessages: messages.length,
        totalSessions: sessions.length,
        avgSessionLength: sessions.length > 0 ? messages.length / sessions.length : 0,
        timeSpan: messages.length > 0 ?
          new Date(messages[messages.length - 1].created_at).getTime() -
          new Date(messages[0].created_at).getTime() : 0
      }
    };
  }

  /**
   * ⏰ 分析时间间隔类型
   */
  analyzeTimeBreak(lastMessage, currentMessage, timeGap) {
    const lastTime = new Date(lastMessage.created_at);
    const currentTime = new Date(currentMessage.created_at);

    // 检查是否跨日
    const isDifferentDay = !this.isSameDay(lastTime, currentTime);

    // 检查是否不同用户
    const isDifferentUser = lastMessage.sender_id !== currentMessage.sender_id;

    // 应用时间调整因子
    const adjustedIntervals = this.calculateAdjustedIntervals(currentTime);

    // 判断分割类型
    const breakAnalysis = {
      timeGap,
      isDifferentDay,
      isDifferentUser,
      needsNewSession: false,
      needsSessionDivider: false,
      needsDateDivider: isDifferentDay,
      breakType: 'none'
    };

    // 日期分割 (优先级最高)
    if (isDifferentDay) {
      breakAnalysis.needsNewSession = true;
      breakAnalysis.needsSessionDivider = true;
      breakAnalysis.breakType = 'new-day';
      return breakAnalysis;
    }

    // 长时间间隔 - 新会话
    if (timeGap >= adjustedIntervals.newSession) {
      breakAnalysis.needsNewSession = true;
      breakAnalysis.needsSessionDivider = true;
      breakAnalysis.breakType = timeGap >= adjustedIntervals.longBreak ? 'long-break' : 'session-break';
      return breakAnalysis;
    }

    // 不同用户的特殊处理
    if (isDifferentUser && timeGap >= adjustedIntervals.differentUser) {
      breakAnalysis.needsNewSession = true;
      breakAnalysis.needsSessionDivider = true;
      breakAnalysis.breakType = 'user-switch';
      return breakAnalysis;
    }

    // 短暂休息 - 添加轻微分割线但不新建会话
    if (timeGap >= adjustedIntervals.shortBreak) {
      breakAnalysis.needsSessionDivider = true;
      breakAnalysis.breakType = 'short-break';
      return breakAnalysis;
    }

    return breakAnalysis;
  }

  /**
   * 🕐 计算调整后的时间间隔
   */
  calculateAdjustedIntervals(currentTime) {
    const baseIntervals = this.groupingRules.intervals;
    const conditions = this.groupingRules.conditions;

    let multiplier = 1.0;

    // 周末调整
    const isWeekend = this.isWeekend(currentTime);
    if (isWeekend && this.groupingRules.special.weekendSessionExtension) {
      multiplier *= this.groupingRules.special.weekendSessionExtension;
    }

    // 夜间调整 (22:00 - 06:00)
    const isNightTime = this.isNightTime(currentTime);
    if (isNightTime && this.groupingRules.special.nightTimeSessionReduction) {
      multiplier *= this.groupingRules.special.nightTimeSessionReduction;
    }

    return {
      newSession: baseIntervals.newSession * multiplier,
      shortBreak: baseIntervals.shortBreak * multiplier,
      longBreak: baseIntervals.longBreak * multiplier,
      differentUser: conditions.differentUserBreakThreshold * multiplier
    };
  }

  /**
   * 🆕 创建新会话
   */
  createNewSession(sessionId, firstMessage, startTime) {
    return {
      id: `session_${sessionId}`,
      startTime,
      endTime: startTime,
      messages: [],
      participantIds: new Set([firstMessage.sender_id]),
      messageCount: 0,
      avgResponseTime: 0,
      type: 'conversation', // 'conversation', 'announcement', 'discussion'
      metadata: {
        isActive: true,
        hasFiles: false,
        hasReactions: false,
        dominantParticipant: firstMessage.sender_id,
        averageMessageLength: 0,
        timespan: 0
      }
    };
  }

  /**
   * 📝 增强消息对象
   */
  enhanceMessageWithSession(message, session) {
    return {
      ...message,
      sessionId: session.id,
      sessionType: session.type,
      isSessionStart: session.messages.length === 0,
      sessionMetadata: {
        participantCount: session.participantIds.size,
        messageIndex: session.messages.length
      }
    };
  }

  /**
   * 📊 更新会话指标
   */
  updateSessionMetrics(session, message, messageTime) {
    session.endTime = messageTime;
    session.participantIds.add(message.sender_id);
    session.messageCount = session.messages.length + 1;
    session.metadata.timespan = messageTime.getTime() - session.startTime.getTime();

    // 更新文件和反应统计
    if (message.files && message.files.length > 0) {
      session.metadata.hasFiles = true;
    }

    if (message.reactions && message.reactions.length > 0) {
      session.metadata.hasReactions = true;
    }

    // 计算平均消息长度
    const totalLength = session.messages.reduce((sum, msg) => sum + (msg.content?.length || 0), 0);
    session.metadata.averageMessageLength = totalLength / session.messages.length;
  }

  /**
   * ✅ 完成会话
   */
  finalizeSession(session) {
    session.metadata.isActive = false;

    // 确定会话类型
    if (session.messageCount === 1) {
      session.type = 'single-message';
    } else if (session.participantIds.size === 1) {
      session.type = 'monologue';
    } else if (session.participantIds.size >= 3) {
      session.type = 'group-discussion';
    } else {
      session.type = 'conversation';
    }

    if (import.meta.env.DEV) {
      console.log(`📅 [MessageSessionGrouper] 会话完成:`, {
        sessionId: session.id,
        type: session.type,
        messageCount: session.messageCount,
        participantCount: session.participantIds.size,
        duration: session.metadata.timespan
      });
    }
  }

  /**
   * 📅 创建日期分割线
   */
  createDateDivider(date, type) {
    const divider = {
      id: `date_divider_${date.getTime()}`,
      type: 'date-divider',
      subType: type, // 'session-start', 'new-day'
      date: date,
      displayText: this.formatDateDivider(date),
      metadata: {
        isToday: this.isToday(date),
        isYesterday: this.isYesterday(date),
        isWeekend: this.isWeekend(date),
        dayOfWeek: date.getDay()
      }
    };

    return divider;
  }

  /**
   * ⏳ 创建会话分割线
   */
  createSessionDivider(lastTime, currentTime, breakType) {
    const timeGap = currentTime.getTime() - lastTime.getTime();

    return {
      id: `session_divider_${currentTime.getTime()}`,
      type: 'session-divider',
      subType: breakType, // 'short-break', 'session-break', 'long-break', 'user-switch'
      timeGap,
      displayText: this.formatSessionBreak(timeGap, breakType),
      metadata: {
        lastMessageTime: lastTime,
        currentMessageTime: currentTime,
        gapDuration: timeGap,
        isLongBreak: timeGap >= this.groupingRules.intervals.longBreak
      }
    };
  }

  /**
   * 📅 创建副日期分割线 - 必须跟在主日期分界线后面
   * 用于显示时间段信息，如"Morning Session"、"Afternoon Break"等
   */
  createSubDateDivider(lastTime, currentTime, breakType) {
    const timeGap = currentTime.getTime() - lastTime.getTime();

    return {
      id: `sub_date_divider_${currentTime.getTime()}`,
      type: 'sub-date-divider', // 🔧 NEW: 副日期分割线类型
      subType: breakType, // 'new-day-session', 'morning-session', 'afternoon-session'
      timeGap,
      displayText: this.formatSubDateDivider(lastTime, currentTime, breakType),
      metadata: {
        lastMessageTime: lastTime,
        currentMessageTime: currentTime,
        gapDuration: timeGap,
        timeOfDay: this.getTimeOfDay(currentTime),
        isNewDay: true
      }
    };
  }

  /**
   * 🎨 格式化日期分割线显示文本 - 具体月日+星期几格式
   */
  formatDateDivider(date) {
    if (this.isToday(date)) {
      // Today - December 17, Monday
      const todayFormat = date.toLocaleDateString('en-US', {
        month: 'long',
        day: 'numeric',
        weekday: 'long'
      });
      return `Today - ${todayFormat}`;
    }

    if (this.isYesterday(date)) {
      // Yesterday - December 16, Sunday
      const yesterdayFormat = date.toLocaleDateString('en-US', {
        month: 'long',
        day: 'numeric',
        weekday: 'long'
      });
      return `Yesterday - ${yesterdayFormat}`;
    }

    const now = new Date();
    const diffDays = Math.floor((now - date) / (24 * 60 * 60 * 1000));

    if (diffDays <= 7) {
      // December 15, Saturday
      return date.toLocaleDateString('en-US', {
        month: 'long',
        day: 'numeric',
        weekday: 'long'
      });
    }

    if (diffDays <= 365) {
      // December 10, Monday
      return date.toLocaleDateString('en-US', {
        month: 'long',
        day: 'numeric',
        weekday: 'long'
      });
    }

    // December 17, 2023 - Monday
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      weekday: 'long'
    });
  }

  /**
   * ⏰ 格式化会话间隔显示文本
   */
  formatSessionBreak(timeGap, breakType) {
    const minutes = Math.floor(timeGap / (60 * 1000));
    const hours = Math.floor(minutes / 60);

    switch (breakType) {
      case 'short-break':
        return minutes < 10 ? `${minutes} minutes later` : null; // 短间隔可能不显示

      case 'session-break':
        if (hours >= 1) {
          return `${hours} hour${hours > 1 ? 's' : ''} later`;
        }
        return `${minutes} minutes later`;

      case 'long-break':
        if (hours >= 24) {
          const days = Math.floor(hours / 24);
          return `${days} day${days > 1 ? 's' : ''} later`;
        }
        return `${hours} hour${hours > 1 ? 's' : ''} later`;

      case 'user-switch':
        return null; // 用户切换通常不需要时间显示

      default:
        return null;
    }
  }

  /**
   * 🕐 格式化副日期分割线显示文本
   * 显示具体时分信息，如"9:30 AM - Morning Conversation Begins"
   */
  formatSubDateDivider(lastTime, currentTime, breakType) {
    // 🎯 核心改进：使用第一条消息的具体时分作为副日期分割线显示
    const timeOfDay = this.getTimeOfDay(currentTime);
    const timeString = currentTime.toLocaleTimeString('en-US', {
      hour: 'numeric',
      minute: '2-digit',
      hour12: true
    });

    switch (breakType) {
      case 'session-start':
        // 🔧 NEW: 显示具体时分 + 会话开始信息
        return `${timeString} - ${timeOfDay} Conversation Begins`;

      case 'new-day-session':
        // 🔧 NEW: 跨日消息显示具体时分
        return `${timeString} - ${timeOfDay} Session`;

      case 'morning-session':
        return `${timeString} - Morning Conversation`;

      case 'afternoon-session':
        return `${timeString} - Afternoon Discussion`;

      case 'evening-session':
        return `${timeString} - Evening Chat`;

      default:
        const hours = Math.floor((currentTime.getTime() - lastTime.getTime()) / (1000 * 60 * 60));
        if (hours >= 12) {
          return `${timeString} - ${timeOfDay} (After ${hours}h break)`;
        }
        return `${timeString} - ${timeOfDay} Conversation`;
    }
  }

  /**
   * 🌅 获取时间段信息
   */
  getTimeOfDay(date) {
    const hour = date.getHours();

    if (hour >= 5 && hour < 12) {
      return 'Morning';
    } else if (hour >= 12 && hour < 17) {
      return 'Afternoon';
    } else if (hour >= 17 && hour < 22) {
      return 'Evening';
    } else {
      return 'Night';
    }
  }

  /**
   * 🗓️ 日期判断辅助函数
   */
  isToday(date) {
    const today = new Date();
    return this.isSameDay(date, today);
  }

  isYesterday(date) {
    const yesterday = new Date();
    yesterday.setDate(yesterday.getDate() - 1);
    return this.isSameDay(date, yesterday);
  }

  isSameDay(date1, date2) {
    return date1.getFullYear() === date2.getFullYear() &&
      date1.getMonth() === date2.getMonth() &&
      date1.getDate() === date2.getDate();
  }

  isWeekend(date) {
    const day = date.getDay();
    return day === 0 || day === 6; // Sunday = 0, Saturday = 6
  }

  isNightTime(date) {
    const hour = date.getHours();
    return hour >= 22 || hour <= 6;
  }

  /**
   * 🗝️ 缓存管理
   */
  generateCacheKey(messages, chatId) {
    if (!messages || messages.length === 0) return `${chatId}_empty`;

    const firstId = messages[0].id;
    const lastId = messages[messages.length - 1].id;
    const count = messages.length;

    return `${chatId}_${firstId}_${lastId}_${count}`;
  }

  cacheAnalysisResult(cacheKey, result) {
    // 清理过期缓存
    if (this.analysisCache.size >= this.performanceConfig.maxCacheSize) {
      this.cleanupCache();
    }

    this.analysisCache.set(cacheKey, {
      result,
      timestamp: Date.now()
    });
  }

  cleanupCache() {
    const now = Date.now();
    const expiration = this.performanceConfig.cacheExpiration;

    for (const [key, value] of this.analysisCache.entries()) {
      if (now - value.timestamp > expiration) {
        this.analysisCache.delete(key);
      }
    }
  }

  /**
   * 📊 获取分组统计
   */
  getGroupingStats() {
    return {
      cacheSize: this.analysisCache.size,
      rules: this.groupingRules,
      performance: this.performanceConfig
    };
  }

  /**
   * 🧹 清理资源
   */
  cleanup() {
    this.sessionCache.clear();
    this.dividerCache.clear();
    this.analysisCache.clear();

    if (import.meta.env.DEV) {
      console.log('🧹 [MessageSessionGrouper] 资源已清理');
    }
  }

  /**
   * 🔧 CRITICAL VERIFICATION: 确保每个主日期分割线的相邻下方都有副日期分割线
   * 这是最终的保障机制，确保100%的主日期分割线都有对应的副日期分割线
   */
  ensureSubDateDividersFollowMainDividers(groupedMessages) {
    const verifiedMessages = [];

    for (let i = 0; i < groupedMessages.length; i++) {
      const currentItem = groupedMessages[i];
      verifiedMessages.push(currentItem);

      // 🎯 核心逻辑：如果当前是主日期分割线
      if (currentItem.type === 'date-divider') {
        const nextItem = groupedMessages[i + 1];

        // 检查下一个元素是否是副日期分割线
        const hasSubDateDivider = nextItem && nextItem.type === 'sub-date-divider';

        if (!hasSubDateDivider) {
          // 🚨 没有副日期分割线！立即插入一个
          const subDateDivider = this.createSubDateDivider(
            currentItem.date,
            currentItem.date,
            currentItem.subType === 'new-day' ? 'new-day-session' : 'session-start'
          );

          verifiedMessages.push(subDateDivider);

          if (import.meta.env.DEV) {
            console.warn(`🔧 [MessageSessionGrouper] 自动插入副日期分割线 after main divider:`, currentItem.displayText);
          }
        }
      }
    }

    if (import.meta.env.DEV) {
      const mainDividers = verifiedMessages.filter(m => m.type === 'date-divider').length;
      const subDividers = verifiedMessages.filter(m => m.type === 'sub-date-divider').length;
      console.log(`📊 [MessageSessionGrouper] 验证完成: ${mainDividers} 个主日期分割线, ${subDividers} 个副日期分割线`);
    }

    return verifiedMessages;
  }
}

// 创建全局单例
export const messageSessionGrouper = new MessageSessionGrouper();

// 开发环境下全局可用
if (import.meta.env.DEV) {
  window.messageSessionGrouper = messageSessionGrouper;
  console.log('📅 MessageSessionGrouper 全局可用');
} 