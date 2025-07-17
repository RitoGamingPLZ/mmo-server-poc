use bevy::prelude::*;
use crate::ecs::components::*;

// ============================================================================
// INPUT SYSTEMS
// ============================================================================

const MAX_INPUT_MAGNITUDE: f32 = 1.1;

pub fn input_processing_system(
    mut input_events: EventReader<InputCommandEvent>,
    mut query: Query<(&Player, &mut DesiredVelocity, &CharacterProfile)>,
) {
    for event in input_events.read() {
        for (player, mut desired_velocity, profile) in query.iter_mut() {
            if player.id == event.player_id {
                match &event.command {
                    InputCommand::Move { direction } => {
                        let normalized_direction = direction.normalize_or_zero();
                        desired_velocity.x = normalized_direction.x * profile.max_speed;
                        desired_velocity.y = normalized_direction.y * profile.max_speed;
                    }
                    InputCommand::Stop => {
                        desired_velocity.x = 0.0;
                        desired_velocity.y = 0.0;
                    }
                }
                break;
            }
        }
    }
}

// ============================================================================
// MOVEMENT SYSTEMS
// ============================================================================

const MIN_VELOCITY_THRESHOLD: f32 = 0.01;
const MIN_CHANGE_THRESHOLD: f32 = 0.01;
const WORLD_MIN_X: f32 = 0.0;
const WORLD_MIN_Y: f32 = 0.0;

pub fn acceleration_friction_system(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &DesiredVelocity, &CharacterProfile, &Friction)>,
) {
    let dt = time.delta_secs();
    
    for (mut velocity, desired_velocity, profile, friction) in query.iter_mut() {
        let is_trying_to_move = desired_velocity.x.abs() > MIN_VELOCITY_THRESHOLD 
                             || desired_velocity.y.abs() > MIN_VELOCITY_THRESHOLD;
        
        let current_speed = velocity.x.abs() + velocity.y.abs();
        
        // Skip processing if already at zero velocity and not trying to move
        if !is_trying_to_move && current_speed < MIN_VELOCITY_THRESHOLD {
            continue;
        }
        
        if is_trying_to_move {
            // Simple linear interpolation towards desired velocity
            let lerp_factor = (profile.acceleration * dt).min(1.0);
            velocity.x += (desired_velocity.x - velocity.x) * lerp_factor;
            velocity.y += (desired_velocity.y - velocity.y) * lerp_factor;
        } else {
            // Simple friction decay
            let friction_factor = 1.0 - (friction.coefficient * dt).min(1.0);
            velocity.x *= friction_factor;
            velocity.y *= friction_factor;
            
            // Stop tiny movements
            if velocity.x.abs() < MIN_VELOCITY_THRESHOLD {
                velocity.x = 0.0;
            }
            if velocity.y.abs() < MIN_VELOCITY_THRESHOLD {
                velocity.y = 0.0;
            }
        }
        
        // Simple max speed clamp using Manhattan distance approximation
        let speed_approx = velocity.x.abs() + velocity.y.abs();
        if speed_approx > profile.max_speed * 1.4 { // 1.4 ≈ sqrt(2) for diagonal movement
            let scale = profile.max_speed * 1.4 / speed_approx;
            velocity.x *= scale;
            velocity.y *= scale;
        }
    }
}

pub fn movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Position, &Velocity)>,
) {
    let dt = time.delta_secs();
    
    for (mut position, velocity) in query.iter_mut() {
        // Skip position updates if velocity is effectively zero
        if velocity.x.abs() < MIN_VELOCITY_THRESHOLD && velocity.y.abs() < MIN_VELOCITY_THRESHOLD {
            continue;
        }
        
        position.x += velocity.x * dt;
        position.y += velocity.y * dt;
    }
}

