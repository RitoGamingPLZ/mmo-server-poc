# MMO Game Server - Localhost Setup

## Quick Start

### Prerequisites
- Docker and Docker Compose installed

#### Install Docker
**Ubuntu/Debian:**
```bash
# Update package index
sudo apt-get update

# Install Docker
sudo apt-get install docker.io docker-compose

# Add user to docker group (optional, avoids using sudo)
sudo usermod -aG docker $USER
newgrp docker
```

**macOS and Windows:**
- Download Docker Desktop from https://www.docker.com/products/docker-desktop/
- Install and start Docker Desktop

### Configuration
Copy the example environment files and customize if needed:
```bash
# Server configuration
cp .env.server.example .env.server

# Client configuration  
cp client-test-tool/.env.client.example client-test-tool/.env.client

# Edit files to change ports, hosts, or game settings
```

### Run the Application
```bash
# Build and start both services
docker-compose up --build

# Run in background
docker-compose up -d --build

# Stop services
docker-compose down
```

### Access
- **Game Client**: http://localhost:5173
- **Game Server**: ws://localhost:5000

### Controls
- **WASD** or **Arrow Keys**: Move
- **Space**: Stop
- **Click buttons**: UI controls

### Development
The client runs with hot-reload - edit files in `client-test-tool/src/` and changes will appear automatically.

For **server hot-reload** during development:
```bash
# Use the development compose file with cargo-watch
docker-compose -f docker-compose.dev.yml up --build
```
The server will automatically rebuild and restart when you edit Rust files in `src/`.

### Logs
```bash
# View all logs
docker-compose logs -f

# View server logs only
docker-compose logs -f game-server

# View client logs only
docker-compose logs -f client
```