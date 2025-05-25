# Fechatter 搜索策略设计

## 🎯 **搜索场景定位**

Fechatter 专注于**聊天内搜索**，即用户在特定聊天中查找历史消息，而非跨聊天的全局搜索。

## 🔍 **搜索策略**

### **1. 群聊搜索策略**

**使用场景**：用户在工作空间的群聊中查找消息

**权限验证**：

- ✅ 验证用户是聊天成员
- ✅ 验证聊天属于用户的工作空间
- ✅ 强制 workspace 过滤

**搜索范围**：

```
chat_id = <指定群聊ID> AND workspace_id = <用户工作空间ID>
```

### **2. 私聊搜索策略**

**使用场景**：用户在私聊中查找消息

**权限验证**：

- ✅ 验证用户是私聊参与者
- ❌ 不验证工作空间（私聊可能跨工作空间）
- 🔄 自动使用聊天所属的工作空间ID

**搜索范围**：

```
chat_id = <指定私聊ID> AND workspace_id = <聊天所属工作空间ID>
```

## 📋 **搜索字段**

### **可搜索字段**


| 字段          | 说明       | 示例          |
| ------------- | ---------- | ------------- |
| `content`     | 消息内容   | "API设计讨论" |
| `sender_name` | 发送者姓名 | "Alice"       |
| `file_names`  | 文件名     | "设计稿.pdf"  |

### **过滤字段**


| 字段           | 说明       | 必需性   |
| -------------- | ---------- | -------- |
| `chat_id`      | 聊天ID     | 必需     |
| `workspace_id` | 工作空间ID | 自动设置 |
| `sender_id`    | 发送者ID   | 可选     |
| `created_at`   | 消息时间   | 可选     |

## 🎨 **Facets设计**

### **有意义的Facets**

- ✅ **时间分布**：查看消息的时间模式
- ✅ **发送者统计**：查看谁发送了最多相关消息

### **移除的Facets**

- ❌ **聊天类型**：在聊天内搜索时无意义

## 🔐 **权限模型**

```rust
// 群聊权限检查
if chat_type == Group {
    verify_user_workspace(user.workspace_id, request.workspace_id)?;
    verify_chat_workspace(chat.workspace_id, user.workspace_id)?;
}

// 私聊权限检查  
if chat_type == Single {
    // 覆盖请求中的workspace_id以确保数据一致性
    request.workspace_id = chat.workspace_id;
}

// 通用权限检查
verify_user_chat_access(user.id, chat_id)?;
```

## 🚀 **API使用示例**

### **在群聊中搜索**

```http
POST /api/chat/123/messages/search
Content-Type: application/json

{
  "query": "API设计",
  "workspace_id": 1,
  "search_type": "fulltext",
  "limit": 20
}
```

### **在私聊中搜索**

```http
POST /api/chat/456/messages/search  
Content-Type: application/json

{
  "query": "文档.pdf",
  "workspace_id": 1,  // 会被自动覆盖为聊天所属工作空间
  "search_type": "fulltext",
  "limit": 20
}
```

## 🎯 **设计优势**

1. **明确的搜索边界**：用户明确知道在哪个聊天中搜索
2. **简化的权限模型**：基于聊天类型的差异化处理
3. **优化的用户体验**：专注于聊天内容而非复杂的跨聊天搜索
4. **高效的索引策略**：针对聊天内搜索优化的字段选择

这种设计确保了搜索功能既强大又易用，同时保持了良好的性能和安全性。
