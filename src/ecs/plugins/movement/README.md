# Movement Plugin

This plugin handles entity movement, velocity, and physics.

## Components

- `Velocity`: Current velocity of an entity
- `DesiredVelocity`: Target velocity for input-driven movement
- `Friction`: Friction coefficient for natural deceleration

## Networked Components

This plugin uses the new macro-based networking system:

```rust
// Networked version with custom thresholds
networked_component! {
    pub struct NetworkedVelocity {
        #[threshold = 0.01]  // Only sync when change > 0.01
        pub x: f32,
        #[threshold = 0.01]
        pub y: f32,
    }
}

// Auto-conversion from source component
impl_from_source!(NetworkedVelocity, Velocity, {x, y});
```

### Auto-Sync Registration

The plugin automatically registers velocity synchronization:

```rust
// In plugin.rs
auto_sync_networked!(app, NetworkedVelocity, Velocity);
```

This ensures that whenever a `Velocity` component changes on an entity with a `NetworkedObject`, the corresponding `NetworkedVelocity` is automatically updated and will be synchronized to all clients.

## Systems

- `friction_system`: Applies friction to reduce velocity over time
- `acceleration_system`: Applies desired velocity to actual velocity
- `movement_system`: Updates position based on velocity
- `boundary_system`: Handles collision with world boundaries (reflection)

All systems run on `FixedUpdate` at 20 TPS for consistent physics simulation.