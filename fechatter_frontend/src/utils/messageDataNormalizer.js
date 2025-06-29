/**
 * 消息数据标准化器
 * 解决搜索和消息列表之间的数据结构不一致问题
 * 
 * 核心原则：
 * 1. 统一数据合约 - 所有消息无论来源都遵循同一数据结构
 * 2. 防御性编程 - 处理任何可能的数据缺失或格式异常
 * 3. 向前兼容 - 支持新旧数据格式的平滑过渡
 */

/**
 * 标准消息对象接口定义
 */
export const MESSAGE_SCHEMA = {
  id: 'number',
  content: 'string',
  sender_id: 'number',
  sender: {
    id: 'number',
    fullname: 'string',
    email: 'string',
    avatar_url: 'string|null'
  },
  chat_id: 'number',
  created_at: 'string', // ISO format
  files: 'array',
  edited_at: 'string|null',
  reply_to: 'object|null',
  reactions: 'array'
};

/**
 * 时间戳标准化函数
 * 处理多种时间格式并统一为ISO字符串
 */
export function normalizeTimestamp(timestamp) {
  if (!timestamp) return new Date().toISOString();

  // 如果已经是ISO字符串格式
  if (typeof timestamp === 'string' && timestamp.includes('T')) {
    return timestamp;
  }

  // 如果是Unix时间戳（秒）
  if (typeof timestamp === 'number') {
    // 判断是秒还是毫秒（典型的秒级时间戳小于毫秒级）
    const tsMs = timestamp < 1e10 ? timestamp * 1000 : timestamp;
    return new Date(tsMs).toISOString();
  }

  // 如果是时间戳字符串
  if (typeof timestamp === 'string') {
    const num = parseFloat(timestamp);
    if (!isNaN(num)) {
      const tsMs = num < 1e10 ? num * 1000 : num;
      return new Date(tsMs).toISOString();
    }

  // 尝试直接解析
  try {
    return new Date(timestamp).toISOString();
  } catch (error) {
    // console.warn('[MessageNormalizer] Failed to parse timestamp:', timestamp);
    return new Date().toISOString();
  }

/**
 * Sender对象标准化函数
 * 从各种可能的数据源构建统一的sender对象
 */
export function normalizeSender(messageData) {
  // 如果已经有完整的sender对象
  if (messageData.sender && typeof messageData.sender === 'object') {
    return {
      id: messageData.sender.id || messageData.sender_id || 0,
      fullname: messageData.sender.fullname || messageData.sender.name || messageData.sender_name || 'Unknown User',
      email: messageData.sender.email || '',
      avatar_url: messageData.sender.avatar_url || null
    };
  }

  // 如果只有基础字段，构建sender对象
  return {
    id: messageData.sender_id || 0,
    fullname: messageData.sender_name || messageData.user_name || 'Unknown User',
    email: messageData.sender_email || '',
    avatar_url: messageData.sender_avatar || messageData.avatar_url || null
  };
}

/**
 * 文件数组标准化函数
 */
export function normalizeFiles(files) {
  if (!files) return [];
  if (Array.isArray(files)) return files;
  if (typeof files === 'string') {
    try {
      return JSON.parse(files);
    } catch {
      return [files]; // 如果解析失败，当作单个文件名
  return [];
}

/**
 * 主要的消息标准化函数
 * 将任何格式的消息数据转换为标准格式
 */
export function normalizeMessage(messageData, source = 'unknown') {
  if (!messageData) {
    // console.warn('[MessageNormalizer] Received null/undefined message data');
    return null;
  }

  try {
    const normalized = {
      // 基础字段
      id: messageData.id || messageData.message_id || 0,
      content: messageData.content || messageData.text || messageData.message || '',
      sender_id: messageData.sender_id || messageData.user_id || 0,
      chat_id: messageData.chat_id || messageData.chatId || 0,

      // 复杂字段
      sender: normalizeSender(messageData),
      created_at: normalizeTimestamp(messageData.created_at || messageData.timestamp || messageData.time),
      files: normalizeFiles(messageData.files || messageData.attachments),

      // 可选字段
      edited_at: messageData.edited_at ? normalizeTimestamp(messageData.edited_at) : null,
      reply_to: messageData.reply_to || messageData.replyTo || null,
      reactions: messageData.reactions || [],

      // 元数据
      _source: source,
      _normalized_at: new Date().toISOString()
    };

    // 验证必需字段
    if (!normalized.id || !normalized.sender_id) {
      // console.warn('[MessageNormalizer] Message missing required fields:', {
      //   id: normalized.id,
      //   sender_id: normalized.sender_id,
      //   source,
      //   original: messageData
      // });

    return normalized;
  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('[MessageNormalizer] Failed to normalize message:', error, messageData);
    return null;
  }

/**
 * 批量消息标准化函数
 */
export function normalizeMessages(messages, source = 'unknown') {
  if (!Array.isArray(messages)) {
    // console.warn('[MessageNormalizer] Expected array but got:', typeof messages);
    return [];
  }

  return messages
    .map(msg => normalizeMessage(msg, source))
    .filter(msg => msg !== null); // 过滤掉标准化失败的消息
}

/**
 * 搜索结果特殊标准化函数
 * 专门处理搜索API的响应格式
 */
export function normalizeSearchResults(searchResponse) {
  // console.log('[MessageNormalizer] Normalizing search results:', searchResponse);

  // 处理不同的响应结构
  let hits = [];
  let total = 0;
  let took_ms = 0;

  if (searchResponse.data && searchResponse.data.hits) {
    // 标准的搜索响应格式
    hits = searchResponse.data.hits;
    total = searchResponse.data.total || 0;
    took_ms = searchResponse.data.took_ms || 0;
  } else if (searchResponse.hits) {
    // 直接的hits数组
    hits = searchResponse.hits;
    total = searchResponse.total || hits.length;
    took_ms = searchResponse.took_ms || 0;
  } else if (Array.isArray(searchResponse.data)) {
    // 简单的消息数组
    hits = searchResponse.data;
    total = hits.length;
  } else if (Array.isArray(searchResponse)) {
    // 直接的消息数组
    hits = searchResponse;
    total = hits.length;
  } else {
    // console.warn('[MessageNormalizer] Unexpected search response format:', searchResponse);
    return {
      results: [],
      total: 0,
      took_ms: 0,
      _error: 'Unexpected response format'
    };
  }

  // 标准化每个搜索结果
  const normalizedResults = normalizeMessages(hits, 'search');

  return {
    results: normalizedResults,
    total: Math.max(total, normalizedResults.length),
    took_ms: took_ms,
    _normalized: true,
    _original_count: hits.length,
    _normalized_count: normalizedResults.length
  };
}

/**
 * 数据质量验证函数
 */
export function validateMessageData(messageData) {
  const issues = [];

  if (!messageData.id) issues.push('Missing ID');
  if (!messageData.content && !messageData.files?.length) issues.push('Missing content and files');
  if (!messageData.sender_id) issues.push('Missing sender_id');
  if (!messageData.sender?.fullname) issues.push('Missing sender name');
  if (!messageData.created_at) issues.push('Missing timestamp');

  return {
    isValid: issues.length === 0,
    issues: issues
  };
}

/**
 * 调试工具：比较两个消息对象的差异
 */
export function compareMessageStructures(msg1, msg2) {
  const diff = {};
  const allKeys = new Set([...Object.keys(msg1 || {}), ...Object.keys(msg2 || {})]);

  for (const key of allKeys) {
    if (msg1[key] !== msg2[key]) {
      diff[key] = {
        msg1: msg1[key],
        msg2: msg2[key],
        type1: typeof msg1[key],
        type2: typeof msg2[key]
      };
    }

  return diff;
}

/**
 * 全局暴露调试工具
 */
// if (typeof window !== 'undefined') {
//   window.messageNormalizer = {
//     normalizeMessage,
//     normalizeMessages,
//     normalizeSearchResults,
//     validateMessageData,
//     compareMessageStructures,
//     normalizeTimestamp,
//     normalizeSender
//   };

//   console.log('Message Data Normalizer loaded. Available commands:');
//   console.log('   window.messageNormalizer.normalizeMessage(data)');
//   console.log('   window.messageNormalizer.normalizeSearchResults(response)');
//   console.log('   window.messageNormalizer.validateMessageData(msg)');
//   console.log('   window.messageNormalizer.compareMessageStructures(msg1, msg2)');
// }

export default {
  normalizeMessage,
  normalizeMessages,
  normalizeSearchResults,
  validateMessageData,
  compareMessageStructures,
  normalizeTimestamp,
  normalizeSender,
  normalizeFiles,
  MESSAGE_SCHEMA
};