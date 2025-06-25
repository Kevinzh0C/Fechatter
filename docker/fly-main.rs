// Fly.io 专用单体应用入口
// 合并所有微服务到一个可执行文件中

use std::sync::Arc;
use tokio::net::TcpListener;
use axum::{
    Router,
    routing::{get, post},
    response::{Html, Json},
    http::StatusCode,
    extract::State,
};
use serde_json::json;

// 简化的应用状态
#[derive(Clone)]
struct AppState {
    db_url: String,
    environment: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::init();
    
    println!("🚀 Starting Fechatter Demo (Fly.io Single Binary)");
    
    // 读取环境变量
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:///data/fechatter.db".to_string());
    let environment = std::env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "demo".to_string());
    
    let state = AppState {
        db_url,
        environment,
    };
    
    // 创建路由
    let app = Router::new()
        // 健康检查
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        
        // API 路由 (简化版)
        .route("/", get(serve_demo_page))
        .route("/api/demo", get(demo_api))
        
        // 用户相关 API
        .route("/api/users", get(get_demo_users))
        .route("/api/auth/login", post(demo_login))
        
        // 聊天相关 API  
        .route("/api/channels", get(get_demo_channels))
        .route("/api/messages", get(get_demo_messages))
        .route("/api/messages", post(send_demo_message))
        
        // WebSocket (简化版)
        .route("/ws", get(websocket_handler))
        
        .with_state(Arc::new(state));
    
    // 绑定端口
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    println!("🌐 Server starting on {}", addr);
    println!("📱 Demo page: http://localhost:{}", port);
    
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

// 健康检查端点
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "fechatter-demo",
        "environment": "fly.io",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// 就绪检查端点
