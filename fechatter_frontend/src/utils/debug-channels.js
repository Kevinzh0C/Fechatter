// 调试脚本：检查 channels 显示问题
// 在浏览器控制台运行此脚本

import { useChatStore } from '@/stores/chat';
import { useAuthStore } from '@/stores/auth';

export async function debugChannelsIssue() {
  if (import.meta.env.DEV) {
    console.log('=== 开始诊断 Channels 显示问题 ===');
  }

  const chatStore = useChatStore();
  const authStore = useAuthStore();

  // 1. 检查认证状态
  if (import.meta.env.DEV) {
    console.log('\n📌 认证状态:');
  if (import.meta.env.DEV) {
    console.log('- 已登录:', authStore.isLoggedIn);
  if (import.meta.env.DEV) {
    console.log('- 用户:', authStore.user?.email);
  if (import.meta.env.DEV) {
    console.log('- Token:', authStore.token ? '存在' : '不存在');
  }

  // 2. 获取原始数据
  if (import.meta.env.DEV) {
    console.log('\n📌 获取聊天数据...');
  try {
    await chatStore.fetchChats();
    if (import.meta.env.DEV) {
      console.log('- 获取成功');
    }
  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('- 获取失败:', error);
    return;
  }

  // 3. 检查原始数据
  if (import.meta.env.DEV) {
    console.log('\n📌 原始数据分析:');
  if (import.meta.env.DEV) {
    console.log('- 总数:', chatStore.chats.length);
  if (import.meta.env.DEV) {
    console.log('- 完整数据:', JSON.stringify(chatStore.chats, null, 2));
  }

  // 4. 分析 chat_type 分布
  const typeDistribution = {};
  chatStore.chats.forEach(chat => {
    const type = chat.chat_type || 'undefined';
    typeDistribution[type] = (typeDistribution[type] || 0) + 1;
  });
  if (import.meta.env.DEV) {
    console.log('\n📌 Chat Type 分布:', typeDistribution);
  }

  // 5. 检查每个 chat 的详细信息
  if (import.meta.env.DEV) {
    console.log('\n📌 每个 Chat 的详细信息:');
  chatStore.chats.forEach((chat, index) => {
    if (import.meta.env.DEV) {
      console.log(`\n[${index}] ${chat.name}:`);
    if (import.meta.env.DEV) {
      console.log('  - ID:', chat.id);
    if (import.meta.env.DEV) {
      console.log('  - Type:', chat.chat_type);
    if (import.meta.env.DEV) {
      console.log('  - Workspace ID:', chat.workspace_id);
    if (import.meta.env.DEV) {
      console.log('  - Members:', chat.chat_members);
    if (import.meta.env.DEV) {
      console.log('  - Member Count:', chat.member_count);
    if (import.meta.env.DEV) {
      console.log('  - Description:', chat.description);
    }
  });

  // 6. 模拟过滤逻辑
  if (import.meta.env.DEV) {
    console.log('\n📌 模拟过滤逻辑:');
  const channels = chatStore.chats.filter(chat =>
    chat && (chat.chat_type === 'PublicChannel' || chat.chat_type === 'PrivateChannel')
  );
  if (import.meta.env.DEV) {
    console.log('- 过滤后的 Channels 数量:', channels.length);
  if (import.meta.env.DEV) {
    console.log('- 过滤后的 Channels:', channels.map(c => ({ id: c.id, name: c.name, type: c.chat_type })));
  }

  const dms = chatStore.chats.filter(chat =>
    chat && chat.chat_type === 'Single'
  );
  if (import.meta.env.DEV) {
    console.log('- 过滤后的 DMs 数量:', dms.length);
  if (import.meta.env.DEV) {
    console.log('- 过滤后的 DMs:', dms.map(d => ({ id: d.id, name: d.name, type: d.chat_type })));
  }

  // 7. 检查可能的问题
  if (import.meta.env.DEV) {
    console.log('\n📌 可能的问题:');
  }

  // 检查 chat_type 大小写问题
  const hasLowercaseTypes = chatStore.chats.some(chat =>
    chat.chat_type && chat.chat_type !== chat.chat_type.charAt(0).toUpperCase() + chat.chat_type.slice(1)
  );
  if (hasLowercaseTypes) {
    if (import.meta.env.DEV) {
      console.warn('⚠️ 发现小写的 chat_type！');
    const lowercaseChats = chatStore.chats.filter(chat =>
      chat.chat_type && chat.chat_type !== chat.chat_type.charAt(0).toUpperCase() + chat.chat_type.slice(1)
    );
    if (import.meta.env.DEV) {
      console.warn('  受影响的 chats:', lowercaseChats.map(c => ({ name: c.name, type: c.chat_type })));
    }

  // 检查缺失的 chat_type
  const missingType = chatStore.chats.filter(chat => !chat.chat_type);
  if (missingType.length > 0) {
    if (import.meta.env.DEV) {
      console.warn('⚠️ 发现缺失 chat_type 的 chats:', missingType.map(c => ({ id: c.id, name: c.name })));
    }

  // 检查意外的 chat_type 值
  const expectedTypes = ['PublicChannel', 'PrivateChannel', 'Single'];
  const unexpectedTypes = chatStore.chats.filter(chat =>
    chat.chat_type && !expectedTypes.includes(chat.chat_type)
  );
  if (unexpectedTypes.length > 0) {
    if (import.meta.env.DEV) {
      console.warn('⚠️ 发现意外的 chat_type 值:', unexpectedTypes.map(c => ({ name: c.name, type: c.chat_type })));
    }

  if (import.meta.env.DEV) {
    console.log('\n=== 诊断完成 ===');
  }

  return {
    totalChats: chatStore.chats.length,
    channels: channels.length,
    dms: dms.length,
    typeDistribution,
    possibleIssues: {
      hasLowercaseTypes,
      missingTypeCount: missingType.length,
      unexpectedTypeCount: unexpectedTypes.length
    }
  };
}

// 导出到 window 对象，方便在控制台调用
if (typeof window !== 'undefined') {
  window.debugChannels = debugChannelsIssue;
} 