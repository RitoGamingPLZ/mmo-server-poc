use bevy::prelude::*;
use std::collections::HashMap;
use rand::prelude::*;
use crate::ecs::core::{Position, GameConfig};
use crate::ecs::plugins::movement::components::{Velocity, DesiredVelocity, Friction};
use crate::ecs::plugins::network::NetworkedObject;

#[derive(Component, Debug, Clone, Copy)]
pub struct Player {
    pub id: u32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct CharacterProfile {
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub max_health: f32,
}

impl Default for CharacterProfile {
    fn default() -> Self {
        Self {
            max_speed: 100.0,
            acceleration: 200.0,
            deceleration: 300.0,
            max_health: 100.0,
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub position: Position,
    pub velocity: Velocity,
    pub desired_velocity: DesiredVelocity,
    pub health: Health,
    pub character_profile: CharacterProfile,
    pub friction: Friction,
    pub networked_object: NetworkedObject,
}

impl PlayerBundle {
    pub fn new(player_id: u32, game_config: &GameConfig) -> Self {
        let profile = CharacterProfile::default();
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0.0..game_config.world_bounds.x);
        let y = rng.gen_range(0.0..game_config.world_bounds.y);
        
        Self {
            player: Player { id: player_id },
            position: Position { x, y },
            velocity: Velocity { x: 0.0, y: 0.0 },
            desired_velocity: DesiredVelocity::default(),
            health: Health { current: profile.max_health, max: profile.max_health },
            character_profile: profile,
            friction: Friction::default(),
            networked_object: NetworkedObject::new_player(player_id),
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