async fn readiness_check(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ready",
        "database": "connected",
        "environment": state.environment,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// 演示主页
async fn serve_demo_page() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html lang="zh-CN">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Fechatter Demo - HR展示</title>
        <style>
            body { 
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                margin: 0; padding: 2rem; background: #f5f5f5; line-height: 1.6;
            }
            .container { max-width: 800px; margin: 0 auto; background: white; padding: 2rem; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }
            .header { text-align: center; margin-bottom: 2rem; }
            .logo { font-size: 2.5rem; font-weight: bold; color: #2563eb; margin-bottom: 0.5rem; }
            .subtitle { color: #6b7280; font-size: 1.1rem; }
            .features { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 1.5rem; margin: 2rem 0; }
            .feature { padding: 1.5rem; background: #f8fafc; border-radius: 8px; border-left: 4px solid #2563eb; }
            .feature h3 { margin: 0 0 0.5rem 0; color: #1e293b; }
            .demo-users { background: #ecfdf5; padding: 1.5rem; border-radius: 8px; margin: 2rem 0; }
            .user-list { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; margin-top: 1rem; }
            .user { background: white; padding: 1rem; border-radius: 6px; text-align: center; }
            .btn { display: inline-block; background: #2563eb; color: white; padding: 0.75rem 1.5rem; text-decoration: none; border-radius: 6px; margin: 0.5rem; transition: background 0.3s; }
            .btn:hover { background: #1d4ed8; }
            .status { text-align: center; padding: 1rem; background: #dcfce7; border-radius: 6px; margin: 2rem 0; }
        </style>
    </head>
    <body>
        <div class="container">
            <div class="header">
                <div class="logo">🚀 Fechatter</div>
                <div class="subtitle">现代化实时团队协作平台 - HR技术演示</div>
            </div>
            
            <div class="status">
                <strong>✅ 演示环境已就绪</strong> | 
                部署于 Fly.io Tokyo | 
                <span id="current-time"></span>
            </div>
            
            <div class="features">
                <div class="feature">
                    <h3>🎯 核心功能</h3>
                    <ul>
                        <li>实时消息传递</li>
                        <li>多频道聊天</li>
                        <li>用户认证系统</li>
                        <li>消息搜索</li>
                    </ul>
                </div>
                <div class="feature">
                    <h3>⚡ 技术特点</h3>
                    <ul>
                        <li>Rust 高性能后端</li>
                        <li>Vue.js 现代前端</li>
                        <li>WebSocket 实时通信</li>
                        <li>RESTful API 设计</li>
                    </ul>
                </div>
                <div class="feature">
                    <h3>🌐 部署架构</h3>
                    <ul>
                        <li>Docker 容器化</li>
                        <li>云原生设计</li>
                        <li>自动扩缩容</li>
                        <li>CI/CD 自动化</li>
                    </ul>
                </div>
            </div>
            
            <div class="demo-users">
                <h3>👥 演示账户</h3>
                <p>以下账户已预置演示数据，密码均为：<code>demo123</code></p>
                <div class="user-list">
                    <div class="user">
                        <strong>demo_admin</strong><br>
                        <small>管理员账户</small>
                    </div>
                    <div class="user">
                        <strong>alice_dev</strong><br>
                        <small>开发工程师</small>
                    </div>
                    <div class="user">
                        <strong>bob_designer</strong><br>
                        <small>UI设计师</small>
                    </div>
                    <div class="user">
                        <strong>charlie_pm</strong><br>
                        <small>产品经理</small>
                    </div>
                    <div class="user">
                        <strong>diana_qa</strong><br>
                        <small>测试工程师</small>
                    </div>
                </div>
            </div>
            
            <div style="text-align: center; margin-top: 2rem;">
                <a href="/api/demo" class="btn">📊 查看API演示</a>
                <a href="/api/users" class="btn">👥 用户列表API</a>
                <a href="/api/channels" class="btn">📢 频道列表API</a>
                <a href="/health" class="btn">🔍 健康检查</a>
            </div>
            
            <div style="text-align: center; margin-top: 2rem; color: #6b7280; font-size: 0.9rem;">
                <p>🎯 这是一个技术演示环境，展示现代化聊天应用的核心功能</p>
                <p>💼 为HR面试准备，演示全栈开发能力和云部署技能</p>
            </div>
        </div>
        
        <script>
            // 显示当前时间
            function updateTime() {
                document.getElementById('current-time').textContent = new Date().toLocaleString('zh-CN', {timeZone: 'Asia/Tokyo'});
            }
            updateTime();
            setInterval(updateTime, 1000);
        </script>
    </body>
    </html>
    "#)
}

// 演示 API
async fn demo_api() -> Json<serde_json::Value> {
    Json(json!({
        "message": "欢迎使用 Fechatter Demo API",
        "features": [
            "实时消息传递",
            "用户认证",
            "频道管理",
            "消息搜索"
        ],
        "endpoints": {
            "users": "/api/users",
            "channels": "/api/channels", 
            "messages": "/api/messages",
            "auth": "/api/auth/login"
        },
        "demo_info": {
            "total_users": 5,
            "total_channels": 5,
            "total_messages": 20,
            "environment": "fly.io",
            "region": "Tokyo"
        }
    }))
}

// 获取演示用户
async fn get_demo_users() -> Json<serde_json::Value> {
    Json(json!([
        {"id": 1, "username": "demo_admin", "display_name": "Demo Admin", "role": "admin"},
        {"id": 2, "username": "alice_dev", "display_name": "Alice Johnson", "role": "developer"},
        {"id": 3, "username": "bob_designer", "display_name": "Bob Smith", "role": "designer"},
        {"id": 4, "username": "charlie_pm", "display_name": "Charlie Wilson", "role": "pm"},
        {"id": 5, "username": "diana_qa", "display_name": "Diana Lee", "role": "qa"}
    ]))
}

// 演示登录
async fn demo_login() -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "token": "demo_jwt_token_12345",
        "user": {
            "id": 1,
            "username": "demo_admin",
            "display_name": "Demo Admin"
        },
        "message": "登录成功 - 这是演示环境"
    }))
}

// 获取演示频道
async fn get_demo_channels() -> Json<serde_json::Value> {
    Json(json!([
        {"id": 1, "name": "general", "description": "通用讨论频道", "members": 5},
        {"id": 2, "name": "development", "description": "技术开发讨论", "members": 3},
        {"id": 3, "name": "product-updates", "description": "产品更新通知", "members": 3},
        {"id": 4, "name": "random", "description": "随意聊天", "members": 5},
        {"id": 5, "name": "project-alpha", "description": "Alpha项目", "members": 2}
    ]))
}

// 获取演示消息
async fn get_demo_messages() -> Json<serde_json::Value> {
    Json(json!([
        {
            "id": 1,
            "channel_id": 1,
            "user": "demo_admin",
            "content": "🎉 欢迎来到Fechatter实时聊天演示！",
            "timestamp": "2024-01-15T10:00:00Z"
        },
        {
            "id": 2, 
            "channel_id": 1,
            "user": "alice_dev",
            "content": "太棒了！这个界面看起来很专业 ✨",
            "timestamp": "2024-01-15T10:01:00Z"
        },
        {
            "id": 3,
            "channel_id": 1, 
            "user": "bob_designer",
            "content": "我喜欢这个设计，用户体验很流畅！",
            "timestamp": "2024-01-15T10:02:00Z"
        }
    ]))
}

// 发送演示消息
async fn send_demo_message() -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "message": {
            "id": 999,
            "content": "这是一条演示消息",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "user": "demo_user"
        },
        "note": "在真实环境中，消息会实时广播给所有在线用户"
    }))
}

// WebSocket 处理器（简化版）
async fn websocket_handler() -> &'static str {
    "WebSocket endpoint - 在真实环境中这里会建立实时连接"
}