# Installation Guide

This guide provides detailed instructions for installing and setting up Fechatter.

## Prerequisites

- Docker 20.10 or higher
- Docker Compose 2.0 or higher
- 4GB RAM minimum
- Git

## Installation Steps

### 1. Clone the Repository

```bash
git clone https://github.com/Kevinzh0C/fechatter.git
cd fechatter
```

### 2. Configure Environment

Copy the example environment file:

```bash
cp .env.example .env
```

Edit `.env` with your specific configuration.

### 3. Start Services

```bash
docker-compose up -d
```

### 4. Verify Installation

Check that all services are running:

```bash
docker-compose ps
```

You should see all services in "Up" state.

### 5. Access the Application

Open your browser and navigate to:
- Frontend: http://localhost:8080
- API Gateway: http://localhost:8080/api

## Troubleshooting

### Port Conflicts

If port 8080 is already in use, you can change it in `docker-compose.yml`:

```yaml
services:
  gateway:
    ports:
      - "YOUR_PORT:8080"
```

### Memory Issues

If you encounter memory issues, ensure Docker has at least 4GB of memory allocated.

## Next Steps

- See [Configuration Guide](../fechatter_server/docs/CONFIGURATION.md) for detailed configuration options
- Read the [Development Guide](../fechatter_server/docs/DEVELOPMENT_GUIDE.md) to start developing

## Support

If you encounter any issues during installation, please:
- Check the [troubleshooting section](#troubleshooting)
- Open an issue on GitHub
- Contact support at support@fechatter.io 