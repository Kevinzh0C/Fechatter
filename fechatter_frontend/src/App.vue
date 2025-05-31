<template>
  <div id="app" class="h-screen bg-gray-100">
    <div class="flex h-full">
      <!-- 侧边栏 -->
      <div class="w-64 bg-white border-r border-gray-200 flex flex-col">
        <div class="p-4 border-b border-gray-200">
          <h1 class="text-xl font-bold text-gray-800">Fechatter</h1>
        </div>
        <div class="flex-1 overflow-y-auto">
          <div class="p-2">
            <div 
              v-for="chat in mockChats" 
              :key="chat.id"
              @click="selectChat(chat)"
              :class="[
                'p-3 rounded-lg cursor-pointer transition-colors',
                selectedChat?.id === chat.id ? 'bg-blue-100' : 'hover:bg-gray-100'
              ]">
              <div class="font-medium text-gray-900">{{ chat.name }}</div>
              <div class="text-sm text-gray-500 truncate">{{ chat.lastMessage }}</div>
            </div>
          </div>
        </div>
      </div>
      
      <!-- 主聊天区域 -->
      <div class="flex-1 flex flex-col">
        <div v-if="selectedChat" class="flex-1 flex flex-col">
          <!-- 聊天头部 -->
          <div class="p-4 border-b border-gray-200 bg-white">
            <h2 class="text-lg font-semibold text-gray-800">{{ selectedChat.name }}</h2>
          </div>
          
          <!-- 消息列表 -->
          <MessageList 
            :messages="currentMessages"
            :current-user-id="currentUserId"
            :loading="loading"
            :has-more="false"
            @reply-message="handleReplyMessage"
          />
          
          <!-- 消息输入 -->
          <div class="p-4 border-t border-gray-200 bg-white">
            <div class="flex space-x-2">
              <input 
                v-model="newMessage"
                @keydown.enter="sendMessage"
                placeholder="输入消息..."
                class="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500">
              <button 
                @click="sendMessage"
                class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500">
                发送
              </button>
            </div>
          </div>
        </div>
        
        <!-- 未选择聊天时的占位符 -->
        <div v-else class="flex-1 flex items-center justify-center">
          <div class="text-center">
            <h3 class="text-lg font-medium text-gray-900 mb-2">欢迎使用 Fechatter</h3>
            <p class="text-gray-500">选择一个聊天开始对话</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import MessageList from './components/chat/MessageList.vue';

// 当前用户ID
const currentUserId = ref(1);

// 模拟聊天数据
const mockChats = ref([
  {
    id: 1,
    name: '技术讨论组',
    lastMessage: '大家对新的架构有什么看法？'
  },
  {
    id: 2,
    name: '项目团队',
    lastMessage: '明天的会议改到下午3点'
  },
  {
    id: 3,
    name: '设计师小组',
    lastMessage: '新的UI设计稿已经完成'
  }
]);

// 模拟消息数据
const mockMessages = ref({
  1: [
    {
      id: 1,
      content: '大家好，我想讨论一下新的系统架构',
      sender: { id: 2, username: 'Alice' },
      created_at: new Date(Date.now() - 3600000).toISOString(),
      status: 'read'
    },
    {
      id: 2,
      content: '听起来很有趣，能详细说说吗？',
      sender: { id: 1, username: 'You' },
      created_at: new Date(Date.now() - 3000000).toISOString(),
      status: 'read'
    },
    {
      id: 3,
      content: '我们可以考虑使用微服务架构，这样可以提高系统的可扩展性和维护性。',
      sender: { id: 2, username: 'Alice' },
      created_at: new Date(Date.now() - 2400000).toISOString(),
      status: 'read'
    },
    {
      id: 4,
      content: '同意！微服务确实是个好选择。我们还需要考虑数据一致性的问题。',
      sender: { id: 3, username: 'Bob' },
      created_at: new Date(Date.now() - 1800000).toISOString(),
      status: 'read'
    },
    {
      id: 5,
      content: '关于数据一致性，我们可以使用事件驱动的架构模式。',
      sender: { id: 1, username: 'You' },
      created_at: new Date(Date.now() - 1200000).toISOString(),
      status: 'read'
    }
  ],
  2: [
    {
      id: 6,
      content: '明天的会议需要改时间',
      sender: { id: 4, username: 'Manager' },
      created_at: new Date(Date.now() - 7200000).toISOString(),
      status: 'read'
    },
    {
      id: 7,
      content: '改到什么时候？',
      sender: { id: 1, username: 'You' },
      created_at: new Date(Date.now() - 6600000).toISOString(),
      status: 'read'
    },
    {
      id: 8,
      content: '下午3点，会议室还是原来的地方',
      sender: { id: 4, username: 'Manager' },
      created_at: new Date(Date.now() - 6000000).toISOString(),
      status: 'read'
    }
  ],
  3: [
    {
      id: 9,
      content: '新的UI设计稿已经完成了',
      sender: { id: 5, username: 'Designer' },
      created_at: new Date(Date.now() - 5400000).toISOString(),
      status: 'read'
    },
    {
      id: 10,
      content: '太棒了！可以发给我看看吗？',
      sender: { id: 1, username: 'You' },
      created_at: new Date(Date.now() - 4800000).toISOString(),
      status: 'read'
    }
  ]
});

// 状态
const selectedChat = ref(null);
const newMessage = ref('');
const loading = ref(false);

// 计算属性
const currentMessages = computed(() => {
  if (!selectedChat.value) return [];
  return mockMessages.value[selectedChat.value.id] || [];
});

// 方法
const selectChat = (chat) => {
  selectedChat.value = chat;
};

const sendMessage = () => {
  if (!newMessage.value.trim() || !selectedChat.value) return;
  
  const message = {
    id: Date.now(),
    content: newMessage.value.trim(),
    sender: { id: currentUserId.value, username: 'You' },
    created_at: new Date().toISOString(),
    status: 'sent'
  };
  
  if (!mockMessages.value[selectedChat.value.id]) {
    mockMessages.value[selectedChat.value.id] = [];
  }
  
  mockMessages.value[selectedChat.value.id].push(message);
  newMessage.value = '';
};

const handleReplyMessage = (message) => {
  console.log('Reply to message:', message);
  // TODO: 实现回复功能
};

// 初始选择第一个聊天
if (mockChats.value.length > 0) {
  selectedChat.value = mockChats.value[0];
}
</script>

<style>
/* 引入自定义CSS */
@import './style.css';

/* 应用特定样式 */
#app {
  height: 100vh;
  overflow: hidden;
}
</style>
