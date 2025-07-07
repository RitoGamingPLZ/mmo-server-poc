use bevy::prelude::*;

#[derive(Resource)]
pub struct GameConfig {
    pub world_bounds: Vec2,
    pub player_speed: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            world_bounds: Vec2::new(1000.0, 1000.0),
            player_speed: 200.0,
        }
    }
}