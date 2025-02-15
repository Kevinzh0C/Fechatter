# SSE Connection Management Analysis

## 🔍 问题分析：为什么会达到Gateway的最大连接数

### 1. **前端SSE连接管理问题**

#### 当前实现的问题
```javascript
// src/services/sse.js
class RealtimeCommunicationService {
  constructor() {
    this.eventSource = null;
    this.maxReconnectAttempts = 5;
    this.reconnectDelay = 3000;
    // ...
  }
}
```

**问题点**：
1. **单例模式但可能多次实例化**：虽然导出的是单例，但在不同组件中可能被多次初始化
2. **没有连接去重**：同一用户可能创建多个SSE连接
3. **页面刷新不清理**：旧连接可能没有正确关闭
4. **标签页切换问题**：每个标签页都会创建新连接

### 2. **SSE连接触发时机分析**

#### 调用链追踪
```
用户登录
  ↓
useHomeLayout.js: connectSSE()
  ↓
channel-preloader.js: preloadSSEConnection()
  ↓
sse.js: connect(token)
  ↓
new EventSource(url)
```

**多重触发点**：
1. `useHomeLayout.js` - Home页面加载时
2. `channel-preloader.js` - 预加载时
3. 组件重新渲染时可能重复调用

### 3. **后端连接限制**

#### Gateway配置
```rust
// fechatter_gateway/src/config.rs
max_connections: Some(10000), // 生产环境
max_connections: Some(100),    // 测试环境
```

#### Notify Server
```rust
// notify_server没有明确的连接数限制
// 使用DashMap存储所有用户连接
pub user_connections: Arc<DashMap<UserId, broadcast::Sender<Arc<NotifyEvent>>>>
```

## 🐛 根本原因

### 1. **连接泄漏**
- 用户刷新页面时，旧的EventSource没有正确关闭
- 浏览器标签页关闭时，连接可能保持活跃状态
- 网络断开重连时，可能创建新连接而不关闭旧连接

### 2. **重复连接**
- 同一用户在多个标签页打开应用
- 组件重新挂载时重复创建连接
- 预加载和实际使用时分别创建连接

### 3. **缺少连接管理机制**
- 前端没有全局连接管理器
- 后端没有限制每个用户的连接数
- 没有连接超时清理机制

## 🔧 解决方案

### 1. **前端改进**

#### A. 使用SharedWorker或BroadcastChannel
```javascript
// 所有标签页共享一个SSE连接
class SharedSSEManager {
  constructor() {
    if (typeof SharedWorker !== 'undefined') {
      this.worker = new SharedWorker('/sse-worker.js');
      this.worker.port.start();
    } else {
      // Fallback to BroadcastChannel
      this.channel = new BroadcastChannel('sse-events');
    }
  }
}
```

#### B. 连接去重
```javascript
// 使用全局管理器确保单一连接
import sseGlobalManager from '@/utils/SSEGlobalManager';

// 在连接前检查
if (sseGlobalManager.hasActiveConnection()) {
  return sseGlobalManager.getConnection();
}
```

#### C. 生命周期管理
```javascript
// 页面卸载时清理
window.addEventListener('beforeunload', () => {
  if (this.eventSource) {
    this.eventSource.close();
    this.eventSource = null;
  }
});

// 页面隐藏时断开
document.addEventListener('visibilitychange', () => {
  if (document.hidden && this.eventSource) {
    this.disconnect();
  }
});
```

### 2. **后端改进**

#### A. 用户连接数限制
```rust
// 每个用户最多1个活跃连接
if state.user_connections.contains_key(&user_id) {
    // 关闭旧连接
    if let Some(old_tx) = state.user_connections.remove(&user_id) {
        drop(old_tx); // 触发旧连接关闭
    }
}
```

#### B. 连接超时清理
```rust
// 定期清理无活动的连接
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5分钟
    loop {
        interval.tick().await;
        cleanup_inactive_connections(&state).await;
    }
});
```

### 3. **连接数计算**

#### 理想情况
- **200日活用户**：最多200个并发连接
- **1000用户**：假设20%同时在线 = 200个连接

#### 实际情况（当前问题）
- **200日活用户**：可能产生 200 × 3-5 = 600-1000个连接
  - 多标签页：×2-3
  - 连接泄漏：×1.5-2
  
#### 优化后
- **200日活用户**：200个连接（每用户1个）
- **1000用户**：200个并发连接

## 📊 监控指标

### 前端监控
```javascript
// 添加连接监控
class SSEMetrics {
  static connections = new Map();
  
  static trackConnection(userId, connectionId) {
    const userConnections = this.connections.get(userId) || [];
    userConnections.push({
      id: connectionId,
      startTime: Date.now(),
      tabId: this.getTabId()
    });
    this.connections.set(userId, userConnections);
    
    // 警告：同一用户超过2个连接
    if (userConnections.length > 2) {
      console.warn(`User ${userId} has ${userConnections.length} SSE connections!`);
    }
  }
}
```

### 后端监控
```rust
// Prometheus metrics
static ref SSE_CONNECTIONS: IntGaugeVec = register_int_gauge_vec!(
    "sse_active_connections",
    "Number of active SSE connections",
    &["user_id"]
).unwrap();

static ref SSE_CONNECTION_DURATION: HistogramVec = register_histogram_vec!(
    "sse_connection_duration_seconds",
    "SSE connection duration",
    &["status"]
).unwrap();
```

## 🚀 实施计划

### Phase 1: 快速修复（1天）
1. 添加 `beforeunload` 事件处理
2. 实现连接去重检查
3. 修复组件重复初始化问题

### Phase 2: 优化（3天）
1. 实现 SharedWorker/BroadcastChannel
2. 添加连接监控
3. 后端添加用户连接数限制

### Phase 3: 长期改进（1周）
1. 实现完整的连接池管理
2. 添加自动重连退避算法
3. 实现连接健康检查机制

## 📝 总结

当前SSE连接管理存在严重的连接泄漏和重复连接问题，导致实际连接数是预期的3-5倍。通过实施上述解决方案，可以将连接数控制在合理范围内，提高系统稳定性和可扩展性。 