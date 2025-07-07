use bevy::prelude::*;
use crate::ecs::plugins::network::{UdpNetworkPlugin, WsNetworkPlugin};
use crate::ecs::core::{Position, NetworkedPosition};
use crate::auto_sync_networked;

pub enum NetworkMode {
    Udp,
    Ws,
}

pub struct NetworkPlugin {
    pub mode: NetworkMode,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        match self.mode {
            NetworkMode::Udp => {
                app.add_plugins(UdpNetworkPlugin);
            }
            NetworkMode::Ws => {
                app.add_plugins(WsNetworkPlugin);
            }
        }
        
        // Auto-register core networked component sync for position
        auto_sync_networked!(app, NetworkedPosition, Position);
    }
}