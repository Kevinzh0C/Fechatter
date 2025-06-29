# Fechatter Architecture

## 🏗️ System Architecture Overview

Fechatter follows a modern microservices architecture with clear separation of concerns across multiple layers:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            Client Layer                                 │
│     ┌─────────────────────┐        ┌──────────────────────┐             │
│     │  fechatter_frontend │        │  Client Applications │             │
│     │  Vue 3 + TypeScript │        │   (Web/Mobile/API)   │             │
│     │       :3000         │        └──────────────────────┘             │
│     └─────────────────────┘                                             │
└─────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                            Gateway Layer                                │
│                    ┌──────────────────────────┐                         │
│                    │   fechatter_gateway      │                         │
│                    │    Pingora Proxy         │                         │
│                    │        :8080             │                         │
│                    └──────────────────────────┘                         │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
       ┌────────────────────────────┼──────────────────────────────┐
       │                            │                              │
       ▼                            ▼                              ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                              Core Services                              │
│      ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐      │
│      │ fechatter_server│  │  notify_server  │  │   bot_server    │      │
│      │  Axum + SQLx    │  │  Tokio + SSE    │  │  OpenAI SDK     │      │
│      │     :6688       │  │     :6687       │  │     :6686       │      │
│      └─────────────────┘  └─────────────────┘  └─────────────────┘      │
│                           ┌─────────────────┐                           │
│                           │ analytics_server│                           │
│                           │   ClickHouse    │                           │
│                           │     :6690       │                           │
│                           └─────────────────┘                           │
└─────────────────────────────────────────────────────────────────────────┘
           │                      │                    │
           ▼                      ▼                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                              Data Layer                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐   │
│  │  PostgreSQL  │  │    Redis     │  │ S3 Compatible│  │ClickHouse  │   │
│  │Primary DB    │  │Cache/Sessions│  │File Storage  │  │ Analytics  │   │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘   │
│  ┌──────────────┐  ┌──────────────┐                                     │
│  │ Meilisearch  │  │NATS JetStream│                                     │
│  │ Full-text    │  │Message Queue │                                     │
│  │   Search     │  │              │                                     │
│  └──────────────┘  └──────────────┘                                     │
└─────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           External Services                             │
│       ┌──────────────────────┐        ┌──────────────────────┐          │
│       │   Apache Superset    │        │     OpenAI API       │          │
│       │       :8088          │        │   (External Service) │          │
│       └──────────────────────┘        └──────────────────────┘          │
└─────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                       Shared Infrastructure                             │
│     ┌────────────────┐  ┌────────────────┐  ┌────────────────┐          │
│     │ fechatter_core │  │fechatter_protos│  │    ai_sdk      │          │
│     │ Common Types   │  │Protocol Buffers│  │AI Integrations │          │
│     └────────────────┘  └────────────────┘  └────────────────┘          │
└─────────────────────────────────────────────────────────────────────────┘
```

## 📋 Service Details

### Core Services

| Service                    | Port | Technology | Purpose                                                | Dependencies                             |
| -------------------------- | ---- | ---------- | ------------------------------------------------------ | ---------------------------------------- |
| **API Gateway**      | 8080 | Pingora    | Load balancing, routing, authentication, rate limiting | -                                        |
| **Fechatter Server** | 6688 | Axum, SQLx | Core API (users, chats, messages, workspaces)          | PostgreSQL, Redis, Meilisearch, NATS, S3 |
| **Notify Server**    | 6687 | Tokio, SSE | Real-time notifications and message delivery           | Redis, NATS                              |
| **Bot Server**       | 6686 | OpenAI SDK | AI-powered chat assistant                              | Redis, OpenAI API                        |
| **Analytics Server** | 6690 | ClickHouse | Event tracking, metrics collection                     | ClickHouse, NATS                         |

### Data Stores

| Component                | Purpose                                      | Technology                             |
| ------------------------ | -------------------------------------------- | -------------------------------------- |
| **PostgreSQL**     | Primary database for persistent data         | Relational database with JSONB support |
| **Redis**          | Session management, caching, real-time state | In-memory key-value store              |
| **Meilisearch**    | Full-text search for messages and users      | Rust-based search engine               |
| **ClickHouse**     | Time-series data for analytics               | Column-oriented OLAP database          |
| **NATS JetStream** | Message queue for event-driven communication | Persistent message streaming           |
| **S3 Compatible**  | File storage for uploads and attachments     | Object storage (MinIO/AWS S3)          |

### External Services

| Service                   | Purpose                                      | Integration            |
| ------------------------- | -------------------------------------------- | ---------------------- |
| **Apache Superset** | Business intelligence and data visualization | Connects to ClickHouse |
| **OpenAI API**      | Powers the AI chat assistant                 | Used by Bot Server     |

## 🔄 Data Flow Patterns

### 1. Message Send Flow

```
Client → Gateway → Fechatter Server → PostgreSQL (persist)
                                   → NATS (publish event)
                                   → Redis (cache)
                               
