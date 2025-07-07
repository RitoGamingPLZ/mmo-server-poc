use bevy::prelude::*;
use crate::ecs::plugins::network::components::*;
use crate::ecs::plugins::network::ws::components::*;
use crate::ecs::plugins::network::ws::systems::*;

pub struct WsNetworkPlugin;

impl Plugin for WsNetworkPlugin {
    fn build(&self, app : &mut App) {
        let (ws_send, ws_recv) = crossbeam_channel::unbounded::<WsEvent>();
        app
            .insert_resource(WsSendChannel(ws_send.clone()))
            .insert_resource(WsRecvChannel(ws_recv))
            .init_resource::<ConnectedClients>()
            .init_resource::<NetworkPlayerRegistry>()
            .add_event::<ClientConnectedEvent>()
            .add_event::<ClientDisconnectedEvent>()
            .add_systems(Update, poll_ws_messages);

        // Spawn background server in a new thread
        std::thread::spawn(move || {
            tokio::runtime::Runtime::new().unwrap().block_on(ws_server_task(ws_send));
        });
    }
}