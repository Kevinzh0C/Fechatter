# DM 预创建机制使用示例

## 场景1：从群聊发起私聊

### 步骤：
1. 用户在 #general 频道看到了 Alice 的消息
2. 点击 Alice 的名字，弹出个人资料卡片
3. 点击"Send Message"按钮
4. 立即跳转到与 Alice 的私聊界面（预创建的DM）
5. 输入消息并发送
6. 系统创建真实的聊天记录并发送消息

### 代码流程：
```javascript
// 1. MessageItem.vue - 点击用户名
<span @click="$emit('user-profile-opened', message.sender)">
  {{ message.sender.fullname }}
</span>

// 2. Chat.vue - 处理打开个人资料
function handleUserProfileOpened(user) {
  selectedUserProfile.value = user;
}

// 3. UserProfile.vue - 点击Send Message
const startDirectMessage = async () => {
  const existingDM = await chatStore.findExistingDM(props.user.id);
  
  if (existingDM) {
    emit('dm-created', existingDM);
  } else {
    const tempChat = chatStore.createTemporaryDM(props.user);
    emit('dm-created', tempChat);
  }
}

// 4. Chat.vue - 处理DM创建
async function handleDMCreated(chat) {
  selectedUserProfile.value = null; // 关闭个人资料
  if (chat && chat.id) {
    router.push(`/chat/${chat.id}`); // 跳转到聊天
  }
}
```

## 场景2：管理DM列表

### 隐藏不常用的DM：
```javascript
// ChannelList.vue
function hideDM(dmId) {
  chatStore.hideDM(dmId);
}

// 效果：
// - DM从主列表消失
// - 出现"1 hidden conversations"提示
// - 聊天记录保留
```

### 恢复隐藏的DM：
```javascript
// 1. 点击"N hidden conversations"展开列表
// 2. 找到要恢复的DM
// 3. 点击"Restore"按钮

function unhideDM(dmId) {
  chatStore.unhideDM(dmId);
  showHiddenDMs.value = false;
}
```

## 场景3：处理临时DM

### 发送第一条消息：
```javascript
// Chat.vue - handleSendMessage
if (chat.is_preview) {
  // 找到对方用户
  const otherUser = chat.chat_members.find(m => m.id !== authStore.user.id);
  
  // 创建真实聊天
  const newChat = await chatStore.createChat(
    otherUser.fullname,
    [otherUser.id],
    '',
    'Single'
  );
  
  // 替换路由
  router.replace(`/chat/${newChat.id}`);
  
  // 发送消息
  await chatStore.sendMessage(newChat.id, { content, files });
  
  // 清理临时聊天
  chatStore.removeTemporaryDM(chat.id);
}
```

### 导航离开时清理：
```javascript
// Chat.vue - 路由监听器
watch(() => route.params.id, async (newId, oldId) => {
  if (oldId && String(oldId).startsWith('preview-dm-')) {
    const previousChat = chatStore.getChatById(oldId);
    if (previousChat && previousChat.is_preview) {
      chatStore.removeTemporaryDM(oldId);
    }
  }
});
```

## 实用技巧

### 1. 检查DM是否被隐藏
```javascript
const isHidden = chatStore.isDMHidden(chatId);
```

### 2. 获取所有可见的聊天
```javascript
const visibleChats = chatStore.visibleChats;
```

### 3. 清除所有隐藏的DM
```javascript
chatStore.clearHiddenDMs();
```

### 4. 在组件中使用
```vue
<template>
  <!-- 只显示可见的聊天 -->
  <div v-for="chat in visibleChats" :key="chat.id">
    <!-- 聊天项 -->
  </div>
  
  <!-- 显示隐藏数量 -->
  <div v-if="hiddenDMsCount > 0">
    {{ hiddenDMsCount }} hidden conversations
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { useChatStore } from '@/stores/chat';

const chatStore = useChatStore();

const visibleChats = computed(() => chatStore.visibleChats);
const hiddenDMsCount = computed(() => chatStore.hiddenDMChats.length);
</script>
```

## 最佳实践

1. **性能优化**
   - 使用 `visibleChats` getter 而非手动过滤
   - 避免频繁调用 `hideDM`/`unhideDM`

2. **用户体验**
   - 在隐藏DM前可以添加确认对话框
   - 显示隐藏DM的未读消息数

3. **错误处理**
   ```javascript
   try {
     const tempChat = chatStore.createTemporaryDM(user);
     // ...
   } catch (error) {
     console.error('Failed to create DM:', error);
     // 显示错误提示
   }
   ```

4. **状态同步**
   - 记住隐藏的DM是本地状态，不会跨设备同步
   - 考虑将隐藏状态同步到后端（未来功能） 