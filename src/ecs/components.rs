use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// INPUT COMPONENTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputCommand {
    Move { direction: Vec2 },
    Stop,
}

#[derive(Event)]
pub struct InputCommandEvent {
    pub player_id: u32,
    pub command: InputCommand,
}


// ============================================================================
// MOVEMENT COMPONENTS
// ============================================================================

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct DesiredVelocity {
    pub x: f32,
    pub y: f32,
}

impl Default for DesiredVelocity {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Friction {
    pub coefficient: f32,
}

impl Default for Friction {
    fn default() -> Self {
        Self { coefficient: 0.98 }
    }
}

// ============================================================================
// TRANSFORM COMPONENTS
// ============================================================================

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

// ============================================================================
// PLAYER COMPONENTS
// ============================================================================

#[derive(Component, Debug, Clone, Copy)]
pub struct Player {
    pub id: u32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ViewDistance {
    pub radius: f32,
}

impl Default for ViewDistance {
    fn default() -> Self {
        Self { radius: 300.0 } // Default view radius
    }
}


#[derive(Component, Debug, Clone, Copy)]
pub struct CharacterProfile {
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
}

impl Default for CharacterProfile {
    fn default() -> Self {
        Self {
            max_speed: 100.0,
            acceleration: 200.0,
            deceleration: 300.0,
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub position: Position,
    pub velocity: Velocity,
    pub desired_velocity: DesiredVelocity,
    pub character_profile: CharacterProfile,
    pub friction: Friction,
    pub view_distance: ViewDistance,
}

impl PlayerBundle {
    pub fn new(player_id: u32, game_config: &GameConfig) -> Self {
        let profile = CharacterProfile::default();
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0.0..game_config.world_bounds.x);
        let y = rng.gen_range(0.0..game_config.world_bounds.y);
        
        Self {
            player: Player { id: player_id },
            position: Position { x, y },
            velocity: Velocity { x: 0.0, y: 0.0 },
            desired_velocity: DesiredVelocity::default(),
            character_profile: profile,
            friction: Friction::default(),
            view_distance: ViewDistance::default(),
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Character {
    pub id: u32,
}

#[derive(Bundle)]
pub struct CharacterBundle {
    pub character: Character,
    pub position: Position,
    pub velocity: Velocity,
    pub desired_velocity: DesiredVelocity,
    pub character_profile: CharacterProfile,
    pub friction: Friction,
}

impl CharacterBundle {
    pub fn new(character_id: u32, position: Option<Position>, game_config: &GameConfig) -> Self {
        let profile = CharacterProfile::default();
        let pos = position.unwrap_or_else(|| {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            Position {
                x: rng.gen_range(0.0..game_config.world_bounds.x),
                y: rng.gen_range(0.0..game_config.world_bounds.y),
            }
        });
        
        Self {
            character: Character { id: character_id },
            position: pos,
            velocity: Velocity { x: 0.0, y: 0.0 },
            desired_velocity: DesiredVelocity::default(),
            character_profile: profile,
            friction: Friction::default(),
        }
    }
}


// ============================================================================
// EVENTS
// ============================================================================

#[derive(Event)]
pub struct PlayerSpawnEvent {
    pub player_id: u32,
}

#[derive(Event)]
pub struct PlayerDespawnEvent {
    pub player_id: u32,
}

#[derive(Event)]
pub struct CharacterSpawnEvent {
    pub character_id: u32,
    pub position: Option<Position>,
}

#[derive(Event)]
pub struct CharacterDespawnEvent {
    pub character_id: u32,
}


// ============================================================================
// RESOURCES
// ============================================================================

#[derive(Resource)]
pub struct GameConfig {
    pub world_bounds: Vec2,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            world_bounds: Vec2::new(1000.0, 1000.0),
        }
    }
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

