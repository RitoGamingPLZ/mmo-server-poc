use bevy::prelude::*;
use crate::ecs::plugins::network::components::*;
use crate::ecs::plugins::network::systems::*;

// ============================================================================
// PLUGIN DEFINITION
// ============================================================================

pub enum NetworkMode {
    Ws,
}

pub struct NetworkPlugin {
    pub mode: NetworkMode,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<NetworkIdAllocator>()
            .init_resource::<NetworkUpdates>()
            
            // Add WebSocket plugin based on mode
            .add_plugins(crate::ecs::plugins::network::ws::WsNetworkPlugin)
            
            // Add networking components to players when they spawn (after player spawn system)
            .add_systems(Update, (
                crate::ecs::plugins::player::systems::player_spawn_system,
                add_networking_to_players_system,
            ).chain())
            
            // Network systems run at 20Hz for consistent packet rate
            .add_systems(FixedUpdate, (
                detect_component_changes_system::<crate::ecs::plugins::transform::NetworkPosition>,
                detect_component_changes_system::<crate::ecs::plugins::movement::NetworkVelocity>,
                build_delta_updates_system,
                build_full_sync_system,
                crate::ecs::plugins::network::ws::systems::send_network_updates_to_clients_system,
            ).chain()
            .after(crate::ecs::plugins::transform::systems::sync_position_to_network_system)
            .after(crate::ecs::plugins::movement::systems::sync_velocity_to_network_system));
    }
}