use bevy::prelude::*;
use crate::ecs::plugins::movement::systems::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (
            friction_system,
            acceleration_system,
            movement_system,
            boundary_system,
            sync_velocity_to_network_system,
        ).chain());
    }
}