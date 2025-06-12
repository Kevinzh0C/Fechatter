# 立即诊断指南
*在浏览器控制台执行*

## 1. 完整链条诊断
```javascript
// 运行完整的消息链条测试
window.testMessageChain()
```

## 2. 特定聊天室3测试  
```javascript
// 测试聊天室3的消息加载
window.testChatLoading(3)
```

## 3. 当前状态检查
```javascript
// 检查当前聊天store状态
const { useChatStore } = await import('/src/stores/chat.js');
const chatStore = useChatStore();
console.log('Current Chat ID:', chatStore.currentChatId);
console.log('Messages Count:', chatStore.messages.length);
console.log('Chats Count:', chatStore.chats.length);
```

## 4. 手动触发消息获取
```javascript
// 手动获取聊天室3的消息
const { useChatStore } = await import('/src/stores/chat.js');
const chatStore = useChatStore();
try {
  const messages = await chatStore.fetchMessages(3);
  console.log('Fetched messages:', messages.length);
} catch (error) {
  console.error('Fetch failed:', error);
}
```

## 5. 检查Chat.vue组件状态
```javascript
// 检查当前Vue组件状态
const app = document.querySelector('#app').__vue_app__;
const instance = app._instance;
console.log('Current route:', instance.proxy.$route.path);
console.log('Route params:', instance.proxy.$route.params);
```

## 期望结果
- 如果后端正常：应该看到消息数据
- 如果后端问题：应该看到网络错误
- 如果前端问题：应该看到组件或store错误 