# Fechatter Server API Reference

## 📋 Table of Contents

1. [Overview](#overview)
2. [Authentication](#authentication)
3. [User Management](#user-management)
4. [Workspace Management](#workspace-management)
5. [Chat Management](#chat-management)
6. [Message Operations](#message-operations)
7. [Real-Time Communication](#real-time-communication)
8. [Search API](#search-api)
9. [Health & Monitoring](#health--monitoring)
10. [Error Handling](#error-handling)

## 🎯 Overview

### Base URL
```
https://api.fechatter.com
```

### Content Types
- Request: `application/json`
- Response: `application/json`

### Rate Limiting
- Default: 1000 requests per hour per user
- Search endpoints: 100 requests per minute
- WebSocket connections: 10 per user

## 🔐 Authentication

### Login
```http
POST /api/auth/login
```

**Request:**
```json
{
  "email": "user@example.com",
  "password": "securepassword"
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "user": {
    "id": 1,
    "email": "user@example.com",
    "username": "johndoe",
    "avatar_url": "https://..."
  },
  "expires_at": "2024-12-25T00:00:00Z"
}
```

### Register
```http
POST /api/auth/register
```

**Request:**
```json
{
  "email": "newuser@example.com",
  "username": "newuser",
  "password": "securepassword"
}
```

**Response:** Same as login

### Refresh Token
```http
POST /api/auth/refresh
```

**Headers:**
```
Authorization: Bearer <expired_token>
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_at": "2024-12-25T00:00:00Z"
}
```

### Change Password
```http
PUT /api/auth/change-password
```

**Headers:**
```
Authorization: Bearer <token>
```

**Request:**
```json
{
  "current_password": "oldpassword",
  "new_password": "newpassword"
}
```

## 👤 User Management

### Get Current User
```http
GET /api/users/me
```

**Response:**
```json
{
  "id": 1,
  "email": "user@example.com",
  "username": "johndoe",
  "avatar_url": "https://...",
  "created_at": "2024-01-01T00:00:00Z",
  "preferences": {
    "theme": "dark",
    "language": "en",
    "notifications_enabled": true
  }
}
```

### Update Profile
```http
PUT /api/users/me
```

**Request:**
```json
{
  "username": "newusername",
  "avatar_url": "https://...",
  "preferences": {
    "theme": "light",
    "language": "zh"
  }
}
```

### List Users (Workspace Context)
```http
GET /api/users?workspace_id=1&page=1&limit=20
```

**Response:**
```json
{
  "users": [
    {
      "id": 1,
      "username": "johndoe",
      "avatar_url": "https://...",
      "status": "online",
      "role": "member"
    }
  ],
  "total": 100,
  "page": 1,
  "pages": 5
}
```

## 🏢 Workspace Management

### List Workspaces
```http
GET /api/workspaces
```

**Response:**
```json
{
  "workspaces": [
    {
      "id": 1,
      "name": "Acme Corp",
      "description": "Company workspace",
      "logo_url": "https://...",
      "member_count": 150,
      "role": "admin",
      "created_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

### Create Workspace
```http
POST /api/workspaces
```

**Request:**
```json
{
  "name": "New Workspace",
  "description": "A new workspace for our team"
}
```

### Get Workspace Details
```http
GET /api/workspaces/{workspace_id}
```

**Response:**
```json
{
  "id": 1,
  "name": "Acme Corp",
  "description": "Company workspace",
  "settings": {
    "allow_guest_access": false,
    "require_2fa": true,
    "retention_days": 90
  },
  "stats": {
    "member_count": 150,
    "channel_count": 25,
    "message_count": 50000
  }
}
```

### Invite User to Workspace
```http
POST /api/workspaces/{workspace_id}/invites
```

**Request:**
```json
{
  "email": "newuser@example.com",
  "role": "member",
  "expires_in_days": 7
}
```

**Response:**
```json
{
  "invite_code": "ABC123XYZ",
  "invite_url": "https://fechatter.com/invite/ABC123XYZ",
  "expires_at": "2024-12-31T00:00:00Z"
}
```

## 💬 Chat Management

### List Chats
```http
GET /api/chats?workspace_id=1&page=1&limit=20
```

**Response:**
```json
{
  "chats": [
    {
      "id": 1,
      "name": "general",
      "type": "channel",
      "description": "General discussion",
      "member_count": 50,
      "unread_count": 5,
      "last_message": {
        "id": 100,
        "content": "Hello everyone!",
        "sender": {
          "id": 2,
          "username": "alice"
        },
        "created_at": "2024-12-19T10:00:00Z"
      }
    }
  ],
  "total": 25,
  "page": 1
}
```

### Create Chat
```http
POST /api/chats
```

**Request:**
```json
{
  "workspace_id": 1,
  "name": "new-channel",
  "type": "channel",
  "description": "A new channel for discussions",
  "is_private": false
}
```

### Create Direct Message
```http
POST /api/chats/direct
```

**Request:**
```json
{
  "workspace_id": 1,
  "user_ids": [2, 3]
}
```

### Join/Leave Chat
```http
POST /api/chats/{chat_id}/join
POST /api/chats/{chat_id}/leave
```

### Get Chat Members
```http
GET /api/chats/{chat_id}/members
```

**Response:**
```json
{
  "members": [
    {
      "user_id": 1,
      "username": "johndoe",
      "role": "owner",
      "joined_at": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 50
}
```

## 📨 Message Operations

### Send Message
```http
POST /api/messages
```

**Request:**
```json
{
  "chat_id": 1,
  "content": "Hello everyone!",
  "idempotency_key": "unique-key-123"
}
```

**Response:**
```json
{
  "id": 101,
  "chat_id": 1,
  "sender_id": 1,
  "content": "Hello everyone!",
  "created_at": "2024-12-19T10:00:00Z",
  "updated_at": "2024-12-19T10:00:00Z"
}
```

### List Messages
```http
GET /api/messages?chat_id=1&limit=50&before_id=100
```

**Response:**
```json
{
  "messages": [
    {
      "id": 99,
      "chat_id": 1,
      "sender": {
        "id": 2,
        "username": "alice",
        "avatar_url": "https://..."
      },
      "content": "Previous message",
      "created_at": "2024-12-19T09:59:00Z",
      "reactions": [
        {
          "emoji": "👍",
          "count": 3,
          "users": [1, 3, 4]
        }
      ]
    }
  ],
  "has_more": true
}
```

### Update Message
```http
PUT /api/messages/{message_id}
```

**Request:**
```json
{
  "content": "Updated message content"
}
```

### Delete Message
```http
DELETE /api/messages/{message_id}
```

### Add Reaction
```http
POST /api/messages/{message_id}/reactions
```

**Request:**
```json
{
  "emoji": "👍"
}
```

## 🔄 Real-Time Communication

### WebSocket Connection
```
wss://api.fechatter.com/ws
```

**Authentication:**
```json
{
  "type": "auth",
  "token": "Bearer <token>"
}
```

### Event Types

#### Message Events
```json
{
  "type": "message.created",
  "data": {
    "message": {
      "id": 102,
      "chat_id": 1,
      "sender_id": 2,
      "content": "New message!",
      "created_at": "2024-12-19T10:01:00Z"
    }
  }
}
```

#### Typing Indicators
```json
{
  "type": "typing.start",
  "data": {
    "chat_id": 1,
    "user_id": 2,
    "username": "alice"
  }
}
```

#### Presence Updates
```json
{
  "type": "presence.update",
  "data": {
    "user_id": 2,
    "status": "online",
    "last_seen": "2024-12-19T10:00:00Z"
  }
}
```

### Server-Sent Events (SSE)
```http
GET /api/events
```

**Headers:**
```
Authorization: Bearer <token>
Accept: text/event-stream
```

**Event Format:**
```
event: message.created
data: {"message":{"id":102,"content":"New message!"}}

event: ping
data: {"timestamp":"2024-12-19T10:00:00Z"}
```

## 🔍 Search API

### Search Messages
```http
POST /api/search/messages
```

**Request:**
```json
{
  "query": "important meeting",
  "workspace_id": 1,
  "filters": {
    "chat_ids": [1, 2, 3],
    "sender_ids": [1, 2],
    "date_from": "2024-01-01",
    "date_to": "2024-12-31"
  },
  "page": 1,
  "limit": 20
}
```

**Response:**
```json
{
  "results": [
    {
      "message_id": 100,
      "chat_id": 1,
      "chat_name": "general",
      "content": "Let's discuss the <mark>important meeting</mark> agenda",
      "sender": {
        "id": 1,
        "username": "johndoe"
      },
      "created_at": "2024-12-19T09:00:00Z",
      "score": 0.95
    }
  ],
  "total": 5,
  "page": 1,
  "processing_time_ms": 15
}
```

### Search Users
```http
GET /api/search/users?q=john&workspace_id=1
```

### Search Chats
```http
GET /api/search/chats?q=general&workspace_id=1
```

## 🏥 Health & Monitoring

### Health Check
```http
GET /api/health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 86400
}
```

### Production Health
```http
GET /admin/production/health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": 1734567890,
  "services": {
    "database": {
      "status": "healthy",
      "latency_ms": 2
    },
    "redis": {
      "status": "healthy",
      "latency_ms": 1
    },
    "search": {
      "status": "healthy",
      "latency_ms": 5
    }
  },
  "metrics": {
    "requests_per_second": 125.5,
    "active_connections": 850,
    "cpu_usage_percent": 42.5,
    "memory_usage_mb": 1024
  }
}
```

### Service Metrics
```http
GET /admin/production/metrics/{service_name}
```

## ❌ Error Handling

### Error Response Format
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input data",
    "details": {
      "field": "email",
      "reason": "Invalid email format"
    }
  },
  "request_id": "req_123456789"
}
```

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Missing or invalid authentication |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `VALIDATION_ERROR` | 422 | Invalid input data |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |

### Rate Limit Headers
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1734567890
```

## 🔧 Advanced Features

### Pagination
All list endpoints support pagination:
```
?page=1&limit=20
```

### Sorting
```
?sort=created_at&order=desc
```

### Field Selection
```
?fields=id,username,email
```

### Batch Operations
```http
POST /api/batch
```

**Request:**
```json
{
  "operations": [
    {
      "method": "POST",
      "path": "/api/messages",
      "body": {"chat_id": 1, "content": "Message 1"}
    },
    {
      "method": "POST",
      "path": "/api/messages",
      "body": {"chat_id": 1, "content": "Message 2"}
    }
  ]
}
```

---

**Version**: 1.0.0  
**Last Updated**: December 2024  
**Status**: Production Ready ✅ 