use bevy::prelude::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU32, Ordering};

// Client identification - can be either u64 (UDP) or SocketAddr (WS)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClientId {
    Udp(u64),
    WebSocket(SocketAddr),
}

// Remove the to_player_id method - we'll generate player IDs uniformly

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub id: ClientId,
    pub connected_at: std::time::Instant,
    pub last_sync: std::time::Instant,
    pub last_heartbeat: std::time::Instant,
    pub requires_full_sync: bool,
}

impl ClientInfo {
    pub fn new(id: ClientId) -> Self {
        let now = std::time::Instant::now();
        Self {
            id,
            connected_at: now,
            last_sync: now,
            last_heartbeat: now,
            requires_full_sync: true, // New clients need full sync
        }
    }
    
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = std::time::Instant::now();
    }
    
    pub fn update_sync(&mut self) {
        self.last_sync = std::time::Instant::now();
        self.requires_full_sync = false;
    }
    
    pub fn is_timed_out(&self, timeout_duration: std::time::Duration) -> bool {
        self.last_heartbeat.elapsed() > timeout_duration
    }
    
    pub fn needs_full_sync_after_reconnect(&self, reconnect_threshold: std::time::Duration) -> bool {
        self.requires_full_sync || self.last_sync.elapsed() > reconnect_threshold
    }
}

// Unified events for both UDP and WS
#[derive(Event)]
pub struct ClientConnectedEvent {
    pub client_id: ClientId,
    pub player_id: u32,
}

#[derive(Event)]
pub struct ClientDisconnectedEvent {
    pub client_id: ClientId,
    pub player_id: u32,
    pub reason: String,
}

#[derive(Event)]
pub struct HeartbeatEvent {
    pub client_id: ClientId,
    pub timestamp: std::time::Instant,
}

#[derive(Event)]
pub struct ClientTimeoutEvent {
    pub client_id: ClientId,
    pub player_id: u32,
}

// Shared connected clients resource
#[derive(Resource, Default)]
pub struct ConnectedClients {
    pub clients: HashMap<ClientId, ClientInfo>,
}

// Player registry for managing player IDs
#[derive(Resource, Default)]
pub struct NetworkPlayerRegistry {
    pub client_to_player: HashMap<ClientId, u32>,
    pub player_to_client: HashMap<u32, ClientId>,
    pub player_metadata: HashMap<u32, PlayerMetadata>,
}

#[derive(Debug, Clone)]
pub struct PlayerMetadata {
    pub player_id: u32,
    pub client_id: ClientId,
    pub connected_at: std::time::Instant,
}

impl NetworkPlayerRegistry {
    pub fn register_player(&mut self, client_id: ClientId, player_id: u32) {
        let now = std::time::Instant::now();
        
        self.client_to_player.insert(client_id.clone(), player_id);
        self.player_to_client.insert(player_id, client_id.clone());
        self.player_metadata.insert(player_id, PlayerMetadata {
            player_id,
            client_id,
            connected_at: now,
        });
    }
    
    pub fn unregister_player(&mut self, client_id: &ClientId) -> Option<u32> {
        if let Some(player_id) = self.client_to_player.remove(client_id) {
            self.player_to_client.remove(&player_id);
            self.player_metadata.remove(&player_id);
            Some(player_id)
        } else {
            None
        }
    }
    
    pub fn get_player_id(&self, client_id: &ClientId) -> Option<u32> {
        self.client_to_player.get(client_id).copied()
    }
    
    pub fn get_client_id(&self, player_id: u32) -> Option<&ClientId> {
        self.player_to_client.get(&player_id)
    }
}

static NEXT_PLAYER_ID: AtomicU32 = AtomicU32::new(1);

pub fn generate_player_id() -> u32 {
    NEXT_PLAYER_ID.fetch_add(1, Ordering::SeqCst)
}