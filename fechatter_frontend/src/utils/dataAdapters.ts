/**
 * 数据适配器 - 处理前端组件和数据库之间的字段映射
 */
import type { Chat, ChatMessage } from '@/types/api';

/**
 * 适配Chat数据 - 处理数据库字段到前端组件字段的映射
 */
export function adaptChatData(rawChat: any): Chat {
  const adaptedChat: Chat = {
    ...rawChat,
    // 映射数据库字段到组件期望的字段
    name: rawChat.chat_name || rawChat.name,
    chat_type: mapChatType(rawChat.type),
    
    // 确保数组字段存在
    chat_members: rawChat.chat_members || [],
    
    // 确保必需字段存在
    workspace_id: rawChat.workspace_id || 1,
    max_members: rawChat.max_members || 100,
    is_public: rawChat.is_public ?? false,
    settings: rawChat.settings || {}
  };

  return adaptedChat;
}

/**
 * 适配ChatMessage数据 - 处理数据库字段到前端组件字段的映射
 */
export function adaptMessageData(rawMessage: any): ChatMessage {
  const adaptedMessage: ChatMessage = {
    ...rawMessage,
    // 确保必需字段存在
    content: rawMessage.content || '',
    files: rawMessage.files || [],
    mentions: rawMessage.mentions || [],
    read_by_users: rawMessage.read_by_users || [],
    
    // 状态字段
    status: rawMessage.status || 'sent',
    priority: rawMessage.priority || 'normal',
    is_important: rawMessage.is_important ?? false,
    is_edited: rawMessage.is_edited ?? false,
    is_scheduled: rawMessage.is_scheduled ?? false,
    edit_count: rawMessage.edit_count || 0,
    sequence_number: rawMessage.sequence_number || 0,
    
    // 处理发送者信息
    sender: rawMessage.sender ? {
      id: rawMessage.sender.id,
      fullname: rawMessage.sender.fullname,
      email: rawMessage.sender.email,
      avatar_url: rawMessage.sender.avatar_url,
      status: rawMessage.sender.status || 'Active',
      title: rawMessage.sender.title,
      department: rawMessage.sender.department
    } : undefined,
    
    // 处理反应数据
    reactions: rawMessage.reactions ? rawMessage.reactions.map(adaptReactionData) : []
  };

  return adaptedMessage;
}

/**
 * 适配反应数据
 */
export function adaptReactionData(rawReaction: any) {
  return {
    id: rawReaction.id,
    message_id: rawReaction.message_id,
    emoji: rawReaction.emoji,
    created_at: rawReaction.created_at,
    count: rawReaction.count || 1,
    users: rawReaction.users || [],
    user_names: rawReaction.user_names || []
  };
}

/**
 * 映射聊天类型
 */
function mapChatType(dbType: string): 'direct' | 'group' | 'channel' {
  switch (dbType?.toLowerCase()) {
    case 'direct':
    case 'dm':
    case 'private':
      return 'direct';
    case 'group':
    case 'private_group':
      return 'group';
    case 'channel':
    case 'public':
    case 'public_channel':
      return 'channel';
    default:
      return 'group'; // 默认类型
  }
}

/**
 * 反向映射聊天类型（前端到数据库）
 */
export function mapChatTypeToDb(frontendType: 'direct' | 'group' | 'channel'): string {
  switch (frontendType) {
    case 'direct':
      return 'direct';
    case 'group':
      return 'group';
    case 'channel':
      return 'channel';
    default:
      return 'group';
  }
}

/**
 * 检查消息是否有内容（文本或文件）
 */
export function hasMessageContent(message: ChatMessage): boolean {
  return !!(
    (message.content && message.content.trim()) ||
    (message.files && message.files.length > 0)
  );
}

/**
 * 获取消息显示内容
 */
export function getMessageDisplayContent(message: ChatMessage): string {
  if (message.content && message.content.trim()) {
    return message.content;
  }
  
  if (message.files && message.files.length > 0) {
    const fileCount = message.files.length;
    return fileCount === 1 ? 'Sent a file' : `Sent ${fileCount} files`;
  }
  
  return 'Empty message';
}

/**
 * 检查用户是否可以编辑消息
 */
export function canEditMessage(message: ChatMessage, currentUserId: number): boolean {
  // 只有发送者可以编辑消息
  if (message.sender_id !== currentUserId) {
    return false;
  }
  
  // 不能编辑系统消息或已调度的消息
  if (message.is_scheduled) {
    return false;
  }
  
  // 检查时间限制（比如24小时内）
  const messageTime = new Date(message.created_at).getTime();
  const now = Date.now();
  const timeDiff = now - messageTime;
  const twentyFourHours = 24 * 60 * 60 * 1000;
  
  return timeDiff < twentyFourHours;
}

/**
 * 检查用户是否可以删除消息
 */
export function canDeleteMessage(message: ChatMessage, currentUserId: number): boolean {
  // 只有发送者可以删除消息
  return message.sender_id === currentUserId;
}

/**
 * 格式化消息时间
 */
export function formatMessageTime(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diffInMinutes = Math.floor((now.getTime() - date.getTime()) / (1000 * 60));
  
  if (diffInMinutes < 1) {
    return 'Just now';
  } else if (diffInMinutes < 60) {
    return `${diffInMinutes}m ago`;
  } else if (diffInMinutes < 1440) { // 24 hours
    const hours = Math.floor(diffInMinutes / 60);
    return `${hours}h ago`;
  } else {
    return date.toLocaleDateString();
  }
}

/**
 * 格式化消息状态文本
 */
export function formatMessageStatus(status: string): string {
  switch (status) {
    case 'sent':
      return 'Sent';
    case 'delivered':
      return 'Delivered';
    case 'read':
      return 'Read';
    case 'failed':
      return 'Failed';
    default:
      return 'Unknown';
  }
}

/**
 * 获取优先级颜色类
 */
export function getPriorityColorClass(priority: string): string {
  switch (priority) {
    case 'low':
      return 'text-gray-500';
    case 'normal':
      return 'text-gray-700';
    case 'high':
      return 'text-orange-600';
    case 'urgent':
      return 'text-red-600';
    default:
      return 'text-gray-700';
  }
}

/**
 * 处理分页参数
 */
export function adaptPaginationParams(params: any) {
  return {
    limit: params.limit || 50,
    offset: params.offset || 0,
    before: params.before, // 基于消息ID的分页
    order: params.order || 'desc'
  };
}

/**
 * 批量适配消息数据
 */
export function adaptMessagesData(rawMessages: any[]): ChatMessage[] {
  if (!Array.isArray(rawMessages)) {
    return [];
  }
  
  return rawMessages.map(adaptMessageData);
}

/**
 * 批量适配聊天数据
 */
export function adaptChatsData(rawChats: any[]): Chat[] {
  if (!Array.isArray(rawChats)) {
    return [];
  }
  
  return rawChats.map(adaptChatData);
}