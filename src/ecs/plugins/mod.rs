pub mod player;
pub mod movement;
pub mod input;
pub mod debug;
pub mod network;

pub use player::PlayerPlugin;
pub use movement::MovementPlugin;
pub use input::InputPlugin;
pub use debug::DebugPlugin;
pub use network::{NetworkPlugin, NetworkMode};

use bevy::prelude::*;
use crate::ecs::core::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameConfig::default());
    }
}