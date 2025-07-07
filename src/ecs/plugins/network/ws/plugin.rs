use bevy::prelude::*;
use crate::ecs::plugins::network::components::*;
use crate::ecs::plugins::network::ws::components::*;
use crate::ecs::plugins::network::ws::systems::*;
use crate::ecs::plugins::network::{NetworkStateSnapshot, NetworkIdAllocator};
use crate::ecs::plugins::network::component_registry::NetworkedComponentRegistry;

pub struct WsNetworkPlugin;

impl Plugin for WsNetworkPlugin {
    fn build(&self, app : &mut App) {
        let (ws_send, ws_recv) = crossbeam_channel::unbounded::<WsEvent>();
        app
            .insert_resource(WsSendChannel(ws_send.clone()))
            .insert_resource(WsRecvChannel(ws_recv))
            .init_resource::<ConnectedClients>()
            .init_resource::<NetworkPlayerRegistry>()
            .init_resource::<NetworkStateSnapshot>()
            .init_resource::<NetworkedComponentRegistry>()
            .insert_resource(NetworkIdAllocator::new())
            .add_event::<ClientConnectedEvent>()
            .add_event::<ClientDisconnectedEvent>()
            .add_systems(Update, poll_ws_messages)
            .add_systems(FixedUpdate, batched_broadcast_system);
        
        // Networked components are auto-registered in their respective plugins:
        // - NetworkedPosition: auto-registered in NetworkPlugin 
        // - NetworkedVelocity: auto-registered in MovementPlugin
        // - Additional components: auto-registered in their respective plugins

        // Spawn background server in a new thread
        std::thread::spawn(move || {
            tokio::runtime::Runtime::new().unwrap().block_on(ws_server_task(ws_send));
        });
    }
}