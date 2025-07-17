use bevy::prelude::*;
use std::collections::HashMap;
use crate::ecs::plugins::network::components::*;
use crate::ecs::plugins::player::{PlayerSpawnEvent, components::PlayerRegistry};

/// System: Add networking components to existing player entities
pub fn add_networking_to_players_system(
    mut commands: Commands,
    mut allocator: ResMut<NetworkIdAllocator>,
    mut spawn_events: EventReader<PlayerSpawnEvent>,
    player_registry: Res<PlayerRegistry>,
    position_query: Query<&crate::ecs::plugins::transform::Position>,
) {
    for event in spawn_events.read() {
        println!("🔍 Looking for player entity for player {}", event.player_id);
        println!("📋 Player registry contains: {:?}", player_registry.players.keys().collect::<Vec<_>>());
        if let Some(player_entity) = player_registry.get_player_entity(event.player_id) {
            if let Ok(position) = position_query.get(player_entity) {
                let network_id = allocator.allocate();
                commands.entity(player_entity).insert(
                    NetworkedEntityBundle::new(network_id),
                );
                
                println!("✅ Added networking to player {} entity with network ID {} at ({}, {})", 
                    event.player_id, network_id, position.x, position.y);
            } else {
                println!("❌ Could not query position for player {}", event.player_id);
            }
        } else {
            println!("❌ Could not find player entity for player {}", event.player_id);
        }
    }
}

/// System: Detect velocity changes and update snapshots
pub fn detect_velocity_changes_system(
    mut query: Query<(&mut NetworkDirty, &mut NetworkSnapshot, &crate::ecs::plugins::movement::components::Velocity), 
                    (With<NetworkId>, Changed<crate::ecs::plugins::movement::components::Velocity>)>,
) {
    for (mut dirty, mut snapshot, velocity) in query.iter_mut() {
        let current_value = serde_json::to_value(velocity).unwrap();
        
        // Update snapshot
        snapshot.components.insert("velocity".to_string(), current_value);
        
        // Mark as dirty if not already there
        if !dirty.changed_components.contains(&"velocity".to_string()) {
            dirty.changed_components.push("velocity".to_string());
        }
    }
}

/// System: Detect position changes and update snapshots
pub fn detect_position_changes_system(
    mut query: Query<(&mut NetworkDirty, &mut NetworkSnapshot, &crate::ecs::plugins::transform::components::Position), 
                    (With<NetworkId>, Changed<crate::ecs::plugins::transform::components::Position>)>,
) {
    for (mut dirty, mut snapshot, position) in query.iter_mut() {
        let current_value = serde_json::to_value(position).unwrap();
        
        // Update snapshot
        snapshot.components.insert("position".to_string(), current_value);
        
        // Mark as dirty if not already there
        if !dirty.changed_components.contains(&"position".to_string()) {
            dirty.changed_components.push("position".to_string());
        }
    }
}

/// System: Build delta updates from dirty entities and clear dirty flags
pub fn build_delta_updates_system(
    mut network_updates: ResMut<NetworkUpdates>,
    mut dirty_query: Query<(&NetworkId, &mut NetworkDirty, &NetworkSnapshot)>,
) {
    let mut entity_updates = Vec::new();
    
    for (network_id, mut dirty, snapshot) in dirty_query.iter_mut() {
        if dirty.changed_components.is_empty() {
            continue;
        }
        
        let mut components = HashMap::new();
        
        // Only include changed components in the update
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
        
        // Clear dirty flags directly
        dirty.changed_components.clear();
    }
    
    if !entity_updates.is_empty() {
        // for update in &entity_updates {
        //     println!("Entity {}: {:?}", update.network_id, update.components);
        // }
        network_updates.messages.push(NetworkMessage {
            message_type: "delta_update".to_string(),
            entity_updates,
        });
    }
}

/// System: Build full sync updates
pub fn build_full_sync_system(
    mut network_updates: ResMut<NetworkUpdates>,
    networked_query: Query<(&NetworkId, &NetworkSnapshot)>,
    time: Res<Time>,
    mut last_full_sync: Local<f32>,
) {
    // Trigger full sync every 10 seconds for demo
    if time.elapsed_secs() - *last_full_sync < 10.0 {
        return;
    }
    *last_full_sync = time.elapsed_secs();
    
    let mut entity_updates = Vec::new();
    
    for (network_id, snapshot) in networked_query.iter() {
        if !snapshot.components.is_empty() {
            entity_updates.push(EntityUpdate {
                network_id: network_id.0,
                components: snapshot.components.clone(),
            });
        }
    }
    
    if !entity_updates.is_empty() {
        let num_entities = entity_updates.len();
        // println!("Building full sync for {} entities", num_entities);
        network_updates.messages.push(NetworkMessage {
            message_type: "full_sync".to_string(),
            entity_updates,
        });
    }
}