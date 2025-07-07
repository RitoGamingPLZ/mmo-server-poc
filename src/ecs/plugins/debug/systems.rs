use bevy::prelude::*;
use crate::ecs::core::Position;
use crate::ecs::plugins::player::components::Player;
use crate::ecs::plugins::movement::components::Velocity;

pub fn debug_system(
    query: Query<(&Player, &Position, &Velocity)>,
    time: Res<Time>,
) {
    static mut LAST_PRINT: f32 = 0.0;
    unsafe {
        if time.elapsed_secs() - LAST_PRINT > 1.0 {
            println!("=== Game State ===");
            for (player, position, velocity) in query.iter() {
                println!("Player {}: pos({:.1}, {:.1}) vel({:.1}, {:.1})", 
                    player.id, position.x, position.y, velocity.x, velocity.y);
            }
            LAST_PRINT = time.elapsed_secs();
        }
    }
}