pub fn boundary_system(
    mut query: Query<(&mut Position, &mut Velocity), Changed<Position>>,
    config: Res<GameConfig>,
) {
    for (mut position, mut velocity) in query.iter_mut() {
        if position.x < WORLD_MIN_X {
            position.x = WORLD_MIN_X;
            velocity.x = -velocity.x;
        }
        
        if position.x > config.world_bounds.x {
            position.x = config.world_bounds.x;
            velocity.x = -velocity.x;
        }
        
        if position.y < WORLD_MIN_Y {
            position.y = WORLD_MIN_Y;
            velocity.y = -velocity.y;
        }
        
        if position.y > config.world_bounds.y {
            position.y = config.world_bounds.y;
            velocity.y = -velocity.y;
        }
    }
}


// ============================================================================
// PLAYER MANAGEMENT SYSTEMS
// ============================================================================

pub fn player_spawn_system(
    mut commands: Commands,
    mut spawn_events: EventReader<PlayerSpawnEvent>,
    mut player_registry: ResMut<PlayerRegistry>,
    mut allocator: ResMut<crate::ecs::plugins::network::components::NetworkIdAllocator>,
    game_config: Res<GameConfig>,
    connections: Res<crate::ecs::plugins::websocket::components::WebSocketConnections>,
) {
    for event in spawn_events.read() {
        println!("🎮 Spawning player {}", event.player_id);
        
        // Spawn player entity with networking
        let network_id = allocator.allocate();
        let player_entity = commands.spawn((
            PlayerBundle::new(event.player_id, &game_config),
            crate::ecs::plugins::network::components::NetworkedEntityBundle::new(network_id),
        )).id();
        
        // Register player
        player_registry.register_player(event.player_id, player_entity);
        
        // Send welcome message with both player_id and network_id
        let welcome_msg = crate::ecs::plugins::network::components::NetworkMessage {
            message_type: crate::ecs::plugins::network::components::WELCOME_TYPE.to_string(),
            entity_updates: vec![crate::ecs::plugins::network::components::EntityUpdate {
                network_id: event.player_id, // Use player_id as the identifier
                components: {
                    let mut components = std::collections::HashMap::new();
                    components.insert("player_id".to_string(), serde_json::Value::Number(serde_json::Number::from(event.player_id)));
                    components.insert("network_id".to_string(), serde_json::Value::Number(serde_json::Number::from(network_id)));
                    components
                },
            }],
        };
        
        let _ = connections.player_network_sender.send((event.player_id, welcome_msg));
        
        println!("✅ Player {} spawned with network ID {}", event.player_id, network_id);
    }
}

pub fn player_despawn_system(
    mut commands: Commands,
    mut despawn_events: EventReader<PlayerDespawnEvent>,
    mut player_registry: ResMut<PlayerRegistry>,
) {
    for event in despawn_events.read() {
        println!("👋 Despawning player {}", event.player_id);
        
        // Despawn player entity
        if let Some(entity) = player_registry.unregister_player(event.player_id) {
            commands.entity(entity).despawn();
        }
    }
}

pub fn character_spawn_system(
    mut commands: Commands,
    mut spawn_events: EventReader<CharacterSpawnEvent>,
    game_config: Res<GameConfig>,
) {
    for event in spawn_events.read() {
        println!("🤖 Spawning character {}", event.character_id);
        
        // Spawn character entity (no networking for bots/NPCs)
        let _character_entity = commands.spawn(
            CharacterBundle::new(event.character_id, event.position, &game_config)
        ).id();
        
        println!("✅ Character {} spawned", event.character_id);
    }
}

pub fn character_despawn_system(
    mut commands: Commands,
    mut despawn_events: EventReader<CharacterDespawnEvent>,
    character_query: Query<(Entity, &Character)>,
) {
    for event in despawn_events.read() {
        println!("👋 Despawning character {}", event.character_id);
        
        // Find and despawn character entity
        for (entity, character) in character_query.iter() {
            if character.id == event.character_id {
                commands.entity(entity).despawn();
                break;
            }
        }
    }
}


