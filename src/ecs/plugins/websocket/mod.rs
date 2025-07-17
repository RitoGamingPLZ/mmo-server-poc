pub mod components;
pub mod systems;

use bevy::prelude::*;
use components::WebSocketConnections;
use systems::{handle_websocket_messages, send_network_updates, setup_websocket_server};

// WebSocket plugin
pub struct WebSocketPlugin {
    pub port: u16,
}

impl Default for WebSocketPlugin {
    fn default() -> Self {
        Self { port: 5000 }
    }
}

impl Plugin for WebSocketPlugin {
    fn build(&self, app: &mut App) {
        let port = self.port;
        app.insert_resource(WebSocketConnections::default())
            .add_systems(Startup, move |connections: Res<WebSocketConnections>| {
                setup_websocket_server(connections, port);
            })
            .add_systems(Update, (
                handle_websocket_messages.before(crate::ecs::systems::player_spawn_system),
                send_network_updates
                    .after(crate::ecs::plugins::network::systems::build_delta_updates_system)
                    .after(crate::ecs::plugins::network::systems::build_full_sync_system),
            ));
    }
}