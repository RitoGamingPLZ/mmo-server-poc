use bevy::prelude::*;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tokio::net::{TcpListener, TcpStream};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crossbeam_channel::Sender;
use std::thread;
use crate::ecs::components::*;
use crate::ecs::plugins::network::components::NetworkUpdates;
use super::components::*;

// Setup WebSocket server in dedicated async runtime
pub fn setup_websocket_server(
    connections: Res<WebSocketConnections>,
    port: u16,
) {
    let connections_clone = connections.connections.clone();
    let message_sender = connections.outgoing_sender.clone();
    let network_receiver = connections.network_receiver.clone();
    let player_network_receiver = connections.player_network_receiver.clone();
    
    // Spawn a dedicated thread for the WebSocket server
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2) // Use # of CPU cores (Pi 4 = 4, Pi 5 = 4/8)
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
            println!("🌐 WebSocket server listening on ws://localhost:{}", port);
            
            // Spawn task to handle outgoing network messages (global)
            let connections_for_sender = connections_clone.clone();
            tokio::spawn(async move {
                loop {
                    // Use try_recv to avoid blocking the async runtime
                    match network_receiver.try_recv() {
                        Ok(network_msg) => {
                            let json_msg = serde_json::to_string(&network_msg).unwrap_or_default();
                            let ws_message = Message::Text(json_msg.into());
                            
                            // Send to all connected clients
                            let conns = connections_for_sender.lock().await;
                            for (_, sender) in conns.iter() {
                                let _ = sender.send(ws_message.clone());
                            }
                        }
                        Err(_) => {
                            // No messages available, sleep briefly to avoid busy waiting
                            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        }
                    }
                }
            });
            
            // Spawn task to handle player-specific network messages
            let connections_for_player_sender = connections_clone.clone();
            tokio::spawn(async move {
                loop {
                    // Use try_recv to avoid blocking the async runtime
                    match player_network_receiver.try_recv() {
                        Ok((player_id, network_msg)) => {
                            let json_msg = serde_json::to_string(&network_msg).unwrap_or_default();
                            let ws_message = Message::Text(json_msg.into());
                            
                            // Send to specific player
                            let conns = connections_for_player_sender.lock().await;
                            if let Some(sender) = conns.get(&player_id) {
                                let _ = sender.send(ws_message);
                            }
                        }
                        Err(_) => {
                            // No messages available, sleep briefly to avoid busy waiting
                            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        }
                    }
                }
            });
            
            // Accept incoming connections
            while let Ok((stream, addr)) = listener.accept().await {
                println!("📡 New connection from: {}", addr);
                let connections = connections_clone.clone();
                let sender = message_sender.clone();
                tokio::spawn(handle_client(stream, connections, sender));
            }
        });
    });
}

// Handle individual WebSocket client
async fn handle_client(
    stream: TcpStream,
    connections: Arc<Mutex<HashMap<u32, tokio::sync::mpsc::UnboundedSender<Message>>>>,
    message_sender: Sender<WebSocketMessage>,
) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            println!("❌ WebSocket handshake failed: {}", e);
            return;
        }
    };
    
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    
    // Generate player ID (range: 1-9999 to avoid overlap with network IDs 10000+)
    let player_id = {
        let mut conns = connections.lock().await;
        let mut player_id = 1;
        while conns.contains_key(&player_id) {
            player_id += 1;
            if player_id >= 10000 {
                // Wrap around if we somehow exceed the range
                player_id = 1;
                break;
            }
        }
        conns.insert(player_id, tx.clone());
        player_id
    };
    
    println!("✅ Player {} connected", player_id);
    
    // Notify ECS that player joined
    let _ = message_sender.send(WebSocketMessage::PlayerJoined(player_id));
    
    // Spawn task to handle outgoing messages
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });
    
    // Handle incoming messages
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(input) = serde_json::from_str::<serde_json::Value>(&text) {
                    handle_input_message(player_id, input, &message_sender).await;
                }
            }
            Ok(Message::Close(_)) => {
                println!("🔌 Player {} disconnected", player_id);
                break;
            }
            Ok(Message::Ping(data)) => {
                let _ = tx_clone.send(Message::Pong(data));
            }
            _ => {}
        }
    }
    
    // Clean up connection
    connections.lock().await.remove(&player_id);
    let _ = message_sender.send(WebSocketMessage::PlayerLeft(player_id));
    println!("🧹 Cleaned up connection for player {}", player_id);
}

// Handle input messages from WebSocket
async fn handle_input_message(player_id: u32, input: serde_json::Value, message_sender: &Sender<WebSocketMessage>) {
    // Parse input and send to ECS
    match serde_json::from_value::<InputCommand>(input.clone()) {
        Ok(command) => {
            let _ = message_sender.send(WebSocketMessage::PlayerInput(player_id, command));
        }
        Err(_) => {
            println!("📥 Player {} sent invalid input: {:?}", player_id, input);
        }
    }
}

// System to handle WebSocket messages and convert to ECS events
pub fn handle_websocket_messages(
    mut input_events: EventWriter<InputCommandEvent>,
    mut spawn_events: EventWriter<PlayerSpawnEvent>,
    mut despawn_events: EventWriter<PlayerDespawnEvent>,
    connections: Res<WebSocketConnections>,
) {
    // Process all incoming messages from WebSocket
    while let Ok(message) = connections.incoming_messages.try_recv() {
        match message {
            WebSocketMessage::PlayerJoined(player_id) => {
                println!("🌐 WebSocket: Player {} connected", player_id);
                // Just send the spawn event - let other systems handle spawning
                spawn_events.send(PlayerSpawnEvent { player_id });
            }
            WebSocketMessage::PlayerLeft(player_id) => {
                println!("🌐 WebSocket: Player {} disconnected", player_id);
                // Just send the despawn event - let other systems handle despawning
                despawn_events.send(PlayerDespawnEvent { player_id });
            }
            WebSocketMessage::PlayerInput(player_id, command) => {
                // Send input event
                input_events.send(InputCommandEvent { player_id, command });
            }
        }
    }
}

// System to send network updates via WebSocket
pub fn send_network_updates(
    mut network_updates: ResMut<NetworkUpdates>,
    connections: Res<WebSocketConnections>,
) {
    // Send global messages to all players (if any)
    if !network_updates.messages.is_empty() {
        for message in network_updates.messages.drain(..) {
            let _ = connections.network_sender.send(message);
        }
    }
    
    // Send player-specific messages
    if !network_updates.player_messages.is_empty() {
        for (player_id, messages) in network_updates.player_messages.drain() {
            for message in messages {
                let _ = connections.player_network_sender.send((player_id, message));
            }
        }
    }
}