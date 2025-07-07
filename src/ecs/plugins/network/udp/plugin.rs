use bevy::prelude::*;
use bevy_renet::{
    renet::{RenetServer, ConnectionConfig},
    RenetServerPlugin,
    netcode::{NetcodeServerTransport, NetcodeServerPlugin},
};
use crate::ecs::plugins::network::components::*;
use crate::ecs::plugins::network::udp::systems::*;

fn create_renet_server() -> RenetServer {
    let server_config = ConnectionConfig::default();
    RenetServer::new(server_config)
}

fn create_netcode_transport() -> NetcodeServerTransport {
    use bevy_renet::netcode::{ServerAuthentication, ServerConfig};
    use std::{net::UdpSocket, time::SystemTime};
    let public_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).unwrap();
    
    let server_config = ServerConfig {
        current_time: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap(),
        max_clients: 64,
        protocol_id: 0,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };
    
    NetcodeServerTransport::new(server_config, socket).unwrap()
}

pub struct UdpNetworkPlugin;

impl Plugin for UdpNetworkPlugin  {
    fn build(&self, app: &mut App) {
        app.insert_resource(create_renet_server())
            .add_plugins(NetcodeServerPlugin)
            .insert_resource(create_netcode_transport())
            .add_plugins(RenetServerPlugin)
            .init_resource::<ConnectedClients>()
            .init_resource::<NetworkPlayerRegistry>()
            .add_event::<ClientConnectedEvent>()
            .add_event::<ClientDisconnectedEvent>()
            .add_systems(Update, (handle_server_events, receive_network_input));
    }
}