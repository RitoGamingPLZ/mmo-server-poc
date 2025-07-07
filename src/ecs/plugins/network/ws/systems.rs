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
use tokio_tungstenite::tungstenite::Message;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::StreamExt;
use crossbeam_channel::Sender;

use crate::ecs::plugins::input::{InputCommand, InputCommandEvent};
use crate::ecs::plugins::player::{PlayerSpawnEvent, PlayerDespawnEvent};
use crate::ecs::plugins::network::components::*;
use crate::ecs::plugins::network::ws::components::*;
use crate::ecs::plugins::network::networked_state::*;
use crate::ecs::plugins::network::NetworkedObject;
use crate::ecs::core::NetworkedPosition;
use crate::ecs::plugins::movement::NetworkedVelocity;

// Network Configuration Constants
/// How long a client can go without sending a heartbeat before being disconnected
const CLIENT_TIMEOUT_SECONDS: u64 = 30;

/// How long since last sync to trigger a full sync (instead of delta) for reconnecting clients
const RECONNECT_THRESHOLD_SECONDS: u64 = 3;

/// Special message that clients send to maintain their connection
const HEARTBEAT_MESSAGE: &str = "heartbeat";

pub async fn ws_server_task(ws_send: Sender<WsEvent>) {
    let host = std::env::var("WEBSOCKET_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("WEBSOCKET_PORT").unwrap_or_else(|_| "5000".to_string());
    let addr = format!("{}:{}", host, port);
    
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("WS server running on ws://{}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        let ws_send = ws_send.clone();
        let _ = ws_send.send(WsEvent::Connected(addr));
        
        tokio::spawn(async move {
            let mut ws_stream = accept_async(stream).await.unwrap();
            println!("New WS client: {:?}", addr);

            while let Some(msg) = ws_stream.next().await {
                if let Ok(msg) = msg {
                    match msg {
                        Message::Binary(data) => {
                            let _ = ws_send.send(WsEvent::Message { client: addr, data: data.to_vec() });
                        }
                        Message::Text(text) => {
                            let _ = ws_send.send(WsEvent::TextMessage { client: addr, text: text.to_string() });
                        }
                        _ => {}
                    }
                } else {
                    break;
                }
            }
            println!("WS client disconnected: {:?}", addr);
            let _ = ws_send.send(WsEvent::Disconnected(addr));
        });
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
            WsEvent::Message { client, data } => {
                let client_id = ClientId::WebSocket(client);
                
                if let Some(player_id) = player_registry.get_player_id(&client_id) {
                    // First, try to convert the bytes to UTF-8 string
                    match std::str::from_utf8(&data) {
                        Ok(txt) => {
                            println!("Input from WS player {}: {:?}", player_id, txt);
                            // Then, try to parse as InputCommand
                            if let Ok(command) = serde_json::from_str::<InputCommand>(txt) {
                                input_events.send(InputCommandEvent {
                                    player_id,
                                    command,
                                });
                            }
                            // Silently ignore parse errors (or add else block to log)
                        }
                        Err(_) => {
                            println!("Invalid UTF-8 input from WS player {}: {:?}", player_id, data);
                        }
                    }
                } else {
                    println!("Received message from unregistered WS client: {:?}", client);
                }
            }
            WsEvent::TextMessage { client, text } => {
                let client_id = ClientId::WebSocket(client);
                
                if let Some(player_id) = player_registry.get_player_id(&client_id) {
                    // Check if this is a heartbeat message
                    if text.trim() == HEARTBEAT_MESSAGE {
                        // Update client heartbeat to prevent timeout
                        if let Some(client_info) = connected_clients.clients.get_mut(&client_id) {
                            client_info.update_heartbeat();
                        }
                        println!("💓 Heartbeat received from player {}", player_id);
                        continue;
                    }
                    
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
            WsEvent::Broadcast { client: _, message: _ } => {
                // Handle broadcast messages if needed
            }
        }
    }
}

pub fn batched_broadcast_system(
    mut snapshot: ResMut<NetworkStateSnapshot>,
    position_query: Query<(Entity, &NetworkedPosition, &NetworkedObject)>,
    velocity_query: Query<(Entity, &NetworkedVelocity, &NetworkedObject)>,
    player_registry: Res<NetworkPlayerRegistry>,
    mut connected_clients: ResMut<ConnectedClients>,
) {
    let reconnect_threshold = std::time::Duration::from_secs(RECONNECT_THRESHOLD_SECONDS);
    
    let mut change_buffer = ChangeBuffer::default();
    
    // Collect position changes
    track_networked_component_changes::<NetworkedPosition>(
        &mut snapshot,
        position_query,
        &mut change_buffer,
    );
    
    // Collect velocity changes
    track_networked_component_changes::<NetworkedVelocity>(
        &mut snapshot,
        velocity_query,
        &mut change_buffer,
    );
    
    // Build batched updates
    let entity_updates = build_batched_updates(&mut change_buffer);
    
    // Process each client individually
    for (client_id, client_info) in connected_clients.clients.iter_mut() {
        if let Some(player_id) = player_registry.get_player_id(client_id) {
            let needs_full_sync = client_info.needs_full_sync_after_reconnect(reconnect_threshold);
            
            if needs_full_sync {
                // Send full sync to this specific client
                let message = NetworkMessage {
                    message_type: "full_sync".to_string(),
                    entity_updates: entity_updates.clone(),
                    my_player_id: player_id,
                };
                
                let compact_msg = compress_message(&message);
                let json_msg = serde_json::to_string(&compact_msg).unwrap();
                
                // Mark this client as synced
                client_info.update_sync();
                
                println!("Full sync to player {} (reconnection): {} entities, {} bytes", 
                    player_id, entity_updates.len(), json_msg.len());
                    
            } else if !entity_updates.is_empty() {
                // Send delta update to this client
                let message = NetworkMessage {
                    message_type: "delta_update".to_string(),
                    entity_updates: entity_updates.clone(),
                    my_player_id: player_id,
                };
                
                let compact_msg = compress_message(&message);
                let json_msg = serde_json::to_string(&compact_msg).unwrap();
                
                println!("Delta update to player {}: {} entities, {} bytes", 
                    player_id, entity_updates.len(), json_msg.len());
            }
        }
    }
}

// Heartbeat monitoring system - checks for timed out clients
pub fn heartbeat_monitor_system(
    mut connected_clients: ResMut<ConnectedClients>,
    mut player_registry: ResMut<NetworkPlayerRegistry>,
    mut disconnect_events: EventWriter<ClientDisconnectedEvent>,
    mut despawn_events: EventWriter<PlayerDespawnEvent>,
) {
    let timeout_duration = std::time::Duration::from_secs(CLIENT_TIMEOUT_SECONDS);
    let mut timed_out_clients = Vec::new();
    
    // Check for timed out clients
    for (client_id, client_info) in &connected_clients.clients {
        if client_info.is_timed_out(timeout_duration) {
            timed_out_clients.push(client_id.clone());
        }
    }
    
    // Remove timed out clients
    for client_id in timed_out_clients {
        if let Some(player_id) = player_registry.unregister_player(&client_id) {
            connected_clients.clients.remove(&client_id);
            
            disconnect_events.send(ClientDisconnectedEvent {
                client_id: client_id.clone(),
                player_id,
                reason: "Heartbeat timeout".to_string(),
            });
            
            despawn_events.send(PlayerDespawnEvent { player_id });
            
            println!("Player {} timed out due to missing heartbeat", player_id);
        }
    }
}

