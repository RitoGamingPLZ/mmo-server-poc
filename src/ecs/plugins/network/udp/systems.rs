use bevy::prelude::*;
use bevy_renet::renet::{RenetServer, DefaultChannel, ServerEvent};
use crate::ecs::plugins::input::{InputCommand, InputCommandEvent};
use crate::ecs::plugins::network::components::*;

pub fn handle_server_events(
    mut server_events: EventReader<ServerEvent>,
    mut connected_clients: ResMut<ConnectedClients>,
    mut player_registry: ResMut<NetworkPlayerRegistry>,
    mut connect_events: EventWriter<ClientConnectedEvent>,
    mut disconnect_events: EventWriter<ClientDisconnectedEvent>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("UDP Client {} connected", client_id);
                
                let client_id_enum = ClientId::Udp(*client_id);
                let player_id = generate_player_id(); // Use unified player ID generation
                
                let client_info = ClientInfo::new(client_id_enum.clone());
                
                connected_clients.clients.insert(client_id_enum.clone(), client_info);
                player_registry.register_player(client_id_enum.clone(), player_id);
                
                connect_events.send(ClientConnectedEvent {
                    client_id: client_id_enum,
                    player_id,
                });
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("UDP Client {} disconnected: {:?}", client_id, reason);
                
                let client_id_enum = ClientId::Udp(*client_id);
                
                // Get player ID before removing from registry
                let player_id = player_registry.unregister_player(&client_id_enum).unwrap_or(0);
                
                connected_clients.clients.remove(&client_id_enum);
                
                disconnect_events.send(ClientDisconnectedEvent {
                    client_id: client_id_enum,
                    player_id,
                    reason: format!("{:?}", reason),
                });
            }
        }
    }
}

pub fn receive_network_input(
    mut server: ResMut<RenetServer>,
    player_registry: Res<NetworkPlayerRegistry>,
    mut input_events: EventWriter<InputCommandEvent>,
) {
    for client_id in server.clients_id() {
        let client_id_enum = ClientId::Udp(client_id);
        
        if let Some(player_id) = player_registry.get_player_id(&client_id_enum) {
            while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered) {
                if let Ok(command) = rmp_serde::from_slice::<InputCommand>(&message) {
                    input_events.send(InputCommandEvent {
                        player_id,
                        command,
                    });
                }
            }
        }
    }
}