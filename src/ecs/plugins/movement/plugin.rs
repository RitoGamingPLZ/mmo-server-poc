use bevy::prelude::*;
use crate::ecs::plugins::movement::systems::*;
use crate::ecs::plugins::movement::components::{Velocity, NetworkedVelocity};
use crate::auto_sync_networked;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (
            friction_system,
            acceleration_system,
            movement_system,
            boundary_system,
        ).chain())
        .insert_resource(Time::<Fixed>::from_hz(20.0));
        
        // Auto-register networked component sync for velocity
        auto_sync_networked!(app, NetworkedVelocity, Velocity);
    }
}