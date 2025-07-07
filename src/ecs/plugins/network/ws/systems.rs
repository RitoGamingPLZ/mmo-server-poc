use bevy::prelude::*;
use tokio_tungstenite::tungstenite::Message;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt};
use crossbeam_channel::Sender;
use crate::ecs::plugins::input::{InputCommand, InputCommandEvent};
use crate::ecs::plugins::player::{PlayerSpawnEvent, PlayerDespawnEvent};
use crate::ecs::plugins::network::components::*;
use crate::ecs::plugins::network::ws::components::*;

pub async fn ws_server_task(ws_send: Sender<WsEvent>) {
    let host = std::env::var("WEBSOCKET_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("WEBSOCKET_PORT").unwrap_or_else(|_| "5000".to_string());
    let addr = format!("{}:{}", host, port);
    
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("WS server running on ws://{}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        let ws_send = ws_send.clone();
        // Notify connected
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
                let now = std::time::Instant::now();
                
                // Update connected clients
                let client_info = ClientInfo {
                    id: client_id.clone(),
                    connected_at: now,
                };
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
        }
    }
}