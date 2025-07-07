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

- **Smart Networking**: Client-aware synchronization with heartbeat monitoring
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

use ecs::plugins::{CorePlugin, InputPlugin, MovementPlugin, PlayerPlugin, DebugPlugin, NetworkPlugin, NetworkMode};

// Export networking macros for use throughout the crate (used in component files)
#[allow(unused_imports)]
pub use ecs::plugins::network::auto_networked::*;
/// Main entry point for the MMO game server.
/// 
/// Sets up the Bevy app with all necessary plugins and starts the game loop.
/// The server will listen for client connections and begin processing game logic.
fn main() {
    // Choose networking protocol (WebSocket for web clients, UDP for native games)
    let use_udp = false; // TODO: Make this configurable via CLI args or config file
    
    let network_mode = if use_udp {
        NetworkMode::Udp
    } else {
        NetworkMode::Ws  // WebSocket - easier for web-based clients
    };

    println!("🚀 Starting MMO Game Server...");
    println!("📡 Network Protocol: {}", if use_udp { "UDP" } else { "WebSocket" });
    
    App::new()
        // Bevy's minimal plugins (no graphics/audio needed for server)
        .add_plugins(MinimalPlugins)
        
        // Game plugins (order matters - core systems first, then features)
        .add_plugins(CorePlugin)                              // Basic components & resources
        .add_plugins(NetworkPlugin { mode: network_mode })    // Client connections & sync
        .add_plugins(InputPlugin)                             // Handle player input
        .add_plugins(MovementPlugin)                          // Physics simulation 
        .add_plugins(PlayerPlugin)                            // Player management
        .add_plugins(DebugPlugin)                             // Development tools
        
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

/// Helper function for testing - simulates player input.
/// 
/// This allows tests to inject input commands directly into the input buffer
/// without going through the network layer.
/// 
/// # Arguments
/// * `input_buffer` - The server's input command buffer
/// * `player_id` - Which player is sending the command
/// * `command` - The input command to process
pub fn simulate_input(
    input_buffer: &mut crate::ecs::plugins::input::InputBuffer, 
    player_id: u32, 
    command: crate::ecs::plugins::input::InputCommand
) {
    input_buffer.commands.insert(player_id, command);
}