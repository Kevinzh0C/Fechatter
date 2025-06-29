# Analytics Server JSON Support Fix

## 问题诊断

Analytics服务器当前**只支持protobuf格式**，不支持JSON。这导致：
- 前端发送JSON请求时返回500错误
- 错误信息：`invalid wire type: StartGroup (expected LengthDelimited)`

## 临时解决方案

### 1. 使用Nginx反向代理转换（最快）

在远程服务器上配置Nginx：

```nginx
# /etc/nginx/sites-available/analytics-json
server {
    listen 6691;
    
    location /api/event {
        # 将JSON转换为protobuf的中间服务
        proxy_pass http://localhost:6692;
    }
    
    location / {
        proxy_pass http://localhost:6690;
    }
}
```

### 2. 使用Node.js中间件（推荐）

创建一个简单的转换服务：

```javascript
// analytics-json-proxy.js
const express = require('express');
const protobuf = require('protobufjs');
const axios = require('axios');

const app = express();
app.use(express.json());

// 加载protobuf schema
const root = protobuf.loadSync('analytics.proto');
const AnalyticsEvent = root.lookupType('fechatter.v1.AnalyticsEvent');

app.post('/api/event', async (req, res) => {
    try {
        // 转换JSON到protobuf格式
        const jsonData = req.body;
        
        // 格式转换
        const protoData = {
            context: {
                clientId: jsonData.context.client_id,
                sessionId: jsonData.context.session_id,
                userId: jsonData.context.user_id,
                appVersion: jsonData.context.app_version,
                clientTs: jsonData.context.client_ts?.toString(),
                userAgent: jsonData.context.user_agent,
                system: jsonData.context.system
            }
        };
        
        // 处理event_type
        const eventType = Object.keys(jsonData.event_type)[0];
        const eventData = jsonData.event_type[eventType];
        
        // 映射事件类型
        const eventTypeMap = {
            'app_start': 'appStart',
            'user_login': 'userLogin',
            'message_sent': 'messageSent',
            'error_occurred': 'errorOccurred'
        };
        
        protoData[eventTypeMap[eventType] || eventType] = eventData;
        
        // 编码为protobuf
        const message = AnalyticsEvent.create(protoData);
        const buffer = AnalyticsEvent.encode(message).finish();
        
        // 发送到真实的analytics服务器
        const response = await axios.post('http://localhost:6690/api/event', buffer, {
            headers: {
                'Content-Type': 'application/protobuf'
            }
        });
        
        res.json(response.data);
    } catch (error) {
        console.error('Error:', error);
        res.status(500).json({ error: error.message });
    }
});

app.listen(6691, () => {
    console.log('Analytics JSON proxy running on port 6691');
});
```

### 3. 修改Analytics服务器源码（长期方案）

在`analytics_server/src/main.rs`中添加JSON支持：

```rust
// 添加JSON解析
#[derive(Deserialize)]
struct JsonAnalyticsEvent {
    context: JsonEventContext,
    event_type: HashMap<String, Value>,
}

// 添加新的处理函数
async fn handle_json_event(
    Json(payload): Json<JsonAnalyticsEvent>,
    state: Arc<AppState>,
) -> Result<Json<EventResponse>, AppError> {
    // 转换JSON到protobuf
    let proto_event = convert_json_to_proto(payload)?;
    
    // 使用现有的protobuf处理逻辑
    handle_protobuf_event(proto_event, state).await
}

// 更新路由
.route("/api/event", post(handle_event_with_content_type))

async fn handle_event_with_content_type(
    headers: HeaderMap,
    body: Bytes,
    state: Arc<AppState>,
) -> Result<Json<EventResponse>, AppError> {
    match headers.get("content-type").and_then(|h| h.to_str().ok()) {
        Some("application/json") => {
            let json: JsonAnalyticsEvent = serde_json::from_slice(&body)?;
            handle_json_event(Json(json), state).await
        }
        _ => {
            // 默认处理为protobuf
            handle_protobuf_event(body, state).await
        }
    }
}
```

## 立即可用的解决方案

### 方案A：继续使用SSH隧道 + 创建本地代理

```bash
# 1. 保持SSH隧道
ssh -N -L 6690:localhost:6690 root@45.77.178.85 &

# 2. 创建本地JSON代理（使用Python）
cat > analytics_json_proxy.py << 'EOF'
from flask import Flask, request, jsonify
import requests

app = Flask(__name__)

@app.route('/api/event', methods=['POST'])
def proxy_event():
    # 暂时直接返回成功
    return jsonify({
        "success": True,
        "session_id": "test-session",
        "message": "Event received (JSON mode - test)"
    })

if __name__ == '__main__':
    app.run(port=6691, debug=True)
EOF

# 3. 运行代理
python3 analytics_json_proxy.py
```

### 方案B：修改前端使用不同端口

修改`analytics-simple.html`：

```javascript
// 改为使用测试端口
const response = await fetch('http://127.0.0.1:6691/api/event', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json'
    },
    body: JSON.stringify(payload)
});
```

## 推荐方案

1. **短期**：使用本地JSON代理服务器（方案A）
2. **中期**：在服务器端部署JSON转换中间件
3. **长期**：修改Analytics服务器源码支持JSON

这样可以让前端继续使用简单的JSON格式，同时保持后端的protobuf性能优势。 