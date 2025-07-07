use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

pub trait NetworkedState: Component + Clone + PartialEq + Serialize + for<'de> Deserialize<'de> {
    fn get_field_changes(&self, previous: Option<&Self>) -> Vec<FieldUpdate>;
    fn apply_field_update(&mut self, update: &FieldUpdate);
    fn get_component_name() -> &'static str;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldUpdate {
    pub field_name: String,
    pub value: serde_json::Value,
}

#[derive(Resource, Default)]
pub struct NetworkStateSnapshot {
    pub snapshots: HashMap<(Entity, &'static str), serde_json::Value>,
    pub snapshot_history: HashMap<u64, SnapshotFrame>, // timestamp -> full state snapshot
    pub next_snapshot_id: u64,
}

#[derive(Clone, Debug)]
pub struct SnapshotFrame {
    pub id: u64,
    pub timestamp: std::time::Instant,
    pub entity_states: HashMap<u32, EntitySnapshot>, // network_id -> entity state
}

#[derive(Clone, Debug)]
pub struct EntitySnapshot {
    pub network_id: u32,
    pub components: HashMap<String, serde_json::Value>, // component_name -> serialized state
}

impl NetworkStateSnapshot {
    pub fn create_snapshot_frame(&mut self) -> u64 {
        let snapshot_id = self.next_snapshot_id;
        self.next_snapshot_id += 1;
        
        let entity_states = HashMap::new();
        
        // TODO: Group current snapshots by network_id
        // This will be populated by the sync system when we have entity -> network_id mapping
        
        let frame = SnapshotFrame {
            id: snapshot_id,
            timestamp: std::time::Instant::now(),
            entity_states,
        };
        
        self.snapshot_history.insert(snapshot_id, frame);
        
        // Clean old snapshots (keep last 100)
        if self.snapshot_history.len() > 100 {
            let oldest_ids: Vec<u64> = self.snapshot_history.keys()
                .copied()
                .collect::<Vec<_>>()
                .into_iter()
                .take(self.snapshot_history.len() - 100)
                .collect();
            
            for id in oldest_ids {
                self.snapshot_history.remove(&id);
            }
        }
        
        snapshot_id
    }
    
