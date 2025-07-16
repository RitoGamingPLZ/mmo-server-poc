/*!
# Debug Plugin

Provides development and debugging tools for monitoring the game server.

This plugin adds systems that help developers understand what's happening
in the game world during development.
*/

use bevy::prelude::*;
use crate::ecs::plugins::debug::systems::*;

/// Plugin that adds debugging and development tools to the game.
/// 
/// Features:
/// - Regular debug output showing player positions and velocities
/// - Server performance monitoring
/// - Safe debugging without unsafe code
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add the debug timer resource (replaces unsafe static)
            .init_resource::<DebugTimer>()
            
            // Add connection metrics tracking
            .insert_resource(ConnectionMetrics::new())
            
            // Add debug systems that run every frame
            // .add_systems(Update, debug_system);
    }
}