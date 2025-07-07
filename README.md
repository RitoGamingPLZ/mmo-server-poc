# 🎮 MMO Game Server

A real-time multiplayer game server built with Rust and Bevy ECS (Entity Component System).

## 🏗️ What Is This?

This is a **game server** that:
- Handles multiple players connecting at the same time
- Simulates a 2D physics world where players can move around
- Keeps all players synchronized - when one player moves, everyone sees it
- Provides a foundation you can build upon for your own multiplayer games

Think of it like the "backend" for a multiplayer game, similar to how game servers work for online games you might play.

## 🚀 Quick Start

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

---

## 🔧 Architecture Overview

The server uses a **plugin-based architecture** where each major system is implemented as a plugin:

```
📦 MMO Game Server
├── 🌍 CorePlugin       - Basic game world (positions, config)
├── 🌐 NetworkPlugin    - Internet connections (WebSocket/UDP)
├── 🎮 InputPlugin      - Player controls (movement commands)
├── 🏃 MovementPlugin   - Physics (acceleration, friction, collision)
├── 👥 PlayerPlugin     - Player lifecycle management (spawn/despawn)
└── 🐛 DebugPlugin      - Development tools (logging, monitoring)
```

Each plugin is independent and focused on one thing. This makes the code easier to understand and modify.

## 📚 Documentation

### Networking System
- **[Network Plugin README](src/ecs/plugins/network/README.md)** - Complete networking documentation and guide

### Plugin Documentation
- **[Core Plugin](src/ecs/core/README.md)** - Basic game world components
- **[Movement Plugin](src/ecs/plugins/movement/README.md)** - Physics and movement systems
- **[Player Plugin](src/ecs/plugins/player/README.md)** - Player lifecycle management

## 🎯 Key Concepts

### 1. **Entity Component System (ECS)**
Instead of traditional objects, Bevy uses ECS:
- **Entity**: A unique ID (like "Player 5" or "Projectile 23")
- **Component**: Data attached to entities (Position, Velocity, Health)
- **System**: Logic that processes entities with specific components

Example: A movement system processes all entities that have both Position and Velocity components.

### 2. **Real-Time Networking**
- Players connect via WebSocket (web browsers) or UDP (native games)
- Server sends only **changed data** to minimize bandwidth
- **Heartbeat system** keeps connections alive and detects disconnects
- **Client-aware sync** - new players get full state, others get just updates

### 3. **Physics Simulation**
- **Desired Velocity**: Where the player wants to go (from input)
- **Velocity**: Where the entity is actually going (smoothed)
- **Position**: Where the entity currently is
- **Friction**: Natural slowdown when not actively moving

## 📂 Code Structure

### Core Files You Should Understand:

1. **`src/main.rs`** - Entry point, sets up all plugins
2. **`src/ecs/core/components.rs`** - Basic game components (Position, etc.)
3. **`src/ecs/plugins/movement/systems.rs`** - Physics simulation
4. **`src/ecs/plugins/network/ws/systems.rs`** - WebSocket networking
5. **`src/ecs/plugins/input/systems.rs`** - Input processing

### Plugin Structure:
Each plugin has the same structure:
```
plugin_name/
├── components.rs  - Data structures
├── systems.rs     - Game logic
├── plugin.rs      - Glues everything together
└── README.md      - Documentation
```

## 🎮 Manual Testing (No Docker)

If you prefer to run without Docker:

1. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Start the server:**
   ```bash
   cargo run
   ```

3. **Connect a test client:**
   - Open your browser's developer console
   - Go to any webpage
   - Paste this JavaScript:
   ```javascript
   const ws = new WebSocket('ws://localhost:5000');
   ws.onopen = () => console.log('Connected!');
   ws.onmessage = (e) => console.log('Server:', e.data);
   
   // Send movement input
   ws.send('{"Move": {"direction": [1.0, 0.0]}}');  // Move right
   ws.send('{"Move": {"direction": [0.0, 1.0]}}');  // Move up
   ws.send('{"Stop": null}');                       // Stop moving
   
   // Send heartbeat (every 15 seconds to stay connected)
   setInterval(() => ws.send('heartbeat'), 15000);
   ```

## 🔧 Common Modifications

### Adding a new component:
```rust
// In components.rs
#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

// To make it networked:
networked_component! {
    pub struct NetworkedHealth {
        #[threshold = 1.0]  // Only sync when health changes > 1.0
        pub current: f32,
        pub max: f32,
    }
}

impl_from_source!(NetworkedHealth, Health, {current, max});
```

### Adding a new system:
```rust
// In systems.rs
pub fn regeneration_system(
    time: Res<Time>,
    mut query: Query<&mut Health>,
) {
    for mut health in query.iter_mut() {
        if health.current < health.max {
            health.current += 10.0 * time.delta_secs(); // 10 HP per second
            if health.current > health.max {
                health.current = health.max;
            }
        }
    }
}
```

### Adding it to your plugin:
```rust
// In plugin.rs
impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, regeneration_system)
            // If you want networking:
            .auto_sync_networked!(app, NetworkedHealth, Health);
    }
}
```

## 📊 Understanding the Debug Output

When you run the server, you'll see output like:
```
🎮 Game State Debug ===
👥 Connected players: 2
🎯 Player 1: pos(45.2, 23.1) vel(12.5, 0.0)
🎯 Player 2: pos(78.9, 67.3) vel(0.0, -8.2)
⏰ Server time: 15.3s
```

This shows:
- How many players are connected
- Each player's position and velocity
- How long the server has been running

## 🌐 Network Protocol

### Messages clients can send:
- `"heartbeat"` - Keep connection alive
- `{"Move": {"direction": [x, y]}}` - Move in direction (x,y should be -1.0 to 1.0)
- `{"Stop": null}` - Stop moving

### Messages server sends:
- `{"t": "full_sync", "es": [...]}` - Complete game state (for new/reconnecting players)
- `{"t": "delta_update", "es": [...]}` - Only changed entities (normal operation)

## 🎯 Performance Features

- **Fixed 20 TPS**: Physics runs at exactly 20 ticks per second for consistency
- **Delta Compression**: Only sends changed data, not everything every frame
- **Client-Aware Sync**: New clients get full state, returning clients get catch-up data
- **Heartbeat Monitoring**: Automatically removes disconnected clients
- **Bandwidth Optimization**: Compressed JSON field names reduce network usage by ~50%

## ⚡ Scalability Notes

**Current Capacity**: ~10-50 concurrent players  
**Educational Focus**: This server prioritizes code clarity over raw performance

For production MMO scale (1000+ players), significant architectural changes would be needed:
- Event-driven systems with lock-free queues
- Spatial partitioning for world state
- Separate network threads
- Component streaming and better serialization
- Distributed state management

## 🚧 What's Next?

Some ideas for extending this server:
1. **Add more game mechanics**: Health, weapons, items, NPCs
2. **Improve graphics**: Add a proper game client with sprites
3. **Add persistence**: Save player data to a database
4. **Add more networking**: Rooms, matchmaking, chat
5. **Add security**: Authentication, anti-cheat, rate limiting

## 🤝 Getting Help

- Read the comments in each file - they explain what everything does
- Check the README.md files in each plugin folder
- Look at the examples in the code
- The code is designed to be readable - don't be afraid to explore!

Remember: This is a learning project. Feel free to break things, experiment, and see how it all works together! 🎉