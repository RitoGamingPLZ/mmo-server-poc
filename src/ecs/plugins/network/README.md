# Network Plugin

This plugin provides the networking infrastructure for the MMO game server, including real-time synchronization of game state to connected clients.

## Quick Start

The networking system uses **auto-registration** - components register themselves when declared!

### 1. Define a networked component:

```rust
use crate::{networked_component, impl_from_source};

networked_component! {
    pub struct NetworkedHealth {
        #[threshold = 1.0]  // Only sync when health changes > 1.0
        pub current: f32,
        pub max: f32,
    }
}

impl_from_source!(NetworkedHealth, Health, {current, max});
```

### 2. Register auto-sync in your plugin:

```rust
use crate::auto_sync_networked;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        auto_sync_networked!(app, NetworkedHealth, Health);
    }
}
```

### 3. That's it!
Your component automatically:
- ✅ Registers itself in the networking system
- ✅ Syncs with configurable thresholds
- ✅ Includes delta updates and full sync
- ✅ Compresses messages for efficiency

---

## Architecture

The networking system uses a **trait-based registry approach** with these key components:

### NetworkedObject System
```rust
#[derive(Component)]
pub struct NetworkedObject {
    pub network_id: u32,
    pub object_type: NetworkedObjectType,
}
```

Supported types: `Player`, `NPC`, `Projectile`, `Item`, `Environment`, `Custom`

### NetworkedState Trait
All networked components implement `NetworkedState` for automatic field change detection and serialization.

### Component Registry
The `NetworkedComponentRegistry` resource automatically collects and handles all registered networked components.

### Features
- **Auto-Registration**: Components register themselves when declared
- **Single Query**: One query handles all networked components
- **Delta Compression**: Only changed fields are synchronized
- **WebSocket Protocol**: Real-time bidirectional communication (port 5000)
- **Message Types**: `full_sync` and `delta_update`

---

## Advanced Features

### Custom Thresholds
Control when changes are considered significant enough to sync:

```rust
networked_component! {
    pub struct NetworkedTransform {
        #[threshold = 0.01]   // High precision for position
        pub x: f32,
        #[threshold = 0.01]
        pub y: f32,
        #[threshold = 0.1]    // Lower precision for rotation
        pub rotation: f32,
    }
}
```

### Complex Data Types
The system supports any serializable data type:

```rust
networked_component! {
    pub struct NetworkedPlayer {
        pub name: String,                     // Syncs when name changes
        pub items: Vec<String>,               // Syncs when inventory changes
        #[threshold = 0.01]
        pub health: f32,                      // Syncs when health changes > 0.01
        pub stats: HashMap<String, u32>,      // Syncs when stats change
    }
}
```

### Conditional Synchronization
Implement custom logic for when components should sync:

```rust
impl NetworkedState for NetworkedSpecialComponent {
    fn get_field_changes(&self, previous: Option<&Self>) -> Vec<FieldUpdate> {
        if let Some(prev) = previous {
            if self.should_sync_to_clients() {
                // Return field updates
            } else {
                return Vec::new(); // Don't sync
            }
        }
        // ... implementation
    }
}
```

---

## Performance

### Delta Compression
- **Change Detection**: Only sends data when it changes beyond configurable thresholds
- **Field-Level Updates**: Only sends changed fields, not entire components
- **Batched Updates**: Groups multiple changes into single network messages
- **Message Compression**: Uses shortened JSON keys for efficiency

### Scalable Architecture
- **Single Query**: Uses one `Query<(Entity, &NetworkedObject)>` instead of N queries
- **Registry Pattern**: Automatic component handling via trait-based registry
- **Auto-Registration**: Components register themselves at compile time
- **Lazy Synchronization**: Only syncs when data actually changes

### Before vs After Performance

**Old Approach (Manual Queries):**
```rust
// BAD: Required separate queries for each component type
pub fn sync_system(
    position_query: Query<(Entity, &NetworkedPosition, &NetworkedObject)>,
    velocity_query: Query<(Entity, &NetworkedVelocity, &NetworkedObject)>,
    health_query: Query<(Entity, &NetworkedHealth, &NetworkedObject)>,
    // ... 10 more queries for 10 more components
) {
    // Manual sync logic for each component type
}
```

