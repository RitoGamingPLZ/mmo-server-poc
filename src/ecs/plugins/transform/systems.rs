use bevy::prelude::*;
use crate::ecs::plugins::transform::components::*;

/// System: Sync Position to NetworkPosition for networking
pub fn sync_position_to_network_system(
    mut query: Query<(&Position, &mut NetworkPosition), Changed<Position>>,
) {
    for (position, mut network_pos) in query.iter_mut() {
        // println!("Position sync: ({}, {}) -> ({}, {})", 
        //     network_pos.x, network_pos.y, position.x, position.y);
        network_pos.x = position.x;
        network_pos.y = position.y;
    }
}