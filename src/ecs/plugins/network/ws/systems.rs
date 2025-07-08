/*!
# WebSocket Network Systems

Real-time networking systems for WebSocket client connections.

These systems handle:
- WebSocket server management and client connections
- Input message processing with heartbeat monitoring
- Network message broadcasting to clients
*/

use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use tokio_tungstenite::tungstenite::Message;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, WebSocketStream};
use futures_util::{StreamExt, SinkExt, stream::SplitSink};
use crossbeam_channel::{Sender, Receiver};
use std::net::SocketAddr;
use std::collections::HashMap;
use tokio::net::TcpStream;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ecs::plugins::input::{InputCommand, InputCommandEvent};
use crate::ecs::plugins::player::{PlayerSpawnEvent, PlayerDespawnEvent};
use crate::ecs::plugins::network::ws::components::*;
use crate::ecs::plugins::network::{NetworkUpdates, NetworkId, NetworkSnapshot, EntityUpdate, NetworkMessage};

pub async fn ws_server_task(ws_send: Sender<WsEvent>) {
    let host = std::env::var("WEBSOCKET_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("WEBSOCKET_PORT").unwrap_or_else(|_| "5000".to_string());
    let addr = format!("{}:{}", host, port);
    
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("WS server running on ws://{}", addr);

    // Shared map of client connections for sending messages
    let connections: Arc<Mutex<HashMap<SocketAddr, SplitSink<WebSocketStream<TcpStream>, Message>>>> = 
        Arc::new(Mutex::new(HashMap::new()));

    // Channel for receiving outbound messages
    let (outbound_send, outbound_recv): (Sender<WsEvent>, Receiver<WsEvent>) = crossbeam_channel::unbounded();
    
    // Spawn task to handle outbound messages
    let connections_clone = connections.clone();
    tokio::spawn(async move {
        while let Ok(event) = outbound_recv.recv() {
            match event {
                WsEvent::SendMessage { client, message } => {
                    let mut conns = connections_clone.lock().await;
                    if let Some(sink) = conns.get_mut(&client) {
                        if let Err(_) = sink.send(Message::Text(message.into())).await {
                            // Remove failed connection
                            conns.remove(&client);
                            println!("Removed failed WebSocket connection: {:?}", client);
                        }
                    }
                }
                _ => {}
            }
        }
    });
    
    // Store the outbound sender globally so poll_ws_messages can access it
    OUTBOUND_SENDER.lock().await.replace(outbound_send);

    while let Ok((stream, client_addr)) = listener.accept().await {
        let ws_send = ws_send.clone();
        let connections = connections.clone();
        let _ = ws_send.send(WsEvent::Connected(client_addr));
        
        tokio::spawn(async move {
            let ws_stream = accept_async(stream).await.unwrap();
            println!("New WS client: {:?}", client_addr);
            
            let (sink, mut stream) = ws_stream.split();
            
            // Store the sink for outbound messages
            connections.lock().await.insert(client_addr, sink);

            while let Some(msg) = stream.next().await {
                if let Ok(msg) = msg {
                    match msg {
                        Message::Text(text) => {
                            let _ = ws_send.send(WsEvent::TextMessage { client: client_addr, text: text.to_string() });
                        }
                        Message::Close(_) => break,
                        _ => {}
                    }
                } else {
                    break;
                }
            }
            
            // Clean up connection
            connections.lock().await.remove(&client_addr);
            println!("WS client disconnected: {:?}", client_addr);
            let _ = ws_send.send(WsEvent::Disconnected(client_addr));
        });
    }
}

// Global sender for outbound messages (lazy static would be better, but this works)
static OUTBOUND_SENDER: tokio::sync::Mutex<Option<Sender<WsEvent>>> = tokio::sync::Mutex::const_new(None);

// Helper function to send WebSocket message to a specific client
async fn send_ws_message(client: SocketAddr, message: String) {
    if let Some(sender) = OUTBOUND_SENDER.lock().await.as_ref() {
        let _ = sender.send(WsEvent::SendMessage { client, message });
    }
}

pub fn poll_ws_messages(
    recv: Res<WsRecvChannel>,
    mut connected_clients: ResMut<ConnectedClients>,
    mut player_registry: ResMut<NetworkPlayerRegistry>,
    mut connect_events: EventWriter<ClientConnectedEvent>,
    mut disconnect_events: EventWriter<ClientDisconnectedEvent>,
    mut input_events: EventWriter<InputCommandEvent>,
    mut spawn_events: EventWriter<PlayerSpawnEvent>,
    mut despawn_events: EventWriter<PlayerDespawnEvent>,
) {
    for event in recv.0.try_iter() {
        match event {
            WsEvent::SendMessage { .. } => {
                // This should not be received in this system - it's handled in the server
            }
            WsEvent::Connected(addr) => {
                let client_id = ClientId::WebSocket(addr);
                let player_id = generate_player_id();
                
                // Update connected clients
                let client_info = ClientInfo::new(client_id.clone());
                connected_clients.clients.insert(client_id.clone(), client_info);
                
                // Register player
                player_registry.register_player(client_id.clone(), player_id);
                
                // Send events
                connect_events.send(ClientConnectedEvent { 
                    client_id: client_id.clone(), 
                    player_id 
                });
                spawn_events.send(PlayerSpawnEvent { player_id });
                
                println!("WS Player {} connected from {:?}", player_id, addr);
            }
            WsEvent::Disconnected(addr) => {
                let client_id = ClientId::WebSocket(addr);
                
                if let Some(player_id) = player_registry.unregister_player(&client_id) {
                    connected_clients.clients.remove(&client_id);
                    
                    disconnect_events.send(ClientDisconnectedEvent { 
                        client_id: client_id.clone(), 
                        player_id,
                        reason: "WebSocket disconnected".to_string(),
                    });
                    despawn_events.send(PlayerDespawnEvent { player_id });
                    
                    println!("WS Player {} disconnected from {:?}", player_id, addr);
                }
            }
            WsEvent::TextMessage { client, text } => {
                let client_id = ClientId::WebSocket(client);
                
                if let Some(player_id) = player_registry.get_player_id(&client_id) {
                    println!("Input from WS player {}: {:?}", player_id, text);
                    // Try to parse as InputCommand
                    match serde_json::from_str::<InputCommand>(&text) {
                        Ok(command) => {
                            println!("Command {:?}", command);
                            input_events.send(InputCommandEvent {
                                player_id,
                                command,
                            });
                        }
                        Err(e) => {
                            println!("Error parsing JSON from player {}: '{}' - Error: {} - Expected format: {{\"Move\": {{\"direction\": [1.0, 0.0]}}}}", player_id, text, e);
                        }
                    }
                    // Silently ignore parse errors (or add else block to log)
                } else {
                    println!("Received text message from unregistered WS client: {:?}", client);
                }
            }
            WsEvent::Message { client, data } => {
                // Handle binary message (MessagePack, etc.)
                let client_id = ClientId::WebSocket(client);
                
                if let Some(player_id) = player_registry.get_player_id(&client_id) {
                    // Try to decode as text first (for JSON compatibility)
                    let data_len = data.len();
                    if let Ok(text) = String::from_utf8(data) {
                        // Try to parse as InputCommand
                        match serde_json::from_str::<InputCommand>(&text) {
                            Ok(command) => {
                                input_events.send(InputCommandEvent {
                                    player_id,
                                    command,
                                });
                            }
                            Err(e) => {
                                println!("Error parsing binary message from player {}: {:?} - Error: {}", player_id, text, e);
                            }
                        }
                    } else {
                        // Handle pure binary data (MessagePack, etc.) here if needed
                        println!("Received binary message from player {}: {} bytes", player_id, data_len);
                    }
                } else {
                    println!("Received binary message from unregistered WS client: {:?}", client);
                }
            }
            WsEvent::Broadcast { client: _, message: _ } => {
                // Handle broadcast messages if needed
            }
        }
    }
}

/// System: Send network updates to WebSocket clients
pub fn send_network_updates_to_clients_system(
    mut network_updates: ResMut<NetworkUpdates>,
    connected_clients: Res<ConnectedClients>,
    player_registry: Res<NetworkPlayerRegistry>,
) {
    if network_updates.messages.is_empty() {
        return;
    }

    for message in &network_updates.messages {
        // println!("Broadcasting {} with {} entities to {} clients", 
        //     message.message_type, 
        //     message.entity_updates.len(),
        //     connected_clients.clients.len()
        // );
        
        // Convert to JSON
        let json_message = serde_json::to_string(message).unwrap_or_else(|e| {
            println!("Failed to serialize message: {}", e);
            return "{}".to_string();
        });
        
        // Send to all connected clients
        for (client_id, _client_info) in &connected_clients.clients {
            if let Some(_player_id) = player_registry.get_player_id(client_id) {
                send_message_to_client(client_id, &json_message);
            }
        }
    }
    
    // Clear sent messages
    network_updates.messages.clear();
}


/// System: Send full sync to newly connected players
pub fn send_full_sync_to_new_players_system(
    mut connect_events: EventReader<ClientConnectedEvent>,
    networked_query: Query<(&NetworkId, &NetworkSnapshot)>,
    _player_registry: Res<NetworkPlayerRegistry>,
) {
    for event in connect_events.read() {
        println!("🔄 Sending full sync to new player {}", event.player_id);
        
        // Build full sync message for all existing entities
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
            let full_sync_message = NetworkMessage {
                message_type: "full_sync".to_string(),
                entity_updates,
            };
            
            let json_message = serde_json::to_string(&full_sync_message).unwrap_or_else(|e| {
                println!("Failed to serialize full sync message: {}", e);
                return "{}".to_string();
            });
            
            println!("📤 Sending full sync with {} entities to player {}", 
                full_sync_message.entity_updates.len(), event.player_id);
                
            send_message_to_client(&event.client_id, &json_message);
        }
    }
}

