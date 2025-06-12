# Strict Channel-Message Correspondence Fix DAG

## 核心问题
消息必须严格对应到其所属的channel，这是软件的根基。任何消息显示在错误channel中都是不可接受的。

## 修复链条

### 1. 问题识别层
```
用户报告: "消息显示在错误的channel中"
    ↓
根因分析:
    - 所有channel共享同一个messages数组
    - 缓存可能包含错误channel的消息
    - 没有验证机制确保消息属于当前channel
    - 扩展干扰导致请求失败
```

### 2. 验证层实现
```
创建 strictChannelMessageValidator.js
    ↓
功能:
    - validateMessage(message, expectedChatId)
    - validateMessageArray(messages, expectedChatId) 
    - 记录所有违规行为
    - 抛出错误阻止错误显示
```

### 3. Store层强化
```
修改 chat.js - fetchMessages方法:
    ↓
1. 验证chatId有效性
    const validChatId = parseInt(chatId, 10)
    ↓
2. 从缓存加载前验证
    strictChannelMessageValidator.validateMessageArray(cachedMessages, validChatId)
    ↓
3. API响应后立即验证
    messagesData.forEach(msg => {
        strictChannelMessageValidator.validateMessage(msg, validChatId)
    })
    ↓
4. 最终加入store前再次验证
    normalizedMessages.forEach(msg => {
        strictChannelMessageValidator.validateMessage(msg, validChatId)
        this.messages.push(msg)
    })
```

### 4. 实时消息验证
```
修改 handleIncomingMessage方法:
    ↓
1. 验证消息属于正确的chat
    strictChannelMessageValidator.validateMessage(normalizedMessage, chat.id)
    ↓
2. 添加到当前消息列表前验证
    strictChannelMessageValidator.validateMessage(normalizedMessage, this.currentChatId)
```

### 5. 扩展干扰阻断
```
创建 aggressiveExtensionBlocker.js
    ↓
功能:
    - 拦截fetch请求，阻止扩展请求
    - 拦截XHR请求
    - 监控DOM变化，移除扩展元素
    - 创建隔离的通信通道
```

### 6. 验证流程
```
每条消息的生命周期:
    
API获取 → 验证chat_id → 规范化 → 再次验证 → 加入store
    ↓ (任何步骤失败)
    拒绝消息，记录违规

缓存加载 → 验证所有消息 → 过滤无效消息 → 显示
    ↓ (发现无效消息)
    清除缓存，重新获取

实时消息 → 验证chat归属 → 验证是否当前chat → 显示
    ↓ (验证失败)
    丢弃消息，记录错误
```

### 7. 测试验证
```
testStrictChannelValidation():
    ↓
1. 遍历多个channel
2. 切换到每个channel
3. 验证所有消息的chat_id
4. 记录任何违规
5. 生成测试报告
```

### 8. 紧急修复
```
fixCurrentChannel():
    ↓
1. 清空当前消息
2. 清除该channel的缓存
3. 重置currentChatId
4. 重新加载消息
5. 验证所有消息正确
```

## 数据流保证

### 消息加载保证
```
setCurrentChat(chatId)
    ↓
this.currentChatId = chatId
    ↓
fetchMessages(chatId)
    ↓
清空 this.messages
    ↓
验证并加载新消息
    ↓
确保 所有 message.chat_id === chatId
```

### 缓存一致性保证
```
messageCache[chatId] = {
    messages: [...],  // 只包含chat_id === chatId的消息
    validated: true,  // 标记已验证
    timestamp: Date.now()
}
```

### 实时更新保证
```
SSE消息到达
    ↓
验证 message.chat_id === expectedChatId
    ↓
如果是当前chat → 再次验证 === currentChatId
    ↓
只有双重验证通过才显示
```

## 监控点

1. **验证统计**
   - window.channelMessageValidator.getReport()
   - 查看所有违规记录

2. **扩展阻断统计**
   - window.extensionBlocker.getStats()
   - 查看被阻断的请求

3. **消息状态**
   - chatStore.debugMessageState()
   - 查看当前消息状态

## 性能影响

- 每条消息增加2-3次验证：可忽略（< 1ms）
- 扩展阻断：提升性能（减少干扰）
- 整体影响：正面（提高稳定性）

## 回滚方案

如需临时禁用验证：
```javascript
window.channelMessageValidator.setEnabled(false)
window.extensionBlocker.reset()
```

## 长期改进

1. **消息存储隔离**
   ```javascript
   state: {
     messagesByChat: new Map() // chatId -> messages[]
   }
   ```

2. **类型安全**
   - 使用TypeScript确保chat_id类型一致性

3. **后端验证**
   - API层面确保不会返回错误channel的消息

## 成功标准

- ✅ 任何channel切换后，只显示该channel的消息
- ✅ 缓存加载的消息经过验证
- ✅ 实时消息只出现在正确的channel
- ✅ 扩展干扰被有效阻断
- ✅ 违规行为被记录和报告 