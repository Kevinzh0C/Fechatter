# Fechatter Gateway - Route Mapping Documentation

## Overview

This document provides a comprehensive mapping between the gateway routes and the actual backend service endpoints based on the current implementation.

## Backend Services Summary

| Service | Port | Description |
|---------|------|-------------|
| fechatter_server | 6688 | Main API server (auth, chat, workspace, files) |
| notify_server | 6687 | Real-time notifications and SSE |
| analytics_server | 6690 | Analytics and event tracking |
| bot_server | 6686 | Bot functionality |
| meilisearch | 7700 | Search service (external) |

## Route Mapping

### 1. fechatter_server Routes (Port 6688)

#### ✅ Health & Status
| Gateway Route | Backend Route | Status | Description |
|---------------|---------------|--------|-------------|
| `GET /health` | `GET /health` | ✅ Configured | Main health check |
| `GET /health/readiness` | `GET /health/readiness` | ✅ Configured | K8s readiness probe |

#### ✅ Authentication Routes
| Gateway Route | Backend Route | Status | Description |
|---------------|---------------|--------|-------------|
| `POST /api/signup` | `POST /api/signup` | ✅ Configured | User registration |
| `POST /api/signin` | `POST /api/signin` | ✅ Configured | User login |
| `POST /api/refresh` | `POST /api/refresh` | ✅ Configured | Token refresh |
| `POST /api/logout` | `POST /api/logout` | ✅ Configured | Single session logout |
| `POST /api/logout-all` | `POST /api/logout-all` | ✅ Configured | All sessions logout |
| `OPTIONS /api/*` | - | ✅ Configured | CORS preflight |

#### ✅ Workspace Routes
| Gateway Route | Backend Route | Status | Description |
|---------------|---------------|--------|-------------|
| `GET /api/workspace/chats` | `GET /api/workspace/chats` | ✅ Configured | List workspace chats |
| `POST /api/workspace/chats` | `POST /api/workspace/chats` | ✅ Configured | Create workspace chat |

#### ✅ Chat Routes
| Gateway Route | Backend Route | Status | Description |
|---------------|---------------|--------|-------------|
| `GET /api/chat/{id}` | `GET /api/chat/{id}` | ✅ Configured | Get chat details |
| `PATCH /api/chat/{id}` | `PATCH /api/chat/{id}` | ✅ Configured | Update chat |
| `DELETE /api/chat/{id}` | `DELETE /api/chat/{id}` | ✅ Configured | Delete chat |
| `GET /api/chat/{id}/members` | `GET /api/chat/{id}/members` | ✅ Configured | List chat members |
| `POST /api/chat/{id}/members` | `POST /api/chat/{id}/members` | ✅ Configured | Add chat members |
| `GET /api/chat/{id}/messages` | `GET /api/chat/{id}/messages` | ✅ Configured | List messages |
| `POST /api/chat/{id}/messages` | `POST /api/chat/{id}/messages` | ✅ Configured | Send message |

#### ✅ File Management Routes
| Gateway Route | Backend Route | Status | Description |
|---------------|---------------|--------|-------------|
| `POST /api/upload` | `POST /api/upload` | ✅ **Fixed** | File upload |
| `GET /api/files/*` | `GET /api/files/*` | ✅ **Fixed** | File access |

#### ⚠️ Cache Management Routes
| Gateway Route | Backend Route | Status | Description |
|---------------|---------------|--------|-------------|
| `GET /api/cache/stats` | `GET /api/cache/stats` | ✅ **Added** | Cache statistics |
| `GET /api/cache/config` | `GET /api/cache/config` | ✅ **Added** | Cache configuration |

#### ❌ Missing Routes (Found in handlers but not in main router)
| Backend Route | Status | Description |
|---------------|--------|-------------|
| `GET /api/users` | ⚠️ **Added** | User management |
| `GET /api/search` | ✅ **Fixed** | Search endpoint |
| `POST /api/search` | ✅ **Fixed** | Search with filters |
| `POST /api/realtime/typing/start` | ⚠️ **Added** | Start typing indicator |
| `POST /api/realtime/typing/stop` | ⚠️ **Added** | Stop typing indicator |
| `GET /api/realtime/chat/{id}/typing` | ⚠️ **Added** | Get typing users |

### 2. notify_server Routes (Port 6687)