/// System: Notify all players when someone disconnects by sending entity removal
pub fn notify_player_disconnect_system(
    mut disconnect_events: EventReader<ClientDisconnectedEvent>,
    connected_clients: Res<ConnectedClients>,
    player_registry: Res<NetworkPlayerRegistry>,
    main_player_registry: Res<crate::ecs::plugins::player::components::PlayerRegistry>,
    networked_query: Query<&NetworkId, With<crate::ecs::plugins::player::Player>>,
) {
    for event in disconnect_events.read() {
        println!("📤 Notifying all players that player {} disconnected", event.player_id);
        
        // Find the network_id of the disconnected player's entity
        if let Some(player_entity) = main_player_registry.get_player_entity(event.player_id) {
            if let Ok(network_id) = networked_query.get(player_entity) {
                // Create an entity removal message
                let disconnect_message = serde_json::json!({
                    "message_type": "entity_removed",
                    "network_id": network_id.0,
                    "player_id": event.player_id,
                    "reason": event.reason
                });
                
                let json_message = serde_json::to_string(&disconnect_message).unwrap_or_else(|e| {
                    println!("Failed to serialize disconnect message: {}", e);
                    return "{}".to_string();
                });
                
                // Send to all remaining connected clients
                for (client_id, _client_info) in &connected_clients.clients {
                    if client_id != &event.client_id {  // Don't send to the disconnected client
                        if let Some(_remaining_player_id) = player_registry.get_player_id(client_id) {
                            send_message_to_client(client_id, &json_message);
                        }
                    }
                }
                
                println!("📤 Sent entity removal for network_id {} (player {}) to {} clients", 
                    network_id.0, event.player_id, connected_clients.clients.len() - 1);
            }
        }
    }
}

/// Helper function to send a message to a specific client
fn send_message_to_client(client_id: &ClientId, message: &str) {
    let ClientId::WebSocket(addr) = client_id;
    let message = message.to_string();
    let client_addr = *addr;
    
    // Use Bevy's async task system to send the message
    IoTaskPool::get().spawn(async move {
        send_ws_message(client_addr, message).await;
    }).detach();
}


