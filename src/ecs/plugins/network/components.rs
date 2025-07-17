use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// NETWORK COMPONENTS
// ============================================================================

#[derive(Component)]
pub struct NetworkId(pub u32);

#[derive(Component, Default)]
pub struct NetworkSnapshot {
    pub components: HashMap<String, serde_json::Value>,
}

#[derive(Component, Default)]
pub struct NetworkDirty {
    pub changed_components: Vec<String>,
}

#[derive(Component, Default)]
pub struct ViewRangeTracker {
    pub players_in_view: std::collections::HashSet<u32>,
}

#[derive(Bundle)]
pub struct NetworkedEntityBundle {
    pub network_id: NetworkId,
    pub snapshot: NetworkSnapshot,
    pub dirty: NetworkDirty,
    pub view_tracker: ViewRangeTracker,
}

impl NetworkedEntityBundle {
    pub fn new(network_id: u32) -> Self {
        Self {
            network_id: NetworkId(network_id),
            snapshot: NetworkSnapshot::default(),
            dirty: NetworkDirty::default(),
            view_tracker: ViewRangeTracker::default(),
        }
    }
}

// ============================================================================
// NETWORK RESOURCES
// ============================================================================

#[derive(Resource)]
pub struct NetworkIdAllocator {
    next_id: u32,
}

impl Default for NetworkIdAllocator {
    fn default() -> Self {
        Self {
            next_id: 10000, // Start network IDs at 10000 to avoid overlap with player IDs
        }
    }
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
    pub player_messages: HashMap<u32, Vec<NetworkMessage>>, // Per-player messages
}

// ============================================================================
// NETWORK MESSAGE TYPES
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkMessage {
    #[serde(rename = "t")]
    pub message_type: String,
    #[serde(rename = "u")]
    pub entity_updates: Vec<EntityUpdate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityUpdate {
    #[serde(rename = "i")]
    pub network_id: u32,
    #[serde(rename = "c")]
    pub components: HashMap<String, serde_json::Value>,
}

// Component name mappings for shorter keys
pub const POSITION_KEY: &str = "p";
pub const VELOCITY_KEY: &str = "v";

// Message type constants
pub const DELTA_UPDATE_TYPE: &str = "d";
pub const FULL_SYNC_TYPE: &str = "f";
pub const WELCOME_TYPE: &str = "w";