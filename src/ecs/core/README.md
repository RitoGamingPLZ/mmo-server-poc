# Core Components

This module contains the fundamental components and resources used throughout the game.

## Components

- `Position`: Entity position in 2D space

## Networked Components

Core components use the macro-based networking system:

```rust
// Networked version of Position
networked_component! {
    pub struct NetworkedPosition {
        #[threshold = 0.01]  // Only sync when position changes > 0.01
        pub x: f32,
        #[threshold = 0.01]
        pub y: f32,
    }
}

// Auto-conversion from source component
impl_from_source!(NetworkedPosition, Position, {x, y});
```

### Auto-Sync Registration

Position synchronization is registered in the network plugin:

```rust
// In WsNetworkPlugin
auto_sync_networked!(app, NetworkedPosition, Position);
```

## Resources

- `GameConfig`: Global game configuration including world bounds and player settings

## Usage

The `Position` component is fundamental to most game entities. When added to an entity with a `NetworkedObject`, it will automatically be synchronized to all clients when it changes by more than the threshold amount (0.01 units).