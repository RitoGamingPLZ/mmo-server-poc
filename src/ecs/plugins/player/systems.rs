use bevy::prelude::*;
use crate::ecs::core::GameConfig;
use crate::ecs::plugins::player::components::*;

pub fn player_spawn_system(
    mut commands: Commands,
    mut spawn_events: EventReader<PlayerSpawnEvent>,
    mut player_registry: ResMut<PlayerRegistry>,
    game_config: Res<GameConfig>,
) {
    for event in spawn_events.read() {
        let entity = commands.spawn(PlayerBundle::new(event.player_id, &game_config)).id();
        player_registry.register_player(event.player_id, entity);
        println!("Spawned player entity for player ID: {} -> Entity: {:?}", event.player_id, entity);
    }
}

pub fn player_despawn_system(
    mut commands: Commands,
    mut despawn_events: EventReader<PlayerDespawnEvent>,
    mut player_registry: ResMut<PlayerRegistry>,
) {
    for event in despawn_events.read() {
        if let Some(entity) = player_registry.unregister_player(event.player_id) {
            commands.entity(entity).despawn();
            println!("Despawned player entity for player ID: {}", event.player_id);
        }
    }
}