# Network Plugin

Real-time networking system for MMO game server with delta updates and WebSocket communication.

## Overview

The networking system provides:
- **Delta Updates**: Only sends changed components to clients
- **Full Sync**: Periodic complete state synchronization 
- **WebSocket Protocol**: Real-time bidirectional communication on port 5000
- **Change Detection**: Automatic detection of component modifications
- **Network Components**: Separate networking versions of game components

## Architecture

### Core Components

```rust
// Network identification for entities
#[derive(Component)]
pub struct NetworkId(pub u32);

// Tracks which components changed since last sync
#[derive(Component)]
pub struct NetworkDirty {
    pub changed_components: Vec<String>,
}

// Stores previous component values for change detection
#[derive(Component)]
pub struct NetworkSnapshot {
    pub components: HashMap<String, serde_json::Value>,
}
```

### Network Message Format

```json
{
  "message_type": "delta_update",
  "entity_updates": [
    {
      "network_id": 1,
      "components": {
        "position": {"x": 100.5, "y": 200.3},
        "velocity": {"x": 0.0, "y": 0.0}
      }
    }
  ]
}
```

## Component System

### Game Components vs Network Components

Each game component has a corresponding network component:

```rust
// Game component (physics, game logic)
#[derive(Component)]
pub struct Position { pub x: f32, pub y: f32 }

// Network component (serialization, networking)
#[derive(Component, Serialize, Deserialize)]
pub struct NetworkPosition { pub x: f32, pub y: f32 }
```

### Sync Systems

Sync systems bridge game components to network components:

```rust
pub fn sync_position_to_network_system(
    mut query: Query<(&Position, &mut NetworkPosition), Changed<Position>>,
) {
    for (position, mut network_pos) in query.iter_mut() {
        network_pos.x = position.x;
        network_pos.y = position.y;
    }
}
```

## System Flow

### 1. Player Input → Game State Updates
```
WebSocket Input → InputCommand → DesiredVelocity → Movement System → Position/Velocity
```

### 2. Game State → Network Sync
```
Position/Velocity → sync_*_to_network_system → NetworkPosition/NetworkVelocity
```

### 3. Change Detection & Broadcasting
```
NetworkComponents → detect_component_changes_system → build_delta_updates_system → WebSocket Broadcast
```

## Adding New Networked Components

### 1. Define Components

```rust
// Game component
#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

// Network component
#[derive(Component, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkHealth {
    pub current: f32,
    pub max: f32,
}

impl NetworkedComponent for NetworkHealth {
    fn component_name() -> &'static str { "health" }
}
```

### 2. Add Sync System

```rust
pub fn sync_health_to_network_system(
    mut query: Query<(&Health, &mut NetworkHealth), Changed<Health>>,
) {
    for (health, mut network_health) in query.iter_mut() {
        network_health.current = health.current;
        network_health.max = health.max;
    }
}
```

### 3. Register Systems

```rust
impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, sync_health_to_network_system);
    }
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (
            detect_component_changes_system::<NetworkHealth>,
            // ... other systems
        ).chain()
        .after(sync_health_to_network_system));
    }
}
```

## System Ordering

Critical system ordering for proper networking:

```
1. Game Logic Systems (movement, combat, etc.)
2. Sync Systems (sync_*_to_network_system)  
3. Network Systems (change detection, delta updates, broadcasting)
```

This ensures network components reflect final game state before transmission.

## Performance Features

### Delta Updates
- Only changed components are sent to clients
- Reduces bandwidth usage significantly
- Automatic change detection using Bevy's `Changed<T>` queries

### Full Sync
- Periodic complete state synchronization (every 10 seconds)
- Ensures client consistency
- Handles packet loss recovery

### Batched Messages
- Multiple entity updates grouped into single WebSocket message
- Reduces network overhead
- JSON serialization for human-readable debugging

## Debugging

### Network Traffic Logging
```rust
// In build_delta_updates_system
for update in &entity_updates {
    println!("Entity {}: {:?}", update.network_id, update.components);
}
```

### Component Sync Logging
```rust
// In sync systems
println!("Position sync: ({}, {}) -> ({}, {})", 
    network_pos.x, network_pos.y, position.x, position.y);
```

### WebSocket Connection Logging
```
✅ Added networking to player 1 entity with network ID 123 at (100.0, 200.0)
WS Player 1 connected from 127.0.0.1:54321
Entity 123: {"position": {"x": 100.5, "y": 200.3}}
```

## Configuration

### Network Update Rate
Systems run in `FixedUpdate` at 20Hz for consistent packet rate:

```rust
.add_systems(FixedUpdate, (/* network systems */))
```

### WebSocket Server
- Host: `WEBSOCKET_HOST` environment variable (default: "0.0.0.0")
- Port: `WEBSOCKET_PORT` environment variable (default: "5000")

## Client Integration

Clients connect via WebSocket and receive JSON messages:

```javascript
const ws = new WebSocket('ws://localhost:5000');
ws.onmessage = (event) => {
    const update = JSON.parse(event.data);
    if (update.message_type === 'delta_update') {
        // Apply entity updates to client game state
    }
};
```

## Common Issues

### "Still Moving When Velocity is Zero"
Ensure sync systems run before network change detection:
```rust
.after(sync_velocity_to_network_system)
```

### "Could Not Find Player Entity"
Ensure player spawn system runs before networking systems:
```rust
(player_spawn_system, add_networking_to_players_system).chain()
```

### High Bandwidth Usage
Check that only necessary components implement `NetworkedComponent` and use appropriate change thresholds.