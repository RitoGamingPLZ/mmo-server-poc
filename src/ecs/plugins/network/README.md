# Network Plugin

This plugin provides the networking infrastructure for the MMO game server, including real-time synchronization of game state to connected clients.

## Architecture

The networking system is built around the `NetworkedObject` component, which allows any entity to be synchronized across the network, not just players.

### NetworkedObject System

```rust
#[derive(Component)]
pub struct NetworkedObject {
    pub network_id: u32,
    pub object_type: NetworkedObjectType,
}
```

Supported object types:
- `Player`: Player-controlled entities
- `NPC`: Non-player characters
- `Projectile`: Bullets, spells, etc.
- `Item`: Collectible items
- `Environment`: Interactive environment objects
- `Custom`: Custom entity types

## Macro-Based Networking

The new macro system eliminates the need for a central networked components file. Define networked components directly in your plugin:

### 1. Define a networked component:

```rust
use crate::{networked_component, impl_from_source};

networked_component! {
    pub struct NetworkedHealth {
        #[threshold = 1.0]  // Only sync when health changes > 1.0
        pub current: f32,
        pub max: f32,      // No threshold = uses default 0.01
    }
}

impl_from_source!(NetworkedHealth, Health, {current, max});
```

### 2. Register auto-sync in your plugin:

```rust
use crate::auto_sync_networked;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        // This automatically syncs Health -> NetworkedHealth
        auto_sync_networked!(app, NetworkedHealth, Health);
    }
}
```

## Delta Compression

The system uses delta compression to minimize network bandwidth:

- **Change Detection**: Only sends data when it changes beyond configurable thresholds
- **Field-Level Updates**: Only sends changed fields, not entire components
- **Batched Updates**: Groups multiple changes into single network messages
- **Message Compression**: Uses shortened JSON field names to reduce payload size

## Network Protocols

### WebSocket (WS)
- Real-time bidirectional communication
- JSON message format
- Automatic reconnection support
- Default port: 5000

### UDP (Legacy)
- Lower latency for fast-paced gameplay
- Binary message format using MessagePack
- Requires manual connection management

## Message Types

- `full_sync`: Complete game state (sent every 3 seconds)
- `delta_update`: Changed entities only (sent when changes detected)

## Usage Examples

### Adding a new networked entity type:

```rust
// Spawn an NPC that will be automatically networked
commands.spawn(NPCBundle::new_merchant(
    "Bob the Merchant".to_string(),
    Position { x: 100.0, y: 100.0 },
    &mut id_allocator,
));
```

### Making any component networked:

```rust
// In your component file
networked_component! {
    pub struct NetworkedInventory {
        #[threshold = 0.0]  // Sync any inventory change
        pub items: Vec<ItemId>,
        pub gold: u32,
    }
}

impl_from_source!(NetworkedInventory, Inventory, {items, gold});

// In your plugin
auto_sync_networked!(app, NetworkedInventory, Inventory);
```

The networking system handles the rest automatically!