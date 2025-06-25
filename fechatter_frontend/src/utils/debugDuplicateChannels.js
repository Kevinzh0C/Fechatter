/**
 * Debug tool for tracking duplicate channels issue
 * Problem: "General Discussion" appears twice in the channel list
 */

export function debugDuplicateChannels() {
  console.log('🔍 [DEBUG] Starting duplicate channels investigation...');

  // Get all stores
  const chatStore = useChatStore();
  const workspaceStore = useWorkspaceStore();

  // 1. Check raw API response
  console.group('📡 API Response Analysis');
  console.log('Workspace chats:', workspaceStore.workspaceChats);
  console.log('Chat store chats:', chatStore.chats);
  console.groupEnd();

  // 2. Check for duplicates in chat store
  console.group('🔍 Duplicate Detection in Chat Store');
  const chatIds = new Set();
  const duplicates = [];

  chatStore.chats.forEach((chat, index) => {
    if (chatIds.has(chat.id)) {
      duplicates.push({ chat, index });
      console.warn(`❌ Duplicate found: ${chat.name} (ID: ${chat.id}) at index ${index}`);
    }
    chatIds.add(chat.id);
  });

  if (duplicates.length === 0) {
    console.log('✅ No duplicates found by ID');
  }

  // Check for same name
  const nameGroups = {};
  chatStore.chats.forEach((chat, index) => {
    const name = chat.name || 'Unnamed';
    if (!nameGroups[name]) {
      nameGroups[name] = [];
    }
    nameGroups[name].push({ chat, index });
  });

  Object.entries(nameGroups).forEach(([name, chats]) => {
    if (chats.length > 1) {
      console.warn(`⚠️ Multiple chats with name "${name}":`, chats);
    }
  });
  console.groupEnd();

  // 3. Check computed properties in Home.vue
  console.group('🏠 Home.vue Computed Properties');
  // This would need to be called from within Home.vue component
  console.log('Call debugChannelsList() from Home.vue to see computed values');
  console.groupEnd();

  // 4. Track data flow
  console.group('📊 Data Flow Trace');
  console.log('1. workspace.fetchWorkspaceChats() -> workspaceChats');
  console.log('2. chat.fetchChats() -> normalizes and stores in chats');
  console.log('3. Home.vue channelsList computed -> filters chats');
  console.log('4. ChannelList.vue receives props -> renders items');
  console.groupEnd();

  return {
    chatStoreChats: chatStore.chats,
    workspaceChats: workspaceStore.workspaceChats,
    duplicates,
    nameGroups
  };
}

// Helper function to be called from Home.vue
export function debugChannelsList(channelsList) {
  console.group('📋 ChannelsList Analysis');
  console.log('Total channels:', channelsList.length);

  // Check for duplicates
  const seen = new Set();
  const duplicates = [];

  channelsList.forEach((channel, index) => {
    const key = `${channel.id}-${channel.name}`;
    if (seen.has(channel.id)) {
      duplicates.push({ channel, index });
      console.warn(`❌ Duplicate in channelsList: ${channel.name} (ID: ${channel.id}) at index ${index}`);
    }
    seen.add(channel.id);
  });

  if (duplicates.length === 0) {
    console.log('✅ No duplicates in channelsList');
  } else {
    console.error('❌ Found duplicates:', duplicates);
  }

  // Show all channels
  console.table(channelsList.map(c => ({
    id: c.id,
    name: c.name,
    type: c.chat_type,
    unread: c.unread_count
  })));

  console.groupEnd();

  return duplicates;
}

// Import required stores
import { useChatStore } from '@/stores/chat';
import { useWorkspaceStore } from '@/stores/workspace'; 