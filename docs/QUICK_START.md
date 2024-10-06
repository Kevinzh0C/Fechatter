# Quick Start Guide

Get Fechatter running in under 2 minutes!

## Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- 4GB RAM minimum

## Quick Setup

```bash
# 1. Clone the repository
git clone https://github.com/Kevinzh0C/fechatter.git
cd fechatter

# 2. Copy environment config
cp .env.example .env

# 3. Start all services
docker-compose up -d

# 4. Open in your browser
open http://localhost:8080
```

## Default Credentials

For development, use these test accounts:

- **Super Admin**: `super@test.com` / `password`
- **Developer**: `developer@test.com` / `password`  
- **Employee**: `employee@test.com` / `password`

## What's Next?

- 📖 Read the [Installation Guide](./INSTALLATION.md) for detailed setup
- 🏗️ Check out the [Architecture Overview](../ARCHITECTURE.md)
- 💻 See the [Development Guide](../fechatter_server/docs/DEVELOPMENT_GUIDE.md)

## Troubleshooting

### Services not starting?

```bash
# Check service status
docker-compose ps

# View logs
docker-compose logs -f
```

### Port conflicts?

If port 8080 is already in use:

```bash
# Stop conflicting service or change port in docker-compose.yml
services:
  gateway:
    ports:
      - "YOUR_PORT:8080"
```

### Need help?

- 📧 Email: support@fechatter.io
- 💬 GitHub Issues: [Report a problem](https://github.com/Kevinzh0C/fechatter/issues) 