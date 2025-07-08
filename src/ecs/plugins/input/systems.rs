/*!
# Input Systems

Player input processing and validation systems.

These systems handle converting raw player input into game actions:
- Processing movement commands from network clients
- Validating input to prevent cheating or invalid data
- Converting input into desired velocities for the physics system
*/

use bevy::prelude::*;
use crate::ecs::plugins::player::components::{Player, CharacterProfile};
use crate::ecs::plugins::movement::components::DesiredVelocity;
use crate::ecs::plugins::input::components::*;

/// Maximum allowed input vector magnitude (prevents speed hacking)
const MAX_INPUT_MAGNITUDE: f32 = 1.1;

/// Processes input commands from the input buffer and applies them to player entities.
/// 
/// This system converts raw input commands into desired velocities that the
/// physics systems can then use to smoothly move players around the world.
/// 
/// Input types:
/// - Move: Sets desired velocity based on direction and player's max speed
/// - Stop: Sets desired velocity to zero for immediate stopping
pub fn input_processing_system(
    mut input_buffer: ResMut<InputBuffer>,
    mut query: Query<(&Player, &mut DesiredVelocity, &CharacterProfile)>,
) {
    for (player, mut desired_velocity, profile) in query.iter_mut() {
        // Check if this player has any pending input commands
        if let Some(command) = input_buffer.commands.remove(&player.id) {
            match command {
                InputCommand::Move { direction } => {
                    // Normalize the direction to prevent speed hacking
                    // (input might be [2.0, 2.0] but we cap it at magnitude 1.0)
                    let normalized_direction = direction.normalize_or_zero();
                    
                    // Apply the player's max speed to get the desired velocity
                    desired_velocity.x = normalized_direction.x * profile.max_speed;
                    desired_velocity.y = normalized_direction.y * profile.max_speed;
                }
                InputCommand::Stop => {
                    // Player wants to stop moving
                    desired_velocity.x = 0.0;
                    desired_velocity.y = 0.0;
                }
            }
        }
    }
}

/// Resets desired velocity to zero for players who didn't receive input this frame.
/// This allows friction to apply properly for momentum-based movement.
pub fn reset_desired_velocity_system(
    input_buffer: Res<InputBuffer>,
    mut query: Query<(&Player, &mut DesiredVelocity)>,
) {
    for (player, mut desired_velocity) in query.iter_mut() {
        // If no input was processed for this player this frame, reset desired velocity
        if !input_buffer.commands.contains_key(&player.id) {
            desired_velocity.x = 0.0;
            desired_velocity.y = 0.0;
        }
    }
}

/// Validates input commands to detect potential cheating or invalid data.
/// 
/// This system checks incoming input for suspicious values that might indicate:
/// - Modified game clients trying to move faster than allowed
/// - Network corruption causing invalid data
/// - Bugs in client input handling
/// 
/// Currently validates:
/// - Movement direction magnitude (should be ≤ 1.0 for normal input)
pub fn input_validation_system(
    mut input_events: EventReader<InputCommandEvent>,
) {
    for event in input_events.read() {
        match &event.command {
            InputCommand::Move { direction } => {
                let magnitude = direction.length();
                
                // Check for suspiciously large input vectors
                if magnitude > MAX_INPUT_MAGNITUDE {
                    println!(
                        "⚠️  WARNING: Player {} sent invalid move direction magnitude: {:.2} (max: {:.2})", 
                        event.player_id, 
                        magnitude,
                        MAX_INPUT_MAGNITUDE
                    );
                    
                    // TODO: In production, you might want to:
                    // - Log this to an anti-cheat system
                    // - Temporarily flag the player for monitoring
                    // - Automatically kick repeat offenders
                }
            }
            InputCommand::Stop => {
                // Stop commands are always considered valid
                // (no parameters to validate)
            }
        }
    }
}

/// Transfers input commands from events into the input buffer for processing.
/// 
/// This system acts as a bridge between the network layer (which sends InputCommandEvents)
/// and the input processing system (which reads from the InputBuffer).
/// 
/// The separation allows for:
/// - Buffering multiple inputs if needed
/// - Input validation before processing
/// - Debugging and logging of all player input
pub fn input_event_system(
    mut input_events: EventReader<InputCommandEvent>,
    mut input_buffer: ResMut<InputBuffer>,
) {
    for event in input_events.read() {
        // Store the latest input command for each player
        // Note: This overwrites any previous command for the same player in the same frame
        // which is usually the desired behavior for real-time games
        input_buffer.commands.insert(event.player_id, event.command.clone());
    }
}