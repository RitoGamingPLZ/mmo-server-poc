use bevy::prelude::*;
use std::collections::HashMap;
use crate::ecs::components::{Position, Velocity, Player, ViewDistance};
use super::components::*;

// ============================================================================
// NETWORK SYSTEMS
// ============================================================================

pub fn detect_velocity_changes_system(
    mut query: Query<(&mut NetworkDirty, &mut NetworkSnapshot, &Velocity), 
                    (With<NetworkId>, Changed<Velocity>)>,
) {
    for (mut dirty, mut snapshot, velocity) in query.iter_mut() {
        // Use compact format: [x, y] instead of {"x": x, "y": y}
        let compact_velocity = vec![velocity.x, velocity.y];
        let current_value = serde_json::to_value(compact_velocity).unwrap();
        snapshot.components.insert(super::components::VELOCITY_KEY.to_string(), current_value);
        
        if !dirty.changed_components.contains(&super::components::VELOCITY_KEY.to_string()) {
            dirty.changed_components.push(super::components::VELOCITY_KEY.to_string());
        }
    }
}

pub fn detect_position_changes_system(
    mut query: Query<(&mut NetworkDirty, &mut NetworkSnapshot, &Position), 
                    (With<NetworkId>, Changed<Position>)>,
) {
    for (mut dirty, mut snapshot, position) in query.iter_mut() {
        // Use compact format: [x, y] instead of {"x": x, "y": y}
        let compact_position = vec![position.x, position.y];
        let current_value = serde_json::to_value(compact_position).unwrap();
        snapshot.components.insert(super::components::POSITION_KEY.to_string(), current_value);
        
        if !dirty.changed_components.contains(&super::components::POSITION_KEY.to_string()) {
            dirty.changed_components.push(super::components::POSITION_KEY.to_string());
        }
    }
}

pub fn build_delta_updates_system(
    mut network_updates: ResMut<NetworkUpdates>,
    mut dirty_query: Query<(&NetworkId, &mut NetworkDirty, &NetworkSnapshot, &Position)>,
    player_query: Query<(&Player, &Position, &ViewDistance)>,
) {
    // Build updates for each player based on their view radius
    for (player, player_pos, view_distance) in player_query.iter() {
        let mut entity_updates = Vec::new();
        
        for (network_id, mut dirty, snapshot, entity_pos) in dirty_query.iter_mut() {
            if dirty.changed_components.is_empty() {
                continue;
            }
            
            // Calculate distance between player and entity (using fast approximation)
            let dx = player_pos.x - entity_pos.x;
            let dy = player_pos.y - entity_pos.y;
            let distance_approx = dx.abs() + dy.abs(); // Manhattan distance
            
            // Only include entities within view radius (adjust for Manhattan distance)
            if distance_approx <= view_distance.radius * 1.4 {
                let mut components = HashMap::new();
                
                for component_name in &dirty.changed_components {
                    if let Some(value) = snapshot.components.get(component_name) {
                        components.insert(component_name.clone(), value.clone());
                    }
                }
                
                if !components.is_empty() {
                    entity_updates.push(EntityUpdate {
                        network_id: network_id.0,
                        components,
                    });
                }
            }
        }
        
        if !entity_updates.is_empty() {
            let message = NetworkMessage {
                message_type: super::components::DELTA_UPDATE_TYPE.to_string(),
                entity_updates,
            };
            network_updates.player_messages.entry(player.id).or_insert_with(Vec::new).push(message);
        }
    }
    
    // Clear dirty components after processing
    for (_, mut dirty, _, _) in dirty_query.iter_mut() {
        dirty.changed_components.clear();
    }
}

pub fn build_full_sync_system(
    mut network_updates: ResMut<NetworkUpdates>,
    networked_query: Query<(&NetworkId, &NetworkSnapshot, &Position)>,
    mut player_spawn_events: EventReader<crate::ecs::components::PlayerSpawnEvent>,
    player_query: Query<(&Player, &Position, &ViewDistance)>,
) {
    // Get joining players
    let joining_players: Vec<u32> = player_spawn_events.read().map(|event| event.player_id).collect();
    
    if joining_players.is_empty() {
        return;
    }
    
    // Send full sync to each joining player based on their view radius
    for joining_player_id in joining_players {
        // Find the joining player's position and view distance
        if let Some((_, player_pos, view_distance)) = player_query.iter()
            .find(|(player, _, _)| player.id == joining_player_id) {
            
            let mut entity_updates = Vec::new();
            
            // Send full state of entities within view radius
            for (network_id, snapshot, entity_pos) in networked_query.iter() {
                if !snapshot.components.is_empty() {
                    // Calculate distance between joining player and entity
                    let dx = player_pos.x - entity_pos.x;
                    let dy = player_pos.y - entity_pos.y;
                    let distance_approx = dx.abs() + dy.abs(); // Manhattan distance
                    
                    // Only include entities within view radius
                    if distance_approx <= view_distance.radius * 1.4 {
                        entity_updates.push(EntityUpdate {
                            network_id: network_id.0,
                            components: snapshot.components.clone(),
                        });
                    }
                }
            }
            
            if !entity_updates.is_empty() {
                println!("🔄 Full sync triggered for player {} with {} entities", joining_player_id, entity_updates.len());
                let message = NetworkMessage {
                    message_type: super::components::FULL_SYNC_TYPE.to_string(),
                    entity_updates,
                };
                network_updates.player_messages.entry(joining_player_id).or_insert_with(Vec::new).push(message);
            }
        }
    }
}