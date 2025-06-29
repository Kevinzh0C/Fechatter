# Analytics端到端数据收集实现指南

## 数据流概览

```
前端(JSON) → Analytics Server → ClickHouse
```

## 1. 前端实现

### 1.1 基础Analytics客户端

```javascript
// analytics-client.js
class AnalyticsClient {
  constructor(endpoint = 'http://127.0.0.1:6691/api/event/json') {
    this.endpoint = endpoint;
    this.clientId = this.getOrCreateClientId();
    this.sessionId = this.createSessionId();
  }

  getOrCreateClientId() {
    let clientId = localStorage.getItem('analytics_client_id');
    if (!clientId) {
      clientId = `client_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      localStorage.setItem('analytics_client_id', clientId);
    }
    return clientId;
  }

  createSessionId() {
    return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  async track(eventType, eventData = {}) {
    const payload = {
      context: {
        client_id: this.clientId,
        session_id: this.sessionId,
        user_id: this.getUserId(), // 从auth store获取
        app_version: '1.0.0',
        client_ts: Date.now(),
        user_agent: navigator.userAgent,
        system: {
          os: this.getOS(),
          browser: this.getBrowser(),
          locale: navigator.language,
          timezone: Intl.DateTimeFormat().resolvedOptions().timeZone
        }
      },
      event_type: {
        [eventType]: eventData
      }
    };

    try {
      const response = await fetch(this.endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(payload)
      });

      if (!response.ok) {
        throw new Error(`Analytics failed: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Analytics error:', error);
      // 可以实现离线队列
      this.queueOfflineEvent(payload);
    }
  }

  getUserId() {
    // 从Vuex/Pinia获取
    return store.state.auth?.user?.id || '';
  }

  getOS() {
    const platform = navigator.platform.toLowerCase();
    if (platform.includes('win')) return 'windows';
    if (platform.includes('mac')) return 'macos';
    if (platform.includes('linux')) return 'linux';
    return 'unknown';
  }

  getBrowser() {
    const ua = navigator.userAgent;
    if (ua.includes('Chrome')) return 'chrome';
    if (ua.includes('Firefox')) return 'firefox';
    if (ua.includes('Safari')) return 'safari';
    return 'unknown';
  }

  queueOfflineEvent(event) {
    // 实现离线事件队列
    const queue = JSON.parse(localStorage.getItem('analytics_queue') || '[]');
    queue.push(event);
    localStorage.setItem('analytics_queue', JSON.stringify(queue));
  }
}

export default new AnalyticsClient();
```

### 1.2 Vue集成

```javascript
// main.js
import analytics from './services/analytics-client';

// 全局错误处理
app.config.errorHandler = (error, instance, info) => {
  analytics.track('error_occurred', {
    error_type: error.name,
    error_message: error.message,
    error_stack: error.stack,
    context: info
  });
};

// 路由跟踪
router.afterEach((to, from) => {
  analytics.track('navigation', {
    from: from.path,
    to: to.path,
    duration_ms: performance.now()
  });
});
```

### 1.3 组件中使用

```vue
<script setup>
import analytics from '@/services/analytics-client';

// 用户登录
const handleLogin = async () => {
  try {
    const result = await loginAPI(credentials);
    analytics.track('user_login', {
      email: credentials.email,
      login_method: 'password'
    });
  } catch (error) {
    analytics.track('error_occurred', {
      error_type: 'login_failed',
      error_message: error.message
    });
  }
};

// 消息发送
const sendMessage = async (message) => {
  analytics.track('message_sent', {
    chat_id: currentChatId,
    type: message.type,
    size: message.content.length,
    has_mentions: message.content.includes('@'),
    has_links: /https?:\/\//.test(message.content)
  });
};
</script>
```

## 2. Analytics Server实现

### 2.1 JSON处理模块（已创建）

文件：`analytics_server/src/json_handlers.rs`

主要功能：
- 接收JSON格式的分析事件
- 转换为内部Protobuf格式
- 添加服务端信息（IP、地理位置、服务器时间戳）
- 批量插入ClickHouse

### 2.2 修改handlers.rs导出函数

```rust
// analytics_server/src/handlers.rs
// 在文件末尾添加：
pub(crate) async fn insert_analytics_event(
    state: &AppState, 
    row: &AnalyticsEventRow
) -> Result<(), AppError> {
    let mut insert = state.client.insert("analytics_events")?;
    insert.write(row).await?;
    insert.end().await?;
    Ok(())
}

pub(crate) async fn insert_batch_analytics_events(
    state: &AppState, 
    rows: &[AnalyticsEventRow]
) -> Result<(), AppError> {
    let mut insert = state.client.insert("analytics_events")?;
    for row in rows {
        insert.write(row).await?;
    }
    insert.end().await?;
    Ok(())
}
```

### 2.3 更新main.rs日志

```rust
// analytics_server/src/main.rs
// 在日志输出部分添加JSON端点信息：
info!("📊 Event Ingestion Methods:");
info!("  - HTTP POST: {}/api/event (protobuf)", addr);
info!("  - HTTP POST: {}/api/event/json (JSON)", addr);
info!("  - HTTP POST: {}/api/batch (protobuf batch)", addr);
info!("  - HTTP POST: {}/api/batch/json (JSON batch)", addr);
```

## 3. 部署步骤

### 3.1 构建Analytics Server

```bash
# 构建支持JSON的Analytics Server
cd analytics_server
cargo build --release --target x86_64-unknown-linux-musl

# 复制到服务器
scp target/x86_64-unknown-linux-musl/release/analytics_server \
    root@45.77.178.85:/app/binaries/
```

### 3.2 重启服务

```bash
# SSH到服务器
ssh root@45.77.178.85

# 重启analytics服务
docker restart analytics-server-vcr

# 检查日志
docker logs -f analytics-server-vcr
```

### 3.3 配置前端

```javascript
// 生产环境配置
const ANALYTICS_ENDPOINT = process.env.NODE_ENV === 'production' 
  ? 'http://45.77.178.85:6690/api/event/json'
  : 'http://127.0.0.1:6691/api/event/json';
```

## 4. 测试验证

### 4.1 本地测试

```bash
# 测试JSON端点
curl -X POST http://127.0.0.1:6691/api/event/json \
  -H "Content-Type: application/json" \
  -d '{
    "context": {
      "client_id": "test-client",
      "client_ts": 1719301234567
    },
    "event_type": {
      "app_start": {}
    }
  }'
```

### 4.2 验证ClickHouse数据

```sql
-- 连接到ClickHouse
clickhouse-client

-- 查询最新事件
SELECT 
    event_type,
    client_id,
    created_at
FROM fechatter_analytics.analytics_events
ORDER BY created_at DESC
LIMIT 10;

-- 统计事件类型
SELECT 
    event_type,
    COUNT(*) as count
FROM fechatter_analytics.analytics_events
GROUP BY event_type
ORDER BY count DESC;
```

## 5. 监控和调试

### 5.1 Analytics Server健康检查

```bash
# 健康检查
curl http://45.77.178.85:6690/health

# Metrics
curl http://45.77.178.85:6690/metrics
```

### 5.2 前端调试

```javascript
// 开启调试模式
if (process.env.NODE_ENV === 'development') {
  window.analytics = analytics;
  
  // 监听所有事件
  analytics.on('track', (event) => {
    console.log('[Analytics]', event);
  });
}
```

## 6. 事件类型清单

| 事件类型 | 触发时机 | 数据字段 |
|---------|---------|---------|
| app_start | 应用启动 | 无 |
| user_login | 用户登录 | email, login_method |
| user_logout | 用户登出 | email |
| message_sent | 发送消息 | chat_id, type, size |
| error_occurred | 错误发生 | error_type, message, stack |
| navigation | 路由切换 | from, to, duration_ms |
| file_uploaded | 文件上传 | file_type, file_size |
| search_performed | 执行搜索 | query_length, results_count |

## 7. 优化建议

### 7.1 批量发送

```javascript
// 批量收集事件
class BatchAnalytics extends AnalyticsClient {
  constructor() {
    super();
    this.queue = [];
    this.flushInterval = 5000; // 5秒
    this.batchSize = 20;
    this.startBatchTimer();
  }

  async track(eventType, eventData) {
    this.queue.push({
      context: this.buildContext(),
      event_type: { [eventType]: eventData }
    });

    if (this.queue.length >= this.batchSize) {
      await this.flush();
    }
  }

  async flush() {
    if (this.queue.length === 0) return;

    const events = [...this.queue];
    this.queue = [];

    try {
      await fetch(`${this.endpoint.replace('/event', '/batch')}/json`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ events })
      });
    } catch (error) {
      // 重新加入队列
      this.queue.unshift(...events);
    }
  }
}
```

### 7.2 性能监控

```javascript
// 添加性能指标
analytics.track('performance', {
  page_load_time: performance.timing.loadEventEnd - performance.timing.navigationStart,
  dom_ready_time: performance.timing.domContentLoadedEventEnd - performance.timing.navigationStart,
  first_paint: performance.getEntriesByType('paint')[0]?.startTime
});
```

## 8. 故障排查

### 问题1：前端发送失败
- 检查CORS配置
- 验证端点地址
- 查看浏览器控制台

### 问题2：数据未入库
- 检查Analytics Server日志
- 验证ClickHouse连接
- 确认表结构正确

### 问题3：数据丢失
- 实现前端离线队列
- 使用批量发送减少请求
- 监控服务器metrics

## 完整调用链总结

1. **前端事件触发** → Analytics客户端收集
2. **JSON格式化** → 添加上下文信息
3. **HTTP POST** → Analytics Server JSON端点
4. **服务端解析** → 转换为Protobuf格式
5. **数据增强** → 添加服务端信息
6. **批量写入** → ClickHouse存储
7. **数据分析** → 查询统计报表 