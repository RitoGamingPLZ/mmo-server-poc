use bevy::prelude::*;
use std::collections::HashMap;
use crate::ecs::core::{Position, NetworkId};
use crate::ecs::plugins::movement::components::Velocity;

#[derive(Component, Debug, Clone, Copy)]
pub struct Player {
    pub id: u32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub position: Position,
    pub velocity: Velocity,
    pub health: Health,
    pub network_id: NetworkId,
}

impl PlayerBundle {
    pub fn new(player_id: u32) -> Self {
        Self {
            player: Player { id: player_id },
            position: Position { x: 0.0, y: 0.0 },
            velocity: Velocity { x: 0.0, y: 0.0 },
            health: Health { current: 100.0, max: 100.0 },
            network_id: NetworkId { id: player_id },
        }
    }
}

#[derive(Event)]
pub struct PlayerSpawnEvent {
    pub player_id: u32,
}

#[derive(Event)]
pub struct PlayerDespawnEvent {
    pub player_id: u32,
}

#[derive(Resource, Default)]
pub struct PlayerRegistry {
    pub players: HashMap<u32, Entity>,
}

impl PlayerRegistry {
    pub fn register_player(&mut self, player_id: u32, entity: Entity) {
        self.players.insert(player_id, entity);
    }
    
    pub fn unregister_player(&mut self, player_id: u32) -> Option<Entity> {
        self.players.remove(&player_id)
    }
    
    pub fn get_player_entity(&self, player_id: u32) -> Option<Entity> {
        self.players.get(&player_id).copied()
    }
}