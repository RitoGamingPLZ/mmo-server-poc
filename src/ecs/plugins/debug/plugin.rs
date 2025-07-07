use bevy::prelude::*;
use crate::ecs::plugins::debug::systems::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_system);
    }
}