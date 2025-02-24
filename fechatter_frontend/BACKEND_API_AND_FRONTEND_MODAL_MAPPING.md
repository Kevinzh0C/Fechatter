# 📋 Backend API与前端Modal搜索选项映射表

## **后端API完整列表**

基于 `fechatter_server/src/lib.rs` 分析的实际可用API：

| API端点 | 方法 | 功能描述 | 权限要求 | 参数 |
|---------|------|----------|----------|------|
| `/search/messages` | `POST` | **全局消息搜索** | 需要认证 | `q`, `limit`, `offset`, `strategy` |
| `/search/chat/{chat_id}/messages` | `GET` | **简化聊天搜索** | 需要认证 | `q`, `limit` |
| `/chat/{id}/messages/search` | `GET` | **简单聊天搜索** | 需要聊天成员权限 | `q` |
| `/chat/{id}/messages/search` | `POST` | **高级聊天搜索** | 需要聊天成员权限 | `strategy`, `filters`, `q`, `limit`, `offset` |
| `/admin/chat/{id}/reindex` | `POST` | **重新索引聊天** | 需要管理员权限 | 无 |

---

## **前端Modal搜索选项配置**

### **🎯 当前优化后的搜索过滤器 (基于真实API)**

```javascript
const quickFilters = [
  { 
    label: 'All Messages', 
    value: 'all', 
    strategy: 'full_text',
    description: '使用全文搜索所有消息',
    apiEndpoint: chatId ? '/chat/{id}/messages/search' : '/search/messages'
  },
  { 
    label: 'Semantic', 
    value: 'semantic', 
    strategy: 'semantic',
    description: '基于语义理解的智能搜索',
    apiEndpoint: chatId ? '/chat/{id}/messages/search' : '/search/messages'
  },
  { 
    label: 'Exact Match', 
    value: 'exact', 
    strategy: 'exact',
    description: '精确匹配搜索词',
    apiEndpoint: chatId ? '/chat/{id}/messages/search' : '/search/messages'
  },
  { 
    label: 'Recent', 
    value: 'recent', 
    strategy: 'temporal',
    description: '按时间权重搜索最近消息',
    apiEndpoint: chatId ? '/chat/{id}/messages/search' : '/search/messages'
  }
];
```

### **❌ 移除的过时选项 (无对应后端支持)**

- ~~`Files`~~ - 后端无独立文件搜索API
- ~~`Links`~~ - 后端无链接专用搜索API
- ~~`Users`~~ - 通过 `user_scoped` 策略实现

### **🔧 高级过滤器映射**

| 前端过滤器 | 后端参数 | API支持 | 功能描述 |
|-----------|----------|---------|----------|
| `timeRange` | `time_weight`, `sort=relevance_time` | ✅ POST搜索 | 时间范围过滤 |
| `sender` | `include_sender=true` | ✅ POST搜索 | 发送者过滤 |
| `exactMatch` | `exact_match=true` | ✅ POST搜索 | 精确匹配 |
| `fuzzySearch` | `max_edits`, `min_similarity` | ✅ POST搜索 | 模糊搜索 |
| `semanticSearch` | `similarity_threshold`, `include_context` | ✅ POST搜索 | 语义搜索 |

---

## **🎨 前端Modal UI组件重构对照**

### **重构前 vs 重构后**

| 组件部分 | 重构前 | 重构后 | 改进点 |
|----------|--------|--------|--------|
| **搜索输入框** | 左侧图标 + 输入框 | 左侧图标 + 输入框 + **右侧搜索按钮** | ✅ 用户体验提升 |
| **过滤器选项** | `All`, `Recent`, `Files`, `Links` | `All Messages`, `Semantic`, `Exact Match`, `Recent` | ✅ 匹配后端API |
| **搜索结果** | 简单列表 | **精美卡片容器** | ✅ 现代化设计 |
| **API调用** | 硬编码端点 | **统一端点 + 策略参数** | ✅ 架构一致性 |

### **新增搜索按钮功能**

```vue
<!-- 🔧 NEW: Search Action Button on the right -->
<button
  v-if="searchQuery.trim() && searchQuery.length >= 2"
  @click="handleSearch"
  class="search-action-button"
  :disabled="isSearching"
  type="button"
  aria-label="Search"
>
  <svg v-if="!isSearching" class="search-button-icon">...</svg>
  <svg v-else class="search-button-icon animate-spin">...</svg>
</button>
```

### **精美搜索结果容器**

每个搜索结果现在显示为独立的卡片，包含：

- **头部**: 用户头像 + 姓名 + 时间戳 + 相关性评分
- **内容**: 高亮显示的消息内容 + 文件信息 + 上下文
- **底部**: 预览/复制/跳转操作按钮

---

## **🔄 API调用流程优化**

### **新的搜索流程**

```mermaid
graph TD
    A[用户选择过滤器] --> B[获取策略参数]
    B --> C{聊天内 vs 全局搜索}
    C -->|聊天内| D[POST /chat/{id}/messages/search]
    C -->|全局| E[POST /search/messages]
    D --> F[传递策略参数]
    E --> F
    F --> G[后端处理策略]
    G --> H[返回统一格式结果]
    H --> I[前端渲染精美容器]
```

### **参数传递示例**

```javascript
// 语义搜索请求
const searchParams = {
  q: "project meeting",
  strategy: "semantic",
  similarity_threshold: 0.7,
  include_context: true,
  limit: 20,
  offset: 0
};

// 精确匹配请求  
const searchParams = {
  q: "important deadline",
  strategy: "exact",
  exact_match: true,
  limit: 20,
  offset: 0
};
```

---

## **✅ 验证检查表**

### **后端API兼容性**
- [x] 使用实际存在的API端点
- [x] 传递正确的参数格式
- [x] 统一错误处理机制
- [x] 支持降级到本地搜索

### **前端UI现代化**
- [x] 搜索按钮位置优化 (右侧)
- [x] 搜索结果卡片化设计
- [x] 响应式布局支持
- [x] 无障碍访问支持

### **用户体验**
- [x] 智能占位符文字
- [x] 实时搜索状态反馈
- [x] 键盘快捷键支持
- [x] 移动端触控优化

---

## **🎯 最终配置总结**

| 配置项 | 值 | 说明 |
|--------|---|------|
| **主要API端点** | `/chat/{id}/messages/search` (POST) | 聊天内高级搜索 |
| **全局API端点** | `/search/messages` (POST) | 全局消息搜索 |
| **支持策略** | `full_text`, `semantic`, `exact`, `temporal` | 四种主要搜索策略 |
| **过滤器数量** | 4个 | 基于实际API能力 |
| **搜索按钮位置** | 输入框右侧 | 符合用户习惯 |
| **结果显示** | 卡片容器 | 现代化设计 |

**这个映射确保了前端Modal选项与后端API的完美对应，提供了统一、现代、高效的搜索体验。** 