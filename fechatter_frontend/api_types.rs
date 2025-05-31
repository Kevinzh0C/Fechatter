// Fechatter API Types
// 自动生成，请勿手动修改
// Generated at: 2025-05-27T23:34:16.271Z

use serde::{Deserialize, Serialize};

// AUTH Types
#[derive(Debug, Serialize, Deserialize)]
pub struct SigninRequest {
    
    pub email: String,
    
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SigninResponse {
    
    pub access_token: String,
    
    pub refresh_token: String,
    
    pub expires_in: i64,
    
    pub user: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupRequest {
    
    pub fullname: String,
    
    pub email: String,
    
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupResponse {
    
    pub access_token: String,
    
    pub refresh_token: String,
    
    pub expires_in: i64,
    
    pub user: serde_json::Value,
}


// CHAT Types
pub type ChatGETResponse = Vec<serde_json::Value>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatPOSTRequest {
    
    pub name: String,
    
    pub chat_type: String,
    
    pub is_public: bool,
    
    pub workspace_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatPOSTResponse {
    
    pub id: i64,
    
    pub name: String,
    
    pub chat_type: String,
    
    pub is_public: bool,
    
    pub created_by: i64,
    
    pub workspace_id: i64,
    
    pub created_at: String,
}

pub type ChatIdMessagesGETResponse = Vec<serde_json::Value>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatIdMessagesPOSTRequest {
    
    pub content: String,
    
    pub message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatIdMessagesPOSTResponse {
    
    pub id: i64,
    
    pub content: String,
    
    pub message_type: String,
    
    pub chat_id: i64,
    
    pub sender_id: i64,
    
    pub created_at: String,
}


