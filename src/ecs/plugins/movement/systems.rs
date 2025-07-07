use bevy::prelude::*;
use crate::ecs::core::{Position, GameConfig};
use crate::ecs::plugins::movement::components::*;

pub fn movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Position, &Velocity)>,
) {
    for (mut position, velocity) in query.iter_mut() {
        position.x += velocity.x * time.delta_secs();
        position.y += velocity.y * time.delta_secs();
    }
}

pub fn boundary_system(
    mut query: Query<(&mut Position, &mut Velocity)>,
    config: Res<GameConfig>,
) {
    for (mut position, mut velocity) in query.iter_mut() {
        if position.x < 0.0 {
            position.x = 0.0;
            velocity.x = 0.0;
        }
        if position.x > config.world_bounds.x {
            position.x = config.world_bounds.x;
            velocity.x = 0.0;
        }
        if position.y < 0.0 {
            position.y = 0.0;
            velocity.y = 0.0;
        }
        if position.y > config.world_bounds.y {
            position.y = config.world_bounds.y;
            velocity.y = 0.0;
        }
    }
}