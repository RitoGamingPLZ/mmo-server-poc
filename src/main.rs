use bevy::prelude::*;

mod ecs;

use ecs::plugins::{CorePlugin, InputPlugin, MovementPlugin, PlayerPlugin, DebugPlugin, NetworkPlugin, NetworkMode};
fn main() {
    let use_udp = false; // set based on CLI/config/env

    let network_mode = if use_udp {
        NetworkMode::Udp
    } else {
        NetworkMode::Ws
    };

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(CorePlugin)
        .add_plugins(NetworkPlugin { mode: network_mode })
        .add_plugins(InputPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(DebugPlugin)
        .add_systems(Startup, setup_game)
        .run();
}

fn setup_game(mut commands: Commands) {
    println!("Starting 2D Shooter Game Server...");
}

pub fn simulate_input(input_buffer: &mut crate::ecs::plugins::input::InputBuffer, player_id: u32, command: crate::ecs::plugins::input::InputCommand) {
    input_buffer.commands.insert(player_id, command);
}