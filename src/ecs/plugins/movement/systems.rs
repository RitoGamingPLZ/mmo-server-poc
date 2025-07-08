/*!
# Movement Systems

Physics simulation systems for entity movement, acceleration, and collision.

These systems handle the core game physics:
- Smooth acceleration/deceleration toward desired velocity
- Friction when not actively moving
- Position updates based on velocity
- Boundary collision with reflection physics
*/

use bevy::prelude::*;
use crate::ecs::core::GameConfig;
use crate::ecs::plugins::transform::Position;
use crate::ecs::plugins::movement::components::*;
use crate::ecs::plugins::player::components::CharacterProfile;

// Physics Constants - adjust these to change game feel
/// Minimum velocity threshold - below this is considered "not moving"
const MIN_VELOCITY_THRESHOLD: f32 = 0.01;

/// Minimum change threshold for smooth acceleration calculations
const MIN_CHANGE_THRESHOLD: f32 = 0.01;

/// World boundary positions
const WORLD_MIN_X: f32 = 0.0;
const WORLD_MIN_Y: f32 = 0.0;

/// Smoothly accelerates entities toward their desired velocity.
/// 
/// This system provides smooth movement by gradually changing the current velocity
/// toward the desired velocity, rather than instantly snapping to it.
/// 
/// Features:
/// - Uses different rates for acceleration vs deceleration  
/// - Respects max speed limits from character profile
/// - Smooth interpolation prevents jerky movement
/// - Handles both keyboard and analog input gracefully
pub fn acceleration_system(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &DesiredVelocity, &CharacterProfile)>,
) {
    let dt = time.delta_secs();
    
    for (mut velocity, desired_velocity, profile) in query.iter_mut() {
        // Calculate the difference between where we want to go and where we're going
        let diff_x = desired_velocity.x - velocity.x;
        let diff_y = desired_velocity.y - velocity.y;
        
        // Determine if we're trying to move or trying to stop
        let is_trying_to_move = desired_velocity.x.abs() > MIN_VELOCITY_THRESHOLD 
                             || desired_velocity.y.abs() > MIN_VELOCITY_THRESHOLD;
        
        // Use different acceleration rates for speeding up vs slowing down
        let acceleration_rate = if is_trying_to_move {
            profile.acceleration  // Accelerating toward movement
        } else {
            profile.deceleration  // Slowing down to stop
        };
        
        // Calculate how much we can change this frame
        let max_change_this_frame = acceleration_rate * dt;
        let change_magnitude = (diff_x * diff_x + diff_y * diff_y).sqrt();
        
        // Apply the velocity change (either full change or limited by acceleration rate)
        if change_magnitude > MIN_CHANGE_THRESHOLD {
            let change_factor = if change_magnitude > max_change_this_frame {
                // Limit the change to our acceleration rate
                max_change_this_frame / change_magnitude
            } else {
                // We can reach the desired velocity this frame
                1.0
            };
            
            let old_velocity_x = velocity.x;
            let old_velocity_y = velocity.y;
            velocity.x += diff_x * change_factor;
            velocity.y += diff_y * change_factor;
            
            if (velocity.x - old_velocity_x).abs() > 0.1 || (velocity.y - old_velocity_y).abs() > 0.1 {
                println!("🏃 Acceleration: desired=({:.2}, {:.2}) current=({:.2}, {:.2}) change_factor={:.3}", 
                    desired_velocity.x, desired_velocity.y, velocity.x, velocity.y, change_factor);
            }
        } else {
            // Very small difference - just snap to the desired velocity
            velocity.x = desired_velocity.x;
            velocity.y = desired_velocity.y;
        }
        
        // Enforce maximum speed limit
        let current_speed = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
        if current_speed > profile.max_speed {
            let scale_factor = profile.max_speed / current_speed;
            velocity.x *= scale_factor;
            velocity.y *= scale_factor;
        }
    }
}