    pub fn get_delta_since(&self, since_timestamp: std::time::Instant) -> Vec<EntityUpdate> {
        // Find the first snapshot after the given timestamp
        let mut relevant_frames: Vec<&SnapshotFrame> = self.snapshot_history.values()
            .filter(|frame| frame.timestamp > since_timestamp)
            .collect();
        
        relevant_frames.sort_by_key(|frame| frame.timestamp);
        
        // For now, return empty - this would be implemented to create delta updates
        // from the snapshot history
        Vec::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentUpdate {
    pub component_name: String,
    pub field_updates: Vec<FieldUpdate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityUpdate {
    pub entity_id: u32, // Using player ID instead of Entity for client compatibility
    pub components: Vec<ComponentUpdate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub message_type: String, // "full_sync" or "delta_update"
    pub entity_updates: Vec<EntityUpdate>,
    pub my_player_id: u32,
}

#[derive(Default)]
pub struct ChangeBuffer {
    pub entity_changes: std::collections::HashMap<u32, std::collections::HashMap<String, Vec<FieldUpdate>>>,
}

pub fn track_networked_component_changes<T: NetworkedState>(
    snapshot: &mut NetworkStateSnapshot,
    query: Query<(Entity, &T, &crate::ecs::plugins::network::NetworkedObject)>,
    change_buffer: &mut ChangeBuffer,
) {
    let component_name = T::get_component_name();
    
    for (entity, component, networked_obj) in query.iter() {
        let snapshot_key = (entity, component_name);
        let current_value = serde_json::to_value(component).unwrap();
        
        let field_updates = if let Some(previous_value) = snapshot.snapshots.get(&snapshot_key) {
            if let Ok(previous_component) = serde_json::from_value::<T>(previous_value.clone()) {
                component.get_field_changes(Some(&previous_component))
            } else {
                component.get_field_changes(None)
            }
        } else {
            component.get_field_changes(None)
        };
        
        if !field_updates.is_empty() {
            // Group changes by entity and component
            change_buffer.entity_changes
                .entry(networked_obj.network_id)
                .or_default()
                .entry(component_name.to_string())
                .or_default()
                .extend(field_updates);
            
            snapshot.snapshots.insert(snapshot_key, current_value);
        }
    }
}

pub fn build_batched_updates(change_buffer: &mut ChangeBuffer) -> Vec<EntityUpdate> {
    let mut entity_updates = Vec::new();
    
    for (entity_id, component_changes) in change_buffer.entity_changes.drain() {
        let mut components = Vec::new();
        
        for (component_name, field_updates) in component_changes {
            if !field_updates.is_empty() {
                components.push(ComponentUpdate {
                    component_name,
                    field_updates,
                });
            }
        }
        
        if !components.is_empty() {
            entity_updates.push(EntityUpdate {
                entity_id,
                components,
            });
        }
    }
    
    entity_updates
}

// Optimized message format with reduced JSON overhead
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompactFieldUpdate {
    pub f: String, // field name (shortened)
    pub v: serde_json::Value, // value
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompactComponentUpdate {
    pub c: String, // component name (shortened)
    pub u: Vec<CompactFieldUpdate>, // updates (shortened)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompactEntityUpdate {
    pub id: u32, // entity_id
    pub cs: Vec<CompactComponentUpdate>, // components (shortened)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompactNetworkMessage {
    pub t: String, // message_type (shortened)
    pub es: Vec<CompactEntityUpdate>, // entity_updates (shortened)
    pub p: u32, // my_player_id (shortened)
}

pub fn compress_message(msg: &NetworkMessage) -> CompactNetworkMessage {
    CompactNetworkMessage {
        t: msg.message_type.clone(),
        es: msg.entity_updates.iter().map(|e| CompactEntityUpdate {
            id: e.entity_id,
            cs: e.components.iter().map(|c| CompactComponentUpdate {
                c: c.component_name.clone(),
                u: c.field_updates.iter().map(|f| CompactFieldUpdate {
                    f: f.field_name.clone(),
                    v: f.value.clone(),
                }).collect(),
            }).collect(),
        }).collect(),
        p: msg.my_player_id,
    }
}

// Enhanced macro that implements both NetworkedState and auto-sync
#[macro_export]
macro_rules! impl_networked_state {
    ($networked_type:ty, $source_type:ty, $name:expr, {$($field:ident : $threshold:expr),*}) => {
        impl NetworkedState for $networked_type {
            fn get_field_changes(&self, previous: Option<&Self>) -> Vec<FieldUpdate> {
                let mut changes = Vec::new();
                
                if let Some(prev) = previous {
                    $(
                        if (self.$field - prev.$field).abs() > $threshold {
                            changes.push(FieldUpdate {
                                field_name: stringify!($field).to_string(),
                                value: serde_json::to_value(&self.$field).unwrap(),
                            });
                        }
                    )*
                } else {
                    // New entity - include all fields
                    $(
                        changes.push(FieldUpdate {
                            field_name: stringify!($field).to_string(),
                            value: serde_json::to_value(&self.$field).unwrap(),
                        });
                    )*
                }
                
                changes
            }
            
            fn apply_field_update(&mut self, update: &FieldUpdate) {
                match update.field_name.as_str() {
                    $(
                        stringify!($field) => {
                            if let Ok(value) = serde_json::from_value(update.value.clone()) {
                                self.$field = value;
                            }
                        }
                    )*
                    _ => {}
                }
            }
            
            fn get_component_name() -> &'static str {
                $name
            }
        }
        
        impl From<&$source_type> for $networked_type {
            fn from(source: &$source_type) -> Self {
                Self {
                    $($field: source.$field,)*
                }
            }
        }
    };
}

// Macro to register networked components for auto-sync
#[macro_export]
macro_rules! register_networked_sync {
    ($app:expr, {$($networked:ty : $source:ty),*}) => {
        $app$(
            .add_systems(bevy::prelude::FixedUpdate, 
                move |mut commands: bevy::prelude::Commands,
                      missing_query: bevy::prelude::Query<(bevy::prelude::Entity, &$source), 
                          (bevy::prelude::With<crate::ecs::plugins::network::NetworkedObject>, bevy::prelude::Without<$networked>)>,
                      mut update_query: bevy::prelude::Query<(&$source, &mut $networked), 
                          (bevy::prelude::With<crate::ecs::plugins::network::NetworkedObject>, bevy::prelude::Changed<$source>)>| {
                    
                    // Add networked component to entities that don't have it
                    for (entity, source) in missing_query.iter() {
                        commands.entity(entity).insert(<$networked>::from(source));
                    }
                    
                    // Update networked component when source changes
                    for (source, mut networked) in update_query.iter_mut() {
                        *networked = <$networked>::from(source);
                    }
                }
            )
        )*;
    };
}