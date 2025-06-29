# Multi-stage build for Fechatter services
# Stage 1: Build environment with all dependencies
FROM rust:1.83-alpine3.20 AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    protobuf-dev \
    git \
    curl \
    build-base \
    perl \
    make \
    cmake

# Install protobuf with Google Well-Known Types
RUN curl -L https://github.com/protocolbuffers/protobuf/releases/download/v25.1/protoc-25.1-linux-x86_64.zip -o protoc.zip && \
    unzip protoc.zip -d /usr/local && \
    rm protoc.zip

# Set up cargo environment for static builds
ENV RUSTFLAGS="-C target-feature=-crt-static" \
    OPENSSL_STATIC=1 \
    OPENSSL_LIB_DIR=/usr/lib \
    OPENSSL_INCLUDE_DIR=/usr/include \
    PKG_CONFIG_ALLOW_CROSS=1 \
    PROTOC=/usr/local/bin/protoc

# Create app directory
WORKDIR /usr/src/app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY fechatter_core/Cargo.toml ./fechatter_core/
COPY fechatter_server/Cargo.toml ./fechatter_server/
COPY fechatter_gateway/Cargo.toml ./fechatter_gateway/
COPY analytics_server/Cargo.toml ./analytics_server/
COPY bot_server/Cargo.toml ./bot_server/
COPY notify_server/Cargo.toml ./notify_server/
COPY ai_sdk/Cargo.toml ./ai_sdk/
COPY fechatter_protos/Cargo.toml ./fechatter_protos/
COPY swiftide-pgvector/Cargo.toml ./swiftide-pgvector/

# Create dummy source files to cache dependencies
RUN mkdir -p fechatter_core/src && echo "fn main() {}" > fechatter_core/src/lib.rs
RUN mkdir -p fechatter_server/src && echo "fn main() {}" > fechatter_server/src/main.rs
RUN mkdir -p fechatter_gateway/src && echo "fn main() {}" > fechatter_gateway/src/main.rs
RUN mkdir -p analytics_server/src && echo "fn main() {}" > analytics_server/src/main.rs
RUN mkdir -p bot_server/src && echo "fn main() {}" > bot_server/src/main.rs
RUN mkdir -p notify_server/src && echo "fn main() {}" > notify_server/src/main.rs
RUN mkdir -p ai_sdk/src && echo "fn main() {}" > ai_sdk/src/lib.rs
RUN mkdir -p fechatter_protos/src && echo "fn main() {}" > fechatter_protos/src/lib.rs
RUN mkdir -p swiftide-pgvector/src && echo "fn main() {}" > swiftide-pgvector/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release --workspace && rm -rf target/release/deps/fechatter*

# Copy actual source code
COPY . .

# Clean and rebuild with actual source
RUN cargo clean -p fechatter_server -p fechatter_gateway -p analytics_server -p bot_server -p notify_server -p fechatter_core -p fechatter_protos -p ai_sdk -p swiftide-pgvector
RUN cargo build --release --bin fechatter-server --bin fechatter_gateway --bin analytics-server --bin bot-server --bin notify-server

# Stage 2: Runtime environment
FROM alpine:3.20

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    openssl \
    libgcc \
    supervisor \
    curl

# Create app user
RUN adduser -D -s /bin/sh appuser

# Copy binaries from builder
COPY --from=builder /usr/src/app/target/release/fechatter-server /usr/local/bin/
COPY --from=builder /usr/src/app/target/release/fechatter_gateway /usr/local/bin/gateway
COPY --from=builder /usr/src/app/target/release/analytics-server /usr/local/bin/
COPY --from=builder /usr/src/app/target/release/bot-server /usr/local/bin/
COPY --from=builder /usr/src/app/target/release/notify-server /usr/local/bin/

# Copy configuration files
COPY fechatter_server/chat.yml /etc/fechatter/chat.yml
COPY fechatter_gateway/gateway.yaml /etc/fechatter/gateway.yaml
COPY analytics_server/analytics.yml /etc/fechatter/analytics.yml
COPY bot_server/bot.yml /etc/fechatter/bot.yml
COPY notify_server/notify.yml /etc/fechatter/notify.yml

# Copy migrations
COPY migrations /app/migrations

# Create directories for uploads and logs
RUN mkdir -p /app/uploads /app/logs && \
    chown -R appuser:appuser /app /etc/fechatter

# Create supervisor configuration
RUN echo "[supervisord]" > /etc/supervisor.conf && \
    echo "nodaemon=true" >> /etc/supervisor.conf && \
    echo "user=root" >> /etc/supervisor.conf && \
    echo "" >> /etc/supervisor.conf && \
    echo "[program:gateway]" >> /etc/supervisor.conf && \
    echo "command=/usr/local/bin/gateway" >> /etc/supervisor.conf && \
    echo "autostart=true" >> /etc/supervisor.conf && \
    echo "autorestart=true" >> /etc/supervisor.conf && \
    echo "user=appuser" >> /etc/supervisor.conf && \
    echo "environment=CONFIG_PATH=\"/etc/fechatter/gateway.yaml\"" >> /etc/supervisor.conf && \
    echo "" >> /etc/supervisor.conf && \
    echo "[program:fechatter]" >> /etc/supervisor.conf && \
    echo "command=/usr/local/bin/fechatter-server" >> /etc/supervisor.conf && \
    echo "autostart=true" >> /etc/supervisor.conf && \
    echo "autorestart=true" >> /etc/supervisor.conf && \
    echo "user=appuser" >> /etc/supervisor.conf && \
    echo "environment=CONFIG_PATH=\"/etc/fechatter/chat.yml\"" >> /etc/supervisor.conf && \
    echo "" >> /etc/supervisor.conf && \
    echo "[program:analytics]" >> /etc/supervisor.conf && \
    echo "command=/usr/local/bin/analytics-server" >> /etc/supervisor.conf && \
    echo "autostart=true" >> /etc/supervisor.conf && \
    echo "autorestart=true" >> /etc/supervisor.conf && \
    echo "user=appuser" >> /etc/supervisor.conf && \
    echo "environment=CONFIG_PATH=\"/etc/fechatter/analytics.yml\"" >> /etc/supervisor.conf && \
    echo "" >> /etc/supervisor.conf && \
    echo "[program:bot]" >> /etc/supervisor.conf && \
    echo "command=/usr/local/bin/bot-server" >> /etc/supervisor.conf && \
    echo "autostart=true" >> /etc/supervisor.conf && \
    echo "autorestart=true" >> /etc/supervisor.conf && \
    echo "user=appuser" >> /etc/supervisor.conf && \
    echo "environment=CONFIG_PATH=\"/etc/fechatter/bot.yml\"" >> /etc/supervisor.conf && \
    echo "" >> /etc/supervisor.conf && \
    echo "[program:notify]" >> /etc/supervisor.conf && \
    echo "command=/usr/local/bin/notify-server" >> /etc/supervisor.conf && \
    echo "autostart=true" >> /etc/supervisor.conf && \
    echo "autorestart=true" >> /etc/supervisor.conf && \
    echo "user=appuser" >> /etc/supervisor.conf && \
    echo "environment=CONFIG_PATH=\"/etc/fechatter/notify.yml\"" >> /etc/supervisor.conf

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Expose ports
EXPOSE 8080 3000 3001 3002 3003

# Run supervisor
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor.conf"]