NATS → Notify Server → SSE/WebSocket → All connected clients
     → Analytics Server → ClickHouse (metrics)
```

### 2. Real-time Notification Flow

```
Event Source → NATS JetStream → Notify Server → Redis (check connections)
                                              → SSE push to clients
```

### 3. Search Flow

```
Client → Gateway → Fechatter Server → Meilisearch (query)
                                   → Redis (cache results)
                                   → Response
```

### 4. AI Bot Interaction

```
Client → Gateway → Fechatter Server → NATS → Bot Server → OpenAI API
                                           → Process response
                                           → NATS → Notify → Client
```

### 5. Analytics Pipeline

```
All Services → NATS (events) → Analytics Server → ClickHouse
                                                → Superset (visualization)
```

## 🏛️ Architecture Principles

### 1. **Microservices Design**

- Each service has a single responsibility
- Services communicate via well-defined APIs
- Independent deployment and scaling

### 2. **Event-Driven Architecture**

- NATS JetStream for asynchronous communication
- Event sourcing for audit trails
- Eventual consistency where appropriate

### 3. **Layered Architecture**

- Clear separation between layers
- Dependency injection
- Interface-based design

### 4. **Data Consistency**

- PostgreSQL for ACID transactions
- Redis for eventual consistency
- NATS for reliable message delivery

## 🔐 Security Architecture

### Authentication & Authorization

- **JWT tokens** for stateless authentication
- **RBAC** (Role-Based Access Control) for permissions
- **API Gateway** handles authentication before routing

### Data Security

- **TLS/SSL** for all network communication
- **Encryption at rest** for sensitive data
- **Input validation** at all entry points

### Network Security

```
Internet → Firewall → Load Balancer → API Gateway → Internal Network
                                                   → (Microservices)
```

## 📈 Scaling Strategy

### Horizontal Scaling

- **Stateless services** can be replicated
- **Database read replicas** for read-heavy workloads
- **Redis cluster** for distributed caching

### Vertical Scaling

- **Resource limits** configured per service
- **Auto-scaling** based on CPU/memory metrics
- **Database connection pooling**

### Performance Optimization

- **Caching layers** (Redis, CDN)
- **Message batching** in NATS
- **Lazy loading** and pagination
- **Database query optimization**

## 🚀 Deployment Architecture

### Development Environment

```yaml
# Docker Compose setup
services:
  gateway:     1 instance
  server:      1 instance
  notify:      1 instance
  bot:         1 instance
  analytics:   1 instance
  postgresql:  1 instance
  redis:       1 instance
  meilisearch: 1 instance
  clickhouse:  1 instance
  nats:        1 instance
```

### Production Environment

```yaml
# Kubernetes deployment
namespaces:
  - fechatter-prod
  - fechatter-staging
  
deployments:
  gateway:     3 replicas (HPA: 3-10)
  server:      5 replicas (HPA: 5-20)
  notify:      3 replicas (HPA: 3-10)
  bot:         2 replicas (HPA: 2-5)
  analytics:   2 replicas (HPA: 2-5)
  
statefulsets:
  postgresql:  1 primary + 2 read replicas
  redis:       3-node cluster
  clickhouse:  3-node cluster
  nats:        3-node cluster
```

## 🔍 Monitoring & Observability

### Metrics Collection

```
Services → Prometheus metrics → Prometheus Server → Grafana
         → Custom metrics → ClickHouse → Superset
```

### Logging Pipeline

```
Services → JSON logs → Fluentd → Elasticsearch → Kibana
```

### Distributed Tracing

```
Services → OpenTelemetry → Jaeger → Trace Analysis
```

## 🛠️ Technology Decisions

### Why Rust?

- **Performance**: Near C++ performance with memory safety
- **Concurrency**: Fearless concurrency with async/await
- **Reliability**: No null pointer exceptions or data races

### Why Microservices?

- **Scalability**: Scale services independently
- **Maintainability**: Smaller, focused codebases
- **Resilience**: Failure isolation

### Why NATS?

- **Lightweight**: Minimal overhead
- **Fast**: High throughput, low latency
- **Simple**: Easy to operate and maintain

### Why PostgreSQL + Redis?

- **PostgreSQL**: ACID compliance, JSON support, proven reliability
- **Redis**: Sub-millisecond latency, perfect for caching and sessions

## 📚 Related Documentation

- [API Reference](./fechatter_server/docs/API_REFERENCE.md) - REST API documentation
- [Development Guide](./fechatter_server/docs/DEVELOPMENT_GUIDE.md) - Setup and development
- [Deployment Guide](./fechatter_server/docs/DEPLOYMENT_GUIDE.md) - Production deployment
- [Performance Guide](./fechatter_server/docs/PERFORMANCE_GUIDE.md) - Optimization strategies
