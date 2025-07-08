use bevy::prelude::*;
use crate::ecs::plugins::player::components::*;
use crate::ecs::plugins::player::systems::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerSpawnEvent>()
            .add_event::<PlayerDespawnEvent>()
            .insert_resource(PlayerRegistry::default())
            .add_systems(Update, player_despawn_system);
    }
}