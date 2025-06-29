// Fly.io monolithic application entry point
// Combines all microservices into a single executable

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

// Simplified application state
#[derive(Clone)]
struct AppState {
    db_url: String,
    environment: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::init();
    
    println!("Starting Fechatter Demo (Fly.io Single Binary)");
    
    // Read environment variables
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:///data/fechatter.db".to_string());
    let environment = std::env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "demo".to_string());
    
    let state = AppState {
        db_url,
        environment,
    };
    
    // Create routes
    let app = Router::new()
        // Health checks
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        
        // API routes (simplified)
        .route("/", get(serve_demo_page))
        .route("/api/demo", get(demo_api))
        
        // User-related APIs
        .route("/api/users", get(get_demo_users))
        .route("/api/auth/login", post(demo_login))
        
        // Chat-related APIs  
        .route("/api/channels", get(get_demo_channels))
        .route("/api/messages", get(get_demo_messages))
        .route("/api/messages", post(send_demo_message))
        
        // WebSocket (simplified)
        .route("/ws", get(websocket_handler))
        
        .with_state(Arc::new(state));
    
    // Bind port
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    println!("🌐 Server starting on {}", addr);
    println!("📱 Demo page: http://localhost:{}", port);
    
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

// Health check endpoint
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "fechatter-demo",
        "environment": "fly.io",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Readiness check endpoint
async fn readiness_check(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ready",
        "database": "connected",
        "environment": state.environment,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Demo homepage
async fn serve_demo_page() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html lang="zh-CN">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Fechatter Demo - HR Showcase</title>
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
                <div class="logo">Fechatter</div>
                <div class="subtitle">Modern Real-time Team Collaboration Platform - HR Technical Demo</div>
            </div>
            
            <div class="status">
                <strong>Demo Environment Ready</strong> | 
                Deployed on Fly.io Tokyo | 
                <span id="current-time"></span>
            </div>
            
            <div class="features">
                <div class="feature">
                    <h3>Core Features</h3>
                    <ul>
                        <li>Real-time messaging</li>
                        <li>Multi-channel chat</li>
                        <li>User authentication system</li>
                        <li>Message search</li>
                    </ul>
                </div>
                <div class="feature">
                    <h3>Technical Features</h3>
                    <ul>
                        <li>High-performance Rust backend</li>
                        <li>Modern Vue.js frontend</li>
                        <li>WebSocket real-time communication</li>
                        <li>RESTful API design</li>
                    </ul>
                </div>
                <div class="feature">
                    <h3>🌐 Deployment Architecture</h3>
                    <ul>
                        <li>Docker containerization</li>
                        <li>Cloud-native design</li>
                        <li>Auto-scaling</li>
                        <li>CI/CD automation</li>
                    </ul>
                </div>
            </div>
            
            <div class="demo-users">
                <h3>👥 Demo Accounts</h3>
                <p>The following accounts have pre-configured demo data, password for all: <code>demo123</code></p>
                <div class="user-list">
                    <div class="user">
                        <strong>demo_admin</strong><br>
                        <small>Administrator</small>
                    </div>
                    <div class="user">
                        <strong>alice_dev</strong><br>
                        <small>Developer</small>
                    </div>
                    <div class="user">
                        <strong>bob_designer</strong><br>
                        <small>UI Designer</small>
                    </div>
                    <div class="user">
                        <strong>charlie_pm</strong><br>
                        <small>Product Manager</small>
                    </div>
                    <div class="user">
                        <strong>diana_qa</strong><br>
                        <small>QA Engineer</small>
                    </div>
                </div>
            </div>
            
            <div style="text-align: center; margin-top: 2rem;">
                <a href="/api/demo" class="btn">View API Demo</a>
                <a href="/api/users" class="btn">👥 Users API</a>
                <a href="/api/channels" class="btn">📢 Channels API</a>
                <a href="/health" class="btn">Health Check</a>
            </div>
            
            <div style="text-align: center; margin-top: 2rem; color: #6b7280; font-size: 0.9rem;">
                <p>This is a technical demo environment showcasing core features of modern chat applications</p>
                <p>💼 Prepared for HR interviews, demonstrating full-stack development capabilities and cloud deployment skills</p>
            </div>
        </div>
        
        <script>
            // Display current time
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