/// Applies friction to slow down entities when they're not actively moving.
/// 
/// This system provides natural feeling deceleration when the player stops giving input.
/// Friction only applies when the entity isn't trying to accelerate in any direction.
/// 
/// How it works:
/// - Only applies when desired velocity is near zero (not actively moving)
/// - Uses exponential decay for smooth, natural feeling slowdown
/// - Snaps to zero when velocity gets very small to prevent jitter
/// - Coefficient of 0.95 means ~5% velocity loss per second at 60fps
pub fn friction_system(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &DesiredVelocity, &Friction)>,
) {
    let dt = time.delta_secs();
    
    for (mut velocity, desired_velocity, friction) in query.iter_mut() {
        // Check if the entity is trying to move
        let is_trying_to_move = desired_velocity.x.abs() > MIN_VELOCITY_THRESHOLD 
                             || desired_velocity.y.abs() > MIN_VELOCITY_THRESHOLD;
        
        // Only apply friction when not actively trying to move
        if !is_trying_to_move {
            // Apply exponential friction decay - feels natural and frame-rate independent
            let friction_factor = friction.coefficient.powf(dt);
            velocity.x *= friction_factor;
            velocity.y *= friction_factor;
            
            // Snap very small velocities to zero to prevent endless tiny movements
            if velocity.x.abs() < MIN_VELOCITY_THRESHOLD {
                velocity.x = 0.0;
            }
            if velocity.y.abs() < MIN_VELOCITY_THRESHOLD {
                velocity.y = 0.0;
            }
        }
    }
}

/// Updates entity positions based on their current velocity.
/// 
/// This is the core movement integration step that actually moves entities
/// through the game world. It applies the current velocity to update positions
/// each frame, scaled by the time since the last frame for smooth movement
/// regardless of framerate.
/// 
/// Simple physics integration: new_position = old_position + (velocity * time)
pub fn movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Position, &Velocity)>,
) {
    let dt = time.delta_secs();
    
    for (mut position, velocity) in query.iter_mut() {
        // Basic physics integration: position += velocity * time
        position.x += velocity.x * dt;
        position.y += velocity.y * dt;
    }
}

/// Handles collision with world boundaries using reflection physics.
/// 
/// When entities hit the edge of the game world, they bounce off in the
/// opposite direction rather than stopping or wrapping around.
/// 
/// Physics behavior:
/// - Clamps position to stay within world bounds
/// - Reflects velocity (negates the component that hit the boundary)
/// - Maintains speed but changes direction for realistic "bouncing"
pub fn boundary_system(
    mut query: Query<(&mut Position, &mut Velocity)>,
    config: Res<GameConfig>,
) {
    for (mut position, mut velocity) in query.iter_mut() {
        // Check and handle collision with left boundary
        if position.x < WORLD_MIN_X {
            position.x = WORLD_MIN_X;
            velocity.x = -velocity.x;  // Reflect horizontally
        }
        
        // Check and handle collision with right boundary
        if position.x > config.world_bounds.x {
            position.x = config.world_bounds.x;
            velocity.x = -velocity.x;  // Reflect horizontally
        }
        
        // Check and handle collision with bottom boundary
        if position.y < WORLD_MIN_Y {
            position.y = WORLD_MIN_Y;
            velocity.y = -velocity.y;  // Reflect vertically
        }
        
        // Check and handle collision with top boundary
        if position.y > config.world_bounds.y {
            position.y = config.world_bounds.y;
            velocity.y = -velocity.y;  // Reflect vertically
        }
    }
}

/// System: Sync Velocity to NetworkVelocity for networking
pub fn sync_velocity_to_network_system(
    mut query: Query<(&Velocity, &mut NetworkVelocity), Changed<Velocity>>,
) {
    for (velocity, mut network_vel) in query.iter_mut() {
        println!("Velocity sync: ({:.3}, {:.3}) -> ({:.3}, {:.3})", 
            network_vel.x, network_vel.y, velocity.x, velocity.y);
        network_vel.x = velocity.x;
        network_vel.y = velocity.y;
    }
}