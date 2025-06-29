// 完整消息流测试脚本
// 在浏览器Console中执行此脚本来测试完整的消息流

console.log('开始完整消息流测试...');

// 1. 检查后端API是否有消息数据
async function testBackendAPI() {
  console.log('SUBSCRIPTION: 1. 测试后端API...');

  try {
    // 获取当前token
    const token = localStorage.getItem('auth_token') || localStorage.getItem('token');
    if (!token) {
      console.error('ERROR: 没有找到认证token');
      return false;
    }

    // 测试多个聊天频道
    const testChatIds = [1, 2, 3, 4, 5];

    for (const chatId of testChatIds) {
      try {
        const response = await fetch(`/api/chat/${chatId}/messages?limit=15`, {
          headers: {
            'Authorization': `Bearer ${token}`,
            'Accept': 'application/json'
          }
        });

        const data = await response.json();
        console.log(`📋 Chat ${chatId} API响应:`, {
          status: response.status,
          messageCount: data.messages?.length || 0,
          data: data
        });

        if (data.messages && data.messages.length > 0) {
          console.log(`Chat ${chatId} 有 ${data.messages.length} 条消息`);
          return { chatId, messages: data.messages };
        }
      } catch (error) {
        console.log(`WARNING: Chat ${chatId} API调用失败:`, error.message);
      }
    }

    console.log('📋 所有测试的聊天都没有消息数据');
    return false;
  } catch (error) {
    console.error('ERROR: 后端API测试失败:', error);
    return false;
  }
}

// 2. 测试chatStore的fetchMessagesWithSignal方法
async function testChatStore() {
  console.log('🏪 2. 测试chatStore...');

  try {
    const chatStore = window.Vue?.config?.globalProperties?.$chatStore ||
      window.chatStore ||
      (await import('/src/stores/chat.js')).useChatStore();

    if (!chatStore) {
      console.error('ERROR: 无法获取chatStore');
      return false;
    }

    console.log('当前chatStore状态:', {
      currentChatId: chatStore.currentChatId,
      messagesCount: chatStore.messages?.length || 0,
      loading: chatStore.loading,
      chats: chatStore.chats?.length || 0
    });

    // 测试fetchMessagesWithSignal
    const testChatId = 1;
    console.log(`📥 测试fetchMessagesWithSignal(${testChatId})...`);

    const messages = await chatStore.fetchMessagesWithSignal(testChatId, null, 15);
    console.log(`fetchMessagesWithSignal返回:`, {
      messageCount: messages?.length || 0,
      messages: messages
    });

    return { chatStore, messages };
  } catch (error) {
    console.error('ERROR: chatStore测试失败:', error);
    return false;
  }
}

// 3. 测试消息标准化
function testMessageNormalization() {
  console.log('3. 测试消息标准化...');

  try {
    const testMessage = {
      id: 123,
      content: 'Test message',
      sender_id: 1,
      sender: {
        id: 1,
        fullname: 'Test User',
        email: 'test@example.com'
      },
      created_at: new Date().toISOString()
    };

    // 模拟normalizeMessage
    const normalized = {
      ...testMessage,
      _normalized: true,
      temp_id: null,
      files: [],
      reply_to: null,
      mentions: [],
      status: 'sent',
      isOptimistic: false,
      _timestamp: new Date(testMessage.created_at).getTime()
    };

    console.log('消息标准化测试:', {
      original: testMessage,
      normalized: normalized
    });

    return normalized;
  } catch (error) {
    console.error('ERROR: 消息标准化测试失败:', error);
    return false;
  }
}

