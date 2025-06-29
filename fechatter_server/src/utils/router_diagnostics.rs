use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use tracing::{info, warn, error};

/// Router diagnostic middleware - logs all requests and their routing results
pub async fn router_diagnostic_middleware(
    req: Request<Body>,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path();
    
    info!("[ROUTER_DIAG] Incoming request: {} {}", method, path);
    
    // Check if this is a chat route
    if path.contains("/api/chat/") && path.contains("/") {
        info!("[ROUTER_DIAG] CHAT ROUTE DETECTED: {}", path);
        
        // Extract chat ID for debugging
        if let Some(chat_id) = extract_chat_id_from_path(path) {
            info!("[ROUTER_DIAG] Chat ID extracted: {}", chat_id);
        } else {
            warn!("[ROUTER_DIAG] WARNING: Failed to extract chat ID from: {}", path);
        }
    }
    
    // Time the request
    let start = std::time::Instant::now();
    
    // Execute the next middleware/handler
    let response = next.run(req).await;
    
    let duration = start.elapsed();
    let status = response.status();
    
    // Log the result
    match status {
        StatusCode::OK => {
            info!("[ROUTER_DIAG] SUCCESS: {} {} -> {} ({:?})", 
                  method, path, status, duration);
        },
        StatusCode::NOT_FOUND => {
            error!("[ROUTER_DIAG] 🚫 NOT FOUND: {} {} -> {} ({:?})", 
                   method, path, status, duration);
            error!("[ROUTER_DIAG] 🚫 This indicates the route is NOT REGISTERED!");
        },
        StatusCode::UNAUTHORIZED => {
            warn!("[ROUTER_DIAG] 🔐 UNAUTHORIZED: {} {} -> {} ({:?})", 
                  method, path, status, duration);
        },
        StatusCode::FORBIDDEN => {
            warn!("[ROUTER_DIAG] 🛡️ FORBIDDEN: {} {} -> {} ({:?})", 
                  method, path, status, duration);
        },
        StatusCode::INTERNAL_SERVER_ERROR => {
            error!("[ROUTER_DIAG] 💥 SERVER ERROR: {} {} -> {} ({:?})", 
                   method, path, status, duration);
        },
        _ => {
            info!("[ROUTER_DIAG] RESPONSE: {} {} -> {} ({:?})", 
                  method, path, status, duration);
        }
    }
    
    response
}

/// Extract chat_id from URL path (copied from chat middleware)
fn extract_chat_id_from_path(path: &str) -> Option<i64> {
    if let Some(start) = path.find("/api/chat/") {
        let after_prefix = &path[start + "/api/chat/".len()..];
        if let Some(end) = after_prefix.find('/') {
            let chat_id_str = &after_prefix[..end];
            chat_id_str.parse().ok()
        } else {
            after_prefix.parse().ok()
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_chat_id_from_path() {
        assert_eq!(extract_chat_id_from_path("/api/chat/123"), Some(123));
        assert_eq!(extract_chat_id_from_path("/api/chat/456/messages"), Some(456));
        assert_eq!(extract_chat_id_from_path("/api/users/123"), None);
        assert_eq!(extract_chat_id_from_path("/api/chat/invalid"), None);
        assert_eq!(extract_chat_id_from_path("/api/chat/"), None);
    }
}