/*!
# WebSocket Network Systems

Real-time networking systems for WebSocket client connections.

These systems handle:
- WebSocket server management and client connections
- Input message processing with heartbeat monitoring
- Smart client-aware synchronization
- Automatic timeout detection and cleanup
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
use crate::ecs::plugins::network::components::*;
use crate::ecs::plugins::network::ws::components::*;
use crate::ecs::plugins::network::networked_state::*;
use crate::ecs::plugins::network::NetworkedObject;
use crate::ecs::plugins::network::component_registry::{
    NetworkedComponentRegistry, build_full_sync_updates_registry, build_delta_updates_registry
};

// Network Configuration Constants
/// How long since last sync to trigger a full sync (instead of delta) for reconnecting clients
const RECONNECT_THRESHOLD_SECONDS: u64 = 3;

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

pub fn batched_broadcast_system(
    mut snapshot: ResMut<NetworkStateSnapshot>,
    networked_query: Query<(Entity, &NetworkedObject)>,
    registry: Res<NetworkedComponentRegistry>,
    player_registry: Res<NetworkPlayerRegistry>,
    mut connected_clients: ResMut<ConnectedClients>,
    _ws_send: Res<WsSendChannel>,
    world: &World,
) {
    let reconnect_threshold = std::time::Duration::from_secs(RECONNECT_THRESHOLD_SECONDS);
    
    // Check if any clients need full sync
    let has_clients_needing_full_sync = connected_clients.clients.iter()
        .any(|(client_id, client_info)| {
            player_registry.get_player_id(client_id).is_some() && 
            client_info.needs_full_sync_after_reconnect(reconnect_threshold)
        });
    
    // Build full sync data if needed using the registry approach
    let full_sync_updates = if has_clients_needing_full_sync {
        Some(build_full_sync_updates_registry(&networked_query, world, &*registry))
    } else {
        None
    };
    
    // Build delta updates using the registry approach
    let entity_updates = build_delta_updates_registry(&networked_query, world, &*registry, &mut snapshot);
    
    // Process each client individually
    for (client_id, client_info) in connected_clients.clients.iter_mut() {
        if let Some(player_id) = player_registry.get_player_id(client_id) {
            let needs_full_sync = client_info.needs_full_sync_after_reconnect(reconnect_threshold);
            
            // Determine what to send
            let message_params = if needs_full_sync {
                full_sync_updates.as_ref().map(|data| (data.clone(), "full_sync", "reconnection"))
            } else if !entity_updates.is_empty() {
                Some((entity_updates.clone(), "delta_update", "changes"))
            } else {
                None
            };

            // Send message if we have one
            if let Some((updates, message_type, sync_reason)) = message_params {
                send_network_message_to_client(
                    client_id,
                    player_id,
                    message_type,
                    updates,
                    sync_reason,
                );
                client_info.update_sync();
            }
        }
    }
}


/// Helper function to send network messages to clients (reduces code duplication)
fn send_network_message_to_client(
    client_id: &ClientId,
    player_id: u32,
    message_type: &str,
    entity_updates: Vec<EntityUpdate>,
    sync_reason: &str,
) {
    let message = NetworkMessage {
        message_type: message_type.to_string(),
        entity_updates: entity_updates.clone(),
        my_player_id: player_id,
    };
    
    let compact_msg = compress_message(&message);
    let json_msg = serde_json::to_string(&compact_msg).unwrap();
    
    // Send the message via WebSocket
    if let ClientId::WebSocket(addr) = client_id {
        let message = json_msg.clone();
        let client_addr = *addr;
        
        // Use Bevy's async task system to send the message
        IoTaskPool::get().spawn(async move {
            send_ws_message(client_addr, message).await;
        }).detach();
    }
    
    println!("{} sync to player {} ({}): {} entities, {} bytes", 
        message_type,
        player_id, 
        sync_reason,
        entity_updates.len(), 
        json_msg.len()
    );
}


