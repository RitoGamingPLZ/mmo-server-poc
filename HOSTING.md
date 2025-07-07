# MMO Game Server Hosting Guide

## Prerequisites

- Docker and Docker Compose installed
- Git (for cloning/updating)
- At least 1GB RAM and 1 CPU core
- Network access to ports 5000 and 5173

## Local Development Setup

### 1. Clone and Setup
```bash
git clone <your-repo-url>
cd mmo_game_server
```

### 2. Run with Docker Compose
```bash
# Build and start both services
docker-compose up --build

# Run in background (detached mode)
docker-compose up -d --build

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### 3. Access Applications
- **Game Client**: http://localhost:5173 (with hot-reload)
- **Game Server**: ws://localhost:5000 (WebSocket)

## Production Deployment

### Option 1: Docker Compose (Recommended)

#### 1. Create Production Docker Compose
```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  game-server:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "5000:5000"
    networks:
      - mmo-network
    restart: always
    environment:
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:5000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  client:
    build:
      context: ./client-test-tool
      dockerfile: Dockerfile.prod
    ports:
      - "80:80"
    networks:
      - mmo-network
    depends_on:
      - game-server
    restart: always

networks:
  mmo-network:
    driver: bridge
```

#### 2. Create Production Client Dockerfile
```dockerfile
# client-test-tool/Dockerfile.prod
FROM node:18-alpine AS builder

WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

#### 3. Deploy
```bash
# Build and deploy
docker-compose -f docker-compose.prod.yml up -d --build

# Update deployment
docker-compose -f docker-compose.prod.yml pull
docker-compose -f docker-compose.prod.yml up -d
```

### Option 2: Cloud Deployment (AWS/GCP/Azure)

#### AWS EC2 Setup
```bash
# 1. Launch EC2 instance (Ubuntu 20.04+)
# 2. Install Docker
sudo apt update
sudo apt install -y docker.io docker-compose
sudo systemctl start docker
sudo systemctl enable docker
sudo usermod -aG docker $USER

# 3. Clone and deploy
git clone <your-repo-url>
cd mmo_game_server
docker-compose -f docker-compose.prod.yml up -d --build
```

#### Configure Security Groups
- **Port 80**: HTTP (0.0.0.0/0)
- **Port 5000**: WebSocket (0.0.0.0/0)
- **Port 22**: SSH (your IP only)

### Option 3: VPS Deployment

#### 1. Server Setup (Ubuntu/Debian)
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/download/v2.20.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

#### 2. Deploy Application
```bash
# Clone repository
git clone <your-repo-url>
cd mmo_game_server

# Create production environment
cp docker-compose.yml docker-compose.prod.yml
# Edit docker-compose.prod.yml as needed

# Deploy
docker-compose -f docker-compose.prod.yml up -d --build
```

#### 3. Setup Reverse Proxy (Optional)
```nginx
# /etc/nginx/sites-available/mmo-game
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://localhost:5173;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    location /ws {
        proxy_pass http://localhost:5000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Environment Configuration

### Server Configuration
```bash
# Environment variables
export RUST_LOG=info
export WEBSOCKET_PORT=5000
export WORLD_BOUNDS_X=1000
export WORLD_BOUNDS_Y=1000
export PLAYER_SPEED=100
```

### Client Configuration
```javascript
// Update client-test-tool/src/main.ts
const WS_URL = process.env.NODE_ENV === 'production' 
  ? 'ws://your-domain.com/ws'
  : 'ws://localhost:5000';
```

## Monitoring and Maintenance

### Health Checks
```bash
# Check service status
docker-compose ps

# View logs
docker-compose logs game-server
docker-compose logs client

# Resource usage
docker stats
```

### Backup Strategy
```bash
# Backup application data
docker-compose exec game-server tar -czf /backup/game-data.tar.gz /app/data

# Backup configuration
cp docker-compose.prod.yml /backup/
cp -r client-test-tool/src /backup/
```

### Updates
```bash
# Pull latest changes
git pull origin main

# Rebuild and redeploy
docker-compose -f docker-compose.prod.yml up -d --build

# Clean up old images
docker image prune -f
```

## Scaling Options

### Horizontal Scaling
```yaml
# docker-compose.scale.yml
version: '3.8'

services:
  game-server:
    build: .
    ports:
      - "5000-5002:5000"
    deploy:
      replicas: 3
    networks:
      - mmo-network

  nginx-lb:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx-lb.conf:/etc/nginx/nginx.conf
    depends_on:
      - game-server
    networks:
      - mmo-network
```

### Load Balancer Configuration
```nginx
# nginx-lb.conf
upstream game_servers {
    server game-server:5000;
    server game-server:5001;
    server game-server:5002;
}

server {
    listen 80;
    location / {
        proxy_pass http://game_servers;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## Troubleshooting

### Common Issues

1. **Port Already in Use**
```bash
# Find process using port
sudo netstat -tlnp | grep :5000
# Kill process
sudo kill -9 <PID>
```

2. **Docker Permission Issues**
```bash
# Add user to docker group
sudo usermod -aG docker $USER
# Restart session
logout && login
```

3. **WebSocket Connection Failed**
- Check firewall settings
- Verify port forwarding
- Check security groups (cloud)

### Performance Tuning

1. **Rust Server Optimization**
```dockerfile
# Dockerfile - optimized build
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release --target-cpu=native

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/mmo_game_server /app/
CMD ["./mmo_game_server"]
```

2. **Resource Limits**
```yaml
# docker-compose.yml
services:
  game-server:
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 256M
```

## Security Considerations

1. **Firewall Rules**
```bash
# UFW setup
sudo ufw allow 22/tcp
sudo ufw allow 80/tcp
sudo ufw allow 5000/tcp
sudo ufw enable
```

2. **SSL/TLS (Production)**
```bash
# Install Certbot
sudo apt install certbot python3-certbot-nginx

# Get certificate
sudo certbot --nginx -d your-domain.com

# Auto-renewal
sudo crontab -e
# Add: 0 12 * * * /usr/bin/certbot renew --quiet
```

3. **Container Security**
```dockerfile
# Run as non-root user
FROM debian:bookworm-slim
RUN useradd -r -s /bin/false gameuser
USER gameuser
```

## Quick Reference

### Commands
```bash
# Start services
docker-compose up -d --build

# Stop services
docker-compose down

# View logs
docker-compose logs -f

# Restart service
docker-compose restart game-server

# Scale service
docker-compose up -d --scale game-server=3

# Clean up
docker system prune -a
```

### URLs
- **Development**: http://localhost:5173
- **Production**: http://your-domain.com
- **WebSocket**: ws://your-domain.com:5000