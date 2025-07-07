use bevy::prelude::*;
use crate::ecs::plugins::network::{UdpNetworkPlugin, WsNetworkPlugin};

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
    }
}