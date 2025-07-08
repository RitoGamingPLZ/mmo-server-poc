use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// CORE NETWORKING COMPONENTS
// ============================================================================

/// Marks an entity as networked with a unique ID
#[derive(Component)]
pub struct NetworkId(pub u32);

/// Generic snapshot storage for any networked component
#[derive(Component, Default)]
pub struct NetworkSnapshot {
    pub components: HashMap<String, serde_json::Value>,
}

/// Tracks which components have changed since last sync
#[derive(Component, Default)]
pub struct NetworkDirty {
    pub changed_components: Vec<String>,
}

// ============================================================================
// TRAIT FOR NETWORKED COMPONENTS
// ============================================================================

/// Trait that networked components must implement
pub trait NetworkedComponent: Component + Serialize + Clone + PartialEq {
    fn component_name() -> &'static str;
    
    fn to_network_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

// ============================================================================
// RESOURCES
// ============================================================================

#[derive(Resource, Default)]
pub struct NetworkIdAllocator {
    next_id: u32,
}

impl NetworkIdAllocator {
    pub fn allocate(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }
}

#[derive(Resource, Default)]
pub struct NetworkUpdates {
    pub messages: Vec<NetworkMessage>,
}

// ============================================================================
// NETWORK MESSAGE TYPES
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub message_type: String,
    pub entity_updates: Vec<EntityUpdate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityUpdate {
    pub network_id: u32,
    pub components: HashMap<String, serde_json::Value>,
}

// ============================================================================
// BUNDLE FOR EASY ENTITY SPAWNING
// ============================================================================

#[derive(Bundle)]
pub struct NetworkedEntityBundle {
    pub network_id: NetworkId,
    pub snapshot: NetworkSnapshot,
    pub dirty: NetworkDirty,
}

impl NetworkedEntityBundle {
    pub fn new(network_id: u32) -> Self {
        Self {
            network_id: NetworkId(network_id),
            snapshot: NetworkSnapshot::default(),
            dirty: NetworkDirty::default(),
        }
    }
}