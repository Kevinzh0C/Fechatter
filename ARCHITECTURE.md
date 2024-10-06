# Fechatter Architecture

## System Overview

```
Vue.js → API Gateway → [Server|Notify|Bot|Analytics] → [PostgreSQL|Redis|NATS|Meilisearch|ClickHouse]
          (:8080)         (:6688-6690)                   (Data Layer)
                                                                                      ↑
                                                                               Superset (:8088)
```

## Core Services

| Service | Port | Purpose |
|---------|------|---------|
| API Gateway (Pingora) | 8080 | Load balancing, auth, rate limiting |
| Fechatter Server | 6688 | Core API (users, chats, messages) |
| Notify Server | 6687 | Real-time SSE/WebSocket |
| Bot Server | 6686 | AI assistant (ChatGPT) |
| Analytics Server | 6690 | Metrics and monitoring |
| Superset | 8088 | Business intelligence dashboard |

## Tech Stack

**Backend:**
- Rust (Axum, Tokio)
- Pingora (API Gateway)
- PostgreSQL (primary data)
- Redis (cache/sessions)
- NATS JetStream (message queue)
- Meilisearch (full-text search)

**Frontend:**
- Vue 3 + TypeScript
- Vite + Tailwind CSS
- Server-Sent Events

**Analytics:**
- ClickHouse (time-series database)
- Apache Superset (BI dashboard)

## Data Flow

### Message Send
```
Client → Gateway → Server → PostgreSQL (persist)
                         → NATS (publish)
                         → Notify → SSE → Clients
```

### Search
```
Client → Gateway → Server → Meilisearch → Results
                         → Redis (cache)
```

## Deployment

- **Development**: Docker Compose
- **Production**: Kubernetes + Helm
- **Monitoring**: Prometheus + Grafana
- **Logging**: JSON structured logs

## Security

- JWT authentication
- RBAC authorization
- TLS everywhere
- Rate limiting
- Input validation

## Scaling

- Stateless services (horizontal scaling)
- Database read replicas
- Redis cluster
- CDN for static assets

## Key Decisions

1. **Rust**: Performance and memory safety
2. **Microservices**: Independent scaling and deployment
3. **NATS**: Lightweight message queue
4. **PostgreSQL**: ACID compliance and JSON support

See [fechatter_server/docs/ARCHITECTURE.md](./fechatter_server/docs/ARCHITECTURE.md) for detailed implementation.