use bevy::prelude::*;
use std::net::SocketAddr;
use crossbeam_channel::{Sender, Receiver};
use std::collections::HashMap;

// WebSocket-specific components
#[derive(Debug, Clone)]
pub enum WsEvent {
    Connected(SocketAddr),
    Disconnected(SocketAddr),
    Message { client: SocketAddr, data: Vec<u8> },
    TextMessage { client: SocketAddr, text: String },
    SendMessage { client: SocketAddr, message: String },
    Broadcast { client: SocketAddr, message: String },
}

#[derive(Resource)]
pub struct WsSendChannel(pub Sender<WsEvent>);

#[derive(Resource)]
pub struct WsRecvChannel(pub Receiver<WsEvent>);

// Client ID type
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ClientId {
    WebSocket(SocketAddr),
}

// Client info
#[derive(Clone, Debug)]
pub struct ClientInfo {
    pub connected_at: std::time::Instant,
}

impl ClientInfo {
    pub fn new(_client_id: ClientId) -> Self {
        Self {
            connected_at: std::time::Instant::now(),
        }
    }
}

// Connected clients resource
#[derive(Resource, Default)]
pub struct ConnectedClients {
    pub clients: HashMap<ClientId, ClientInfo>,
}

// Network player registry
#[derive(Resource, Default)]
pub struct NetworkPlayerRegistry {
    client_to_player: HashMap<ClientId, u32>,
    player_to_client: HashMap<u32, ClientId>,
}

impl NetworkPlayerRegistry {
    pub fn register_player(&mut self, client_id: ClientId, player_id: u32) {
        self.client_to_player.insert(client_id.clone(), player_id);
        self.player_to_client.insert(player_id, client_id);
    }
    
    pub fn unregister_player(&mut self, client_id: &ClientId) -> Option<u32> {
        if let Some(player_id) = self.client_to_player.remove(client_id) {
            self.player_to_client.remove(&player_id);
            Some(player_id)
        } else {
            None
        }
    }
    
    pub fn get_player_id(&self, client_id: &ClientId) -> Option<u32> {
        self.client_to_player.get(client_id).copied()
    }
}

// Generate unique player IDs
pub fn generate_player_id() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

// Events
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