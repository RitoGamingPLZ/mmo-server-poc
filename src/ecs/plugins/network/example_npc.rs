// Example: How to create networked NPCs using the new NetworkedObject system

use bevy::prelude::*;
use crate::ecs::core::Position;
use crate::ecs::plugins::movement::components::Velocity;
use crate::ecs::plugins::network::{NetworkedObject, NetworkedObjectType, NetworkIdAllocator};

#[derive(Component)]
pub struct NPC {
    pub name: String,
    pub npc_type: NPCType,
}

#[derive(Clone, Debug)]
pub enum NPCType {
    Merchant,
    Guard,
    Enemy,
    Projectile,
}

#[derive(Bundle)]
pub struct NPCBundle {
    pub npc: NPC,
    pub position: Position,
    pub velocity: Velocity,
    pub networked_object: NetworkedObject,
}

impl NPCBundle {
    pub fn new_merchant(name: String, position: Position, id_allocator: &mut NetworkIdAllocator) -> Self {
        Self {
            npc: NPC {
                name,
                npc_type: NPCType::Merchant,
            },
            position,
            velocity: Velocity { x: 0.0, y: 0.0 },
            networked_object: NetworkedObject::new_npc(id_allocator.allocate_id()),
        }
    }
    
    pub fn new_projectile(position: Position, velocity: Velocity, id_allocator: &mut NetworkIdAllocator) -> Self {
        Self {
            npc: NPC {
                name: "Projectile".to_string(),
                npc_type: NPCType::Projectile,
            },
            position,
            velocity,
            networked_object: NetworkedObject::new_with_type(
                id_allocator.allocate_id(),
                NetworkedObjectType::Projectile
            ),
        }
    }
}

// Example spawn system
pub fn spawn_example_npcs(
    mut commands: Commands,
    mut id_allocator: ResMut<NetworkIdAllocator>,
    mut spawned: Local<bool>,
) {
    if !*spawned {
        *spawned = true;
        
        // Spawn a merchant NPC
        commands.spawn(NPCBundle::new_merchant(
            "Bob the Merchant".to_string(),
            Position { x: 100.0, y: 100.0 },
            &mut id_allocator,
        ));
        
        // Spawn a moving projectile
        commands.spawn(NPCBundle::new_projectile(
            Position { x: 50.0, y: 50.0 },
            Velocity { x: 10.0, y: 5.0 },
            &mut id_allocator,
        ));
        
        println!("Spawned example NPCs - they will automatically be networked!");
    }
}

/*
To use this system, just add it to your plugin:

.add_systems(Update, spawn_example_npcs)

The NPCs will automatically be synchronized to all clients because they have:
1. NetworkedObject component (marks them for networking)
2. Position/Velocity components (automatically synced via our macros)
3. Unique network IDs (allocated from the non-player range)

Clients will receive these as networked entities with their object types,
allowing different rendering/behavior based on NetworkedObjectType!
*/