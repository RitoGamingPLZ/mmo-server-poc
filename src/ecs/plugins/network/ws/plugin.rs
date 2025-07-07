use bevy::prelude::*;
use crate::ecs::plugins::network::components::*;
use crate::ecs::plugins::network::ws::components::*;
use crate::ecs::plugins::network::ws::systems::*;
use crate::ecs::plugins::network::{NetworkStateSnapshot, NetworkIdAllocator};
use crate::ecs::core::{Position, NetworkedPosition};
use crate::ecs::plugins::movement::components::{Velocity, NetworkedVelocity};
use crate::auto_sync_networked;

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
            .insert_resource(NetworkIdAllocator::new())
            .add_event::<ClientConnectedEvent>()
            .add_event::<ClientDisconnectedEvent>()
            .add_event::<HeartbeatEvent>()
            .add_event::<ClientTimeoutEvent>()
            .add_systems(Update, (poll_ws_messages, heartbeat_monitor_system))
            .add_systems(FixedUpdate, batched_broadcast_system);
        
        // Auto-register networked component sync systems
        auto_sync_networked!(app, NetworkedPosition, Position);
        auto_sync_networked!(app, NetworkedVelocity, Velocity);

        // Spawn background server in a new thread
        std::thread::spawn(move || {
            tokio::runtime::Runtime::new().unwrap().block_on(ws_server_task(ws_send));
        });
    }
}