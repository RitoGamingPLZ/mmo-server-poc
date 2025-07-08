/*!
# Debug Systems

Development and debugging tools for the game server.

These systems help developers monitor the game state during development.
All debug output can be safely removed in production builds.
*/

use bevy::prelude::*;
use crate::ecs::plugins::transform::{Position, NetworkPosition};
use crate::ecs::plugins::player::components::Player;
use crate::ecs::plugins::movement::components::Velocity;
use crate::ecs::plugins::network::NetworkId;

/// How often to print debug information (in seconds)
const DEBUG_PRINT_INTERVAL: f32 = 1.0;

/// Resource to track when we last printed debug info (replaces unsafe static)
#[derive(Resource, Default)]
pub struct DebugTimer {
    last_print_time: f32,
}

/// Debug system that prints game state information every second.
/// 
/// This helps developers see what's happening in the game world:
/// - Player positions and velocities
/// - How many players are connected
/// - Current game state
/// 
/// The system uses a safe Resource instead of unsafe static variables.
pub fn debug_system(
    player_query: Query<(&Player, &Position, &Velocity)>,
    network_query: Query<(Entity, &NetworkId, &NetworkPosition), With<Player>>,
    time: Res<Time>,
    mut debug_timer: ResMut<DebugTimer>,
) {
    let current_time = time.elapsed_secs();
    
    // Only print debug info every DEBUG_PRINT_INTERVAL seconds
    if current_time - debug_timer.last_print_time > DEBUG_PRINT_INTERVAL {
        println!("=== 🎮 Game State Debug ===");
        
        let player_count = player_query.iter().count();
        if player_count == 0 {
            println!("📭 No players connected");
        } else {
            println!("👥 Connected players: {}", player_count);
            
            for (player, position, velocity) in player_query.iter() {
                println!(
                    "🎯 Player {}: pos({:.1}, {:.1}) vel({:.1}, {:.1})", 
                    player.id, 
                    position.x, position.y, 
                    velocity.x, velocity.y
                );
            }
        }
        
        let networked_count = network_query.iter().count();
        println!("🌐 Networked players: {}", networked_count);
        for (entity, network_id, net_pos) in network_query.iter() {
            println!("📡 Entity {:?} NetworkID {} NetworkPos({:.1}, {:.1})", 
                entity, network_id.0, net_pos.x, net_pos.y);
        }
        
        println!("⏰ Server time: {:.1}s", current_time);
        println!(); // Empty line for readability
        
        debug_timer.last_print_time = current_time;
    }
}