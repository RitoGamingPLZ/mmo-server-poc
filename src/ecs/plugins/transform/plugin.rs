use bevy::prelude::*;
use crate::ecs::plugins::transform::systems::sync_position_to_network_system;

pub struct TransformPlugin;

impl Plugin for TransformPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, sync_position_to_network_system)
           .insert_resource(Time::<Fixed>::from_hz(10.0));
    }
}