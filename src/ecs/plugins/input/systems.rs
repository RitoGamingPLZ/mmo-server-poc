use bevy::prelude::*;
use crate::ecs::core::GameConfig;
use crate::ecs::plugins::player::components::Player;
use crate::ecs::plugins::movement::components::Velocity;
use crate::ecs::plugins::input::components::*;

pub fn input_processing_system(
    mut input_buffer: ResMut<InputBuffer>,
    mut query: Query<(&Player, &mut Velocity)>,
    config: Res<GameConfig>,
) {
    for (player, mut velocity) in query.iter_mut() {
        if let Some(command) = input_buffer.commands.remove(&player.id) {
            match command {
                InputCommand::Move { direction } => {
                    let normalized = direction.normalize_or_zero();
                    velocity.x = normalized.x * config.player_speed;
                    velocity.y = normalized.y * config.player_speed;
                }
                InputCommand::Stop => {
                    velocity.x = 0.0;
                    velocity.y = 0.0;
                }
            }
        }
    }
}

pub fn input_validation_system(
    mut input_events: EventReader<InputCommandEvent>,
) {
    for event in input_events.read() {
        match &event.command {
            InputCommand::Move { direction } => {
                if direction.length() > 1.1 {
                    println!("Warning: Invalid move direction magnitude for player {}: {}", 
                        event.player_id, direction.length());
                }
            }
            InputCommand::Stop => {
                // Stop command is always valid
            }
        }
    }
}

pub fn input_event_system(
    mut input_events: EventReader<InputCommandEvent>,
    mut input_buffer: ResMut<InputBuffer>,
) {
    for event in input_events.read() {
        input_buffer.commands.insert(event.player_id, event.command.clone());
    }
}