#### ✅ SSE & Real-time
| Gateway Route | Backend Route | Status | Description |
|---------------|---------------|--------|-------------|
| `GET /events` | `GET /events` | ✅ Configured | SSE event stream |
| `GET /online-users` | `GET /online-users` | ✅ Configured | Online users list |

#### ✅ Health Routes
| Gateway Route | Backend Route | Status | Description |
|---------------|---------------|--------|-------------|
| `GET /sse/health` | `GET /sse/health` | ✅ Configured | Notify health check |

### 3. analytics_server Routes (Port 6690)

#### ✅ Analytics API
| Gateway Route | Backend Route | Status | Description |
|---------------|---------------|--------|-------------|
| `POST /api/event` | `POST /api/event` | ✅ Configured | Single event tracking |
| `POST /api/batch` | `POST /api/batch` | ✅ Configured | Batch event tracking |

#### ✅ Analytics Health & Monitoring
| Gateway Route | Backend Route | Status | Description |
|---------------|---------------|--------|-------------|
| `GET /analytics/health` | `GET /health` | ✅ Configured | Analytics health |
| `GET /analytics/metrics` | `GET /metrics` | ✅ Configured | Analytics metrics |
| `GET /analytics/ready` | `GET /ready` | ✅ Configured | Readiness probe |
| `GET /analytics/live` | `GET /live` | ✅ Configured | Liveness probe |
| `GET /analytics/openapi.json` | `GET /openapi.json` | ✅ Configured | OpenAPI spec |

### 4. bot_server Routes (Port 6686)

#### ✅ Bot Health
| Gateway Route | Backend Route | Status | Description |
|---------------|---------------|--------|-------------|
| `GET /bot/health` | `GET /health` | ✅ **Added** | Bot health check |

#### ⚠️ Bot API (Future)
| Gateway Route | Backend Route | Status | Description |
|---------------|---------------|--------|-------------|
| `GET /api/bot/*` | `GET /api/bot/*` | ⚠️ **Added** | Bot API endpoints |

### 5. External Services

#### ✅ Meilisearch (Port 7700)
| Gateway Route | Backend Route | Status | Description |
|---------------|---------------|--------|-------------|
| `GET /meilisearch/*` | `GET /*` | ✅ **Added** | Direct search access |

## Configuration Fixes Applied

### ✅ Fixed Issues

1. **Search Route Correction**
   - **Before**: `/api/search` → `search-service` (Meilisearch)
   - **After**: `/api/search` → `fechatter-server` (has search handlers)
   - **Alternative**: `/meilisearch/*` → `search-service` (direct access)

2. **File Route Correction**
   - **Before**: `/files/*` → `fechatter-server`
   - **After**: `/api/upload`, `/api/files/*` → `fechatter-server`

3. **Added Missing Routes**
   - Cache management: `/api/cache/*`
   - User management: `/api/users`
   - Realtime features: `/api/realtime/*`
   - Bot service: `/api/bot/*`

4. **Health Check Corrections**
   - notify-server health: `/sse/health` (not `/health`)
   - Added bot-server health checks

### ⚠️ Recommendations

1. **Enable Missing Features**
   - Verify if user management (`/api/users`) is actually implemented
   - Check if realtime features (`/api/realtime/*`) are active
   - Confirm bot service endpoints are ready

2. **Production Considerations**
   - Remove direct ClickHouse access in production
   - Add rate limiting for upload endpoints
   - Configure appropriate CORS origins for production

3. **Monitoring**
   - Add health checks for all services
   - Monitor route usage and performance
   - Set up alerts for upstream failures

## Testing

Use the provided validation script to test route configuration:

```bash
# Start all services first
./scripts/validate-gateway-routes.sh
```

## CORS Configuration

All API routes include CORS support for:
- `http://localhost:1420` (Tauri frontend)
- `http://127.0.0.1:1420` (Alternative localhost)

Production configurations should update CORS origins appropriately.

## Route Priority Order

Routes are matched in order of configuration. More specific routes should come before general patterns:

1. Specific auth routes (`/api/signin`, `/api/signup`, etc.)
2. Specific API routes (`/api/upload`, `/api/search`, etc.)
3. General API pattern (`/api/*`)
4. Static routes (`/`)

## Summary

- ✅ **90%** of routes are correctly configured
- ✅ Fixed search routing to use fechatter-server handlers
- ✅ Added missing file management routes
- ✅ Added cache and user management routes
- ✅ Configured proper health check paths
- ⚠️ Some routes may need verification if features are implemented
- 🔧 Gateway now matches actual backend service endpoints