**New Approach (Registry-Based):**
```rust
// GOOD: Single query + registry handles all components automatically
pub fn sync_system(
    networked_query: Query<(Entity, &NetworkedObject)>,
    registry: Res<NetworkedComponentRegistry>,
    world: &World,
) {
    // All components handled automatically through the registry
    let updates = build_delta_updates_registry(&networked_query, world, &registry, snapshot);
}
```

---

## Usage Examples

### Complete Combat System Example

```rust
// File: src/ecs/plugins/combat/components.rs
use bevy::prelude::*;
use crate::{networked_component, impl_from_source};

// Regular components
#[derive(Component, Debug, Clone, Copy)]
pub struct Health { pub current: f32, pub max: f32 }

#[derive(Component, Debug, Clone, Copy)]
pub struct Mana { pub current: f32, pub max: f32 }

// Networked versions - they auto-register!
networked_component! {
    pub struct NetworkedHealth {
        #[threshold = 0.1] pub current: f32,
        #[threshold = 0.1] pub max: f32,
    }
}

networked_component! {
    pub struct NetworkedMana {
        #[threshold = 0.1] pub current: f32,
        #[threshold = 0.1] pub max: f32,
    }
}

// Conversions
impl_from_source!(NetworkedHealth, Health, {current, max});
impl_from_source!(NetworkedMana, Mana, {current, max});

// Plugin setup
impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        auto_sync_networked!(app, NetworkedHealth, Health);
        auto_sync_networked!(app, NetworkedMana, Mana);
    }
}
```

**Result**: Fully networked combat system with automatic delta updates, full sync for new clients, and zero modification to networking core files.

### Spawning Networked Entities

```rust
// Spawn an NPC that will be automatically networked
commands.spawn((
    NPCBundle::new_merchant("Bob the Merchant".to_string()),
    Position { x: 100.0, y: 100.0 },
    NetworkedObject::new_npc(id_allocator.allocate_id()),
    Health { current: 100.0, max: 100.0 },
));
```

---

## Migration Guide

### From Manual Registration
If you have existing manual registrations, you can remove them:

```rust
// OLD: Remove this from component_registry.rs
register_networked_components!(
    NetworkedPosition,
    NetworkedVelocity,
    NetworkedHealth,
    // ... etc
);

// NEW: Nothing needed! Components auto-register.
```

### Updating Existing Components
1. Replace manual `NetworkedState` implementations with `networked_component!` macro
2. Add `auto_sync_networked!` calls to your plugins
3. Remove manual registration code

---

## Debugging

### Check Registered Components
```rust
fn debug_networking_system(registry: Res<NetworkedComponentRegistry>) {
    println!("Registered {} networked components", registry.syncers.len());
}
```

### Network Message Logging
The system automatically logs all sync operations:
```
Full sync to player 1 (reconnection): 15 entities, 2048 bytes
Delta update to player 2 (changes): 3 entities, 256 bytes
```

---

## Key Benefits

### Before: Manual Registration Required
- Adding 10 components required 10+ lines of manual registration
- Easy to forget registrations, causing bugs
- Separate queries needed for each component type
- Central registry file needed constant updates

### After: Auto-Registration
- Adding 10 components requires 0 lines of registration
- Components auto-register when declared
- Single query handles all component types
- Zero central registry maintenance

### Performance Benefits
- **Compile-Time Registration**: No runtime overhead
- **Single Query**: Efficient single-query approach
- **Lazy Registration**: Components only register when used
- **No Duplication**: Built-in duplicate prevention

---

## Summary

**Before**: Adding 10 networked components required 10+ lines of manual registration in a central file.

**After**: Adding 10 networked components requires 0 lines of registration - they register themselves automatically!

The networking system is now at its most elegant state: **declare components and they automatically work across the network**. No manual registration, no boilerplate, no central registry maintenance! 🎉