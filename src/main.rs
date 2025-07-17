/*!
# MMO Game Server

A real-time multiplayer game server built with Bevy ECS (Entity Component System).

## Architecture Overview

This server uses a plugin-based architecture where each major system is implemented as a plugin:

- **CorePlugin**: Basic game components (Position, game configuration)
- **NetworkPlugin**: Real-time networking (WebSocket + UDP) with smart sync
- **InputPlugin**: Player input processing and validation
- **MovementPlugin**: Physics simulation (movement, friction, boundaries)
- **PlayerPlugin**: Player lifecycle management (spawn/despawn)
- **DebugPlugin**: Development tools and logging

## Key Features

- **Smart Networking**: Client-aware synchronization with delta updates
- **Physics Simulation**: Real-time movement with friction and boundary collision
- **Multiple Protocols**: Both WebSocket (web clients) and UDP (game clients) support
- **Scalable Architecture**: Plugin system allows easy feature additions

## How It Works

1. Clients connect via WebSocket or UDP
2. Players spawn with random positions in the game world
3. Clients send input commands (Move, Stop)
4. Server simulates physics and updates all clients with changes
5. Only changed data is sent to minimize bandwidth usage

Run the server and connect clients to `ws://localhost:5000` for WebSocket
or UDP on the configured port.
*/

use bevy::prelude::*;

mod ecs;

use ecs::components::*;
use ecs::systems::*;
use ecs::{WebSocketPlugin, NetworkPlugin};

// Core game modules
/// Main entry point for the MMO game server.
/// 
/// Sets up the Bevy app with all necessary plugins and starts the game loop.
/// The server will listen for client connections and begin processing game logic.
fn main() {
    println!("🚀 Starting MMO Game Server...");
    println!("📡 Network Protocol: WebSocket");
    
    App::new()
        // Bevy's minimal plugins (no graphics/audio needed for server)
        .add_plugins(MinimalPlugins)
        
        // Add plugins
        .add_plugins(NetworkPlugin)
        .add_plugins(WebSocketPlugin::default())
        
        // Add resources
        .insert_resource(GameConfig::default())
        .insert_resource(PlayerRegistry::default())
        .insert_resource(Time::<Fixed>::from_hz(10.0))
        
        // Add events
        .add_event::<InputCommandEvent>()
        .add_event::<PlayerSpawnEvent>()
        .add_event::<PlayerDespawnEvent>()
        .add_event::<CharacterSpawnEvent>()
        .add_event::<CharacterDespawnEvent>()
        
        // Add systems
        .add_systems(FixedUpdate, (
            // Player management systems
            player_spawn_system,
            player_despawn_system,
            
            // Character management systems
            character_spawn_system,
            character_despawn_system,
            
            // Input systems
            input_processing_system,
            
            // Movement systems
            (
                acceleration_friction_system,
                movement_system,
                boundary_system
            ).chain()
            
        ))
        
        // Setup game world when server starts
        .add_systems(Startup, setup_game_world)
        
        // Start the game loop
        .run();
}

/// Initialize the game world and print startup information.
/// 
/// This runs once when the server starts up. Add any initial game state setup here.
fn setup_game_world(_commands: Commands) {
    println!("🌍 Game world initialized!");
    println!("🎮 Server ready for player connections");
    println!("📍 WebSocket: ws://localhost:5000");
    println!();
    println!("💡 Send 'heartbeat' messages every 15s to maintain connection");
    println!("📤 Input format: {{\"Move\": {{\"direction\": [1.0, 0.0]}}}}");
}

