use bevy::prelude::*;
use serde::{Serialize, Deserialize};

/// A component that marks an entity as networked and provides its network identity
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NetworkedObject {
    /// Unique network ID for this object (used for client synchronization)
    pub network_id: u32,
    /// Optional object type for client-side handling
    pub object_type: NetworkedObjectType,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NetworkedObjectType {
    Player,
    NPC,
    Projectile,
    Item,
    Environment,
    Custom(String),
}

impl NetworkedObject {
    pub fn new_player(player_id: u32) -> Self {
        Self {
            network_id: player_id,
            object_type: NetworkedObjectType::Player,
        }
    }
    
    pub fn new_npc(npc_id: u32) -> Self {
        Self {
            network_id: npc_id,
            object_type: NetworkedObjectType::NPC,
        }
    }
    
    pub fn new_with_type(network_id: u32, object_type: NetworkedObjectType) -> Self {
        Self {
            network_id,
            object_type,
        }
    }
}

impl Default for NetworkedObject {
    fn default() -> Self {
        Self {
            network_id: 0,
            object_type: NetworkedObjectType::Custom("Unknown".to_string()),
        }
    }
}

/// Resource to manage network ID allocation
#[derive(Resource, Default)]
pub struct NetworkIdAllocator {
    next_id: u32,
    reserved_player_range: (u32, u32), // (start, end) - reserved for players
}

impl NetworkIdAllocator {
    pub fn new() -> Self {
        Self {
            next_id: 10000, // Start after player range
            reserved_player_range: (1, 9999), // Players get IDs 1-9999
        }
    }
    
    /// Allocate a new network ID for non-player objects
    pub fn allocate_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    
    /// Check if an ID is in the player range
    pub fn is_player_id(&self, id: u32) -> bool {
        id >= self.reserved_player_range.0 && id <= self.reserved_player_range.1
    }
    
    /// Reserve a specific player ID (for player spawning)
    pub fn reserve_player_id(&self, player_id: u32) -> Result<u32, String> {
        if self.is_player_id(player_id) {
            Ok(player_id)
        } else {
            Err(format!("Player ID {} is outside valid range ({}-{})", 
                player_id, self.reserved_player_range.0, self.reserved_player_range.1))
        }
    }
}