use bevy::prelude::*;
use tokio_tungstenite::tungstenite::Message;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crossbeam_channel::{Receiver, Sender};
use crate::ecs::components::*;
use crate::ecs::plugins::network::components::NetworkMessage;

// Messages from WebSocket to ECS
#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    PlayerJoined(u32),
    PlayerLeft(u32),
    PlayerInput(u32, InputCommand),
}

// WebSocket connection resource
#[derive(Resource)]
pub struct WebSocketConnections {
    pub connections: Arc<Mutex<HashMap<u32, tokio::sync::mpsc::UnboundedSender<Message>>>>,
    pub incoming_messages: Receiver<WebSocketMessage>,
    pub outgoing_sender: Sender<WebSocketMessage>,
    pub network_sender: Sender<NetworkMessage>,
    pub network_receiver: Receiver<NetworkMessage>,
    pub player_network_sender: Sender<(u32, NetworkMessage)>,
    pub player_network_receiver: Receiver<(u32, NetworkMessage)>,
}

impl Default for WebSocketConnections {
    fn default() -> Self {
        let (ws_tx, ws_rx) = crossbeam_channel::unbounded();
        let (net_tx, net_rx) = crossbeam_channel::unbounded();
        let (player_net_tx, player_net_rx) = crossbeam_channel::unbounded();
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            incoming_messages: ws_rx,
            outgoing_sender: ws_tx,
            network_sender: net_tx,
            network_receiver: net_rx,
            player_network_sender: player_net_tx,
            player_network_receiver: player_net_rx,
        }
    }
}