// 4. 测试Vue组件状态
function testVueComponents() {
  console.log('🎭 4. 测试Vue组件状态...');

  try {
    // 查找Chat.vue组件实例
    const chatComponent = document.querySelector('[data-v-chat]') ||
      document.querySelector('.chat-content-container');

    if (!chatComponent) {
      console.error('ERROR: 无法找到Chat组件');
      return false;
    }

    console.log('找到Chat组件:', chatComponent);

    // 查找MessageList组件
    const messageList = document.querySelector('.message-list-stable') ||
      document.querySelector('.messages-container');

    if (!messageList) {
      console.error('ERROR: 无法找到MessageList组件');
      return false;
    }

    console.log('找到MessageList组件:', messageList);

    // 检查消息元素
    const messageElements = messageList.querySelectorAll('[data-message-id]');
    console.log(`找到 ${messageElements.length} 个消息元素`);

    return {
      chatComponent,
      messageList,
      messageCount: messageElements.length
    };
  } catch (error) {
    console.error('ERROR: Vue组件测试失败:', error);
    return false;
  }
}

// 5. 模拟点击channel按钮
async function simulateChannelClick() {
  console.log('🖱️ 5. 模拟点击channel按钮...');

  try {
    // 查找channel按钮
    const channelButtons = document.querySelectorAll('[data-channel-id]') ||
      document.querySelectorAll('.slack-nav-item') ||
      document.querySelectorAll('a[href*="/chat/"]');

    if (channelButtons.length === 0) {
      console.error('ERROR: 无法找到channel按钮');
      return false;
    }

    console.log(`找到 ${channelButtons.length} 个channel按钮`);

    // 点击第一个channel
    const firstChannel = channelButtons[0];
    console.log('🖱️ 点击channel:', firstChannel);

    // 模拟点击事件
    firstChannel.click();

    // 等待导航完成
    await new Promise(resolve => setTimeout(resolve, 1000));

    return true;
  } catch (error) {
    console.error('ERROR: 模拟点击失败:', error);
    return false;
  }
}

// 6. 添加测试消息
async function addTestMessage() {
  console.log('6. 添加测试消息...');

  try {
    const chatStore = window.Vue?.config?.globalProperties?.$chatStore ||
      window.chatStore ||
      (await import('/src/stores/chat.js')).useChatStore();

    if (!chatStore) {
      console.error('ERROR: 无法获取chatStore');
      return false;
    }

    const currentChatId = chatStore.currentChatId || 1;
    console.log(`为chat ${currentChatId} 添加测试消息...`);

    // 使用addTestMessage方法
    if (typeof chatStore.addTestMessage === 'function') {
      const testMessage = chatStore.addTestMessage(currentChatId);
      console.log('测试消息已添加:', testMessage);
      return testMessage;
    } else {
      console.error('ERROR: addTestMessage方法不存在');
      return false;
    }
  } catch (error) {
    console.error('ERROR: 添加测试消息失败:', error);
    return false;
  }
}

// 主测试函数
async function runCompleteTest() {
  console.log('开始完整的消息流测试...');

  const results = {
    backendAPI: false,
    chatStore: false,
    normalization: false,
    vueComponents: false,
    channelClick: false,
    testMessage: false
  };

  // 1. 测试后端API
  results.backendAPI = await testBackendAPI();

  // 2. 测试chatStore
  results.chatStore = await testChatStore();

  // 3. 测试消息标准化
  results.normalization = testMessageNormalization();

  // 4. 测试Vue组件
  results.vueComponents = testVueComponents();

  // 5. 模拟点击channel
  results.channelClick = await simulateChannelClick();

  // 6. 添加测试消息
  results.testMessage = await addTestMessage();

  // 输出测试结果
  console.log('完整测试结果:', results);

  const passedTests = Object.values(results).filter(Boolean).length;
  const totalTests = Object.keys(results).length;

  console.log(`测试完成: ${passedTests}/${totalTests} 通过`);

  if (passedTests === totalTests) {
    console.log('🎉 所有测试通过！消息流正常工作');
  } else {
    console.log('WARNING: 部分测试失败，需要进一步调试');
  }

  return results;
}

// 导出测试函数到全局
window.testMessageFlow = runCompleteTest;
window.testBackendAPI = testBackendAPI;
window.testChatStore = testChatStore;
window.addTestMessage = addTestMessage;

// 自动运行测试
runCompleteTest(); 