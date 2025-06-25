# API Endpoints Fix

## 🐛 Problem: Message Send API 404 Error

### Error Details
- **Error**: `POST http://127.0.0.1:8080/api/messages 404 (Not Found)`
- **Location**: `chat.js:1293`
- **Error Code**: `ERR_BAD_REQUEST`

### 🔍 Root Cause Analysis

#### Call Chain (DAG - Directed Acyclic Graph)
```
User sends message
    ↓
Chat.vue component
    ↓
chatStore.sendMessage() [chat.js:1233]
    ↓
api.post('/messages', ...) [chat.js:1293] ❌
    ↓
Gateway (http://127.0.0.1:8080/api/messages)
    ↓
404 Not Found
```

#### Backend Route Investigation
Searched backend routes and found the correct endpoint in `fechatter_server/src/lib.rs:399`:
```rust
.route("/chat/{id}/messages", 
  get(list_messages_handler)
  .post(send_message_handler)
)
```

### 🔧 Solution

#### API Endpoint Mismatch
- ❌ **Frontend was calling**: `POST /messages`
- ✅ **Correct endpoint**: `POST /chat/{chatId}/messages`

#### Code Fix
Changed line 1293 in `chat.js`:
```javascript
// Before
const response = await api.post('/messages', formData, {

// After  
const response = await api.post(`/chat/${chatId}/messages`, formData, {
```

### 📋 API Endpoints Reference

#### Message Operations
- **Send Message**: `POST /chat/{chatId}/messages`
- **Get Messages**: `GET /chat/{chatId}/messages`
- **Search Messages**: `POST /chat/{chatId}/messages/search`
- **Mark as Read**: `POST /chat/{chatId}/messages/{messageId}/read`

#### Chat Operations
- **Create Chat**: `POST /workspace/chats`
- **Get Chat**: `GET /chat/{chatId}`
- **Update Chat**: `PATCH /chat/{chatId}`
- **Delete Chat**: `DELETE /chat/{chatId}`
- **List Chats**: `GET /workspace/chats`

### 🚀 Prevention Guidelines
1. Always check backend route definitions when encountering 404 errors
2. Use RESTful conventions: resources should be nested under their parent
3. Document API endpoints in a central location
4. Consider using OpenAPI/Swagger for API documentation 