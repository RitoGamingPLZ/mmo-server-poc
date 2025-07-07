use bevy::prelude::*;
use std::net::SocketAddr;
use crossbeam_channel::{Sender, Receiver};

// WebSocket-specific components
#[derive(Debug, Clone)]
pub enum WsEvent {
    Connected(SocketAddr),
    Disconnected(SocketAddr),
    Message { client: SocketAddr, data: Vec<u8> },
    TextMessage { client: SocketAddr, text: String },
}

#[derive(Resource)]
pub struct WsSendChannel(pub Sender<WsEvent>);

#[derive(Resource)]
pub struct WsRecvChannel(pub Receiver<WsEvent>);