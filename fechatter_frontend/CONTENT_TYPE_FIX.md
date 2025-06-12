# Content-Type 415 Error Fix

## 🐛 Problem: 415 Unsupported Media Type

### Error Details
- **Error**: `POST http://127.0.0.1:8080/api/chat/1/messages 415 (Unsupported Media Type)`
- **Expected**: `Content-Type: application/json`
- **Actual**: `Content-Type: multipart/form-data`

### 🔍 Root Cause Analysis

#### Call Chain
```
User sends message
    ↓
chatStore.sendMessage() with FormData
    ↓
api.post() with Content-Type: multipart/form-data
    ↓
Backend expects JSON(request): Json<SendMessageRequest>
    ↓
415 Error
```

### 🔧 Solution

The backend message endpoint only accepts JSON format. File uploads should be handled separately:

1. **For text-only messages**: Send as JSON
2. **For messages with files**: 
   - First upload files to `/files/single` endpoint
   - Get file URLs from upload response
   - Send message with file URLs in JSON format

#### Code Fix Required in chat.js
```javascript
// Remove FormData usage
const payload = {
  content,
  files: [], // Array of file URLs (not File objects)
  idempotency_key: null,
  reply_to: null,
  mentions: null
};

// Send as JSON (default Content-Type)
const response = await api.post(`/chat/${chatId}/messages`, payload);
```

### 📋 Correct Workflow
1. **Upload files** (if any): `POST /files/single` with multipart/form-data
2. **Send message**: `POST /chat/{chatId}/messages` with application/json

### 🚀 Implementation Notes
- The backend `SendMessageRequest` expects `files` as `Vec<String>` (URLs)
- File upload returns URLs that should be included in the message
- Separate file upload allows for progress tracking and better error handling 