# MMO Game Server - Localhost Setup

## Quick Start

### Prerequisites
- Docker and Docker Compose installed

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

### Logs
```bash
# View all logs
docker-compose logs -f

# View server logs only
docker-compose logs -f game-server

# View client logs only
docker-compose logs -f client
```