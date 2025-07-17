pub mod components;
pub mod systems;

use bevy::prelude::*;
use components::{NetworkIdAllocator, NetworkUpdates};
use systems::{detect_velocity_changes_system, detect_position_changes_system, build_delta_updates_system, build_full_sync_system};

// Network plugin for entity synchronization
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetworkIdAllocator::default())
            .insert_resource(NetworkUpdates::default())
            .add_systems(FixedUpdate, (
                detect_velocity_changes_system,
                detect_position_changes_system,
                build_delta_updates_system,
                build_full_sync_system.after(crate::ecs::systems::player_spawn_system),
            ));
    }
}