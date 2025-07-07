# Player Plugin

This plugin manages player entities, including spawning, despawning, and player-specific components.

## Components

- `Player`: Core player identifier component
- `Health`: Player health with current and max values
- `CharacterProfile`: Player stats (speed, acceleration, health)
- `PlayerBundle`: Complete bundle for spawning new players

## NetworkedObject Integration

Players are automatically assigned `NetworkedObject` components for network synchronization:

```rust
impl PlayerBundle {
    pub fn new(player_id: u32, game_config: &GameConfig) -> Self {
        Self {
            player: Player { id: player_id },
            // ... other components
            networked_object: NetworkedObject::new_player(player_id),
        }
    }
}
```

## Player Spawning

Players spawn at random positions within the world bounds:

```rust
let mut rng = rand::thread_rng();
let x = rng.gen_range(0.0..game_config.world_bounds.x);
let y = rng.gen_range(0.0..game_config.world_bounds.y);
```

## Events

- `PlayerSpawnEvent`: Triggered when a new player should be spawned
- `PlayerDespawnEvent`: Triggered when a player disconnects

## Adding Networked Player Components

To add new networked components to players, use the macro system:

```rust
// Example: Adding networked health
networked_component! {
    pub struct NetworkedHealth {
        #[threshold = 1.0]  // Only sync health changes > 1.0
        pub current: f32,
        pub max: f32,
    }
}

impl_from_source!(NetworkedHealth, Health, {current, max});

// Register in plugin
auto_sync_networked!(app, NetworkedHealth, Health);
```

## Player Registry

The `PlayerRegistry` resource tracks all active players and their entity references for efficient lookup.