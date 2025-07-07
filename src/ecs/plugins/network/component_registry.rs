use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use crate::ecs::plugins::network::networked_state::*;
use crate::ecs::plugins::network::NetworkedObject;

/// Trait for components that can auto-register themselves
pub trait AutoRegisterNetworkedComponent {
    fn register();
}

/// Global registry that collects networked components
static COMPONENT_REGISTRY: OnceLock<Mutex<Vec<fn() -> Box<dyn NetworkedComponentSyncer>>>> = OnceLock::new();

/// A registry that manages all networked components dynamically
/// This allows you to add new networked components without modifying the sync system
#[derive(Resource)]
pub struct NetworkedComponentRegistry {
    pub syncers: Vec<Box<dyn NetworkedComponentSyncer>>,
}

impl Default for NetworkedComponentRegistry {
    fn default() -> Self {
        let registry = COMPONENT_REGISTRY.get_or_init(|| Mutex::new(Vec::new()));
        let component_factories = registry.lock().unwrap();
        
        let syncers = component_factories.iter()
            .map(|factory| factory())
            .collect();

        Self { syncers }
    }
}

/// Register a networked component type
pub fn register_networked_component<T: NetworkedState + 'static>() {
    let registry = COMPONENT_REGISTRY.get_or_init(|| Mutex::new(Vec::new()));
    let mut component_factories = registry.lock().unwrap();
    
    // Check if already registered (avoid duplicates)
    let _type_name = std::any::type_name::<T>();
    if component_factories.iter().any(|_| false) { // TODO: Better duplicate detection
        return;
    }
    
    component_factories.push(|| {
        Box::new(NetworkedComponentSyncerImpl::<T>::new())
    });
}


/// Macro to easily register components defined elsewhere
#[macro_export]
macro_rules! register_all_networked_components {
    ($($component:ty),*) => {
        $(
            <$component as crate::ecs::plugins::network::component_registry::AutoRegisterNetworkedComponent>::register();
        )*
    };
}

/// Trait for syncing specific component types
pub trait NetworkedComponentSyncer: Send + Sync {
    fn sync_full(&self, entity: Entity, network_id: u32, world: &World) -> Option<ComponentUpdate>;
    fn sync_delta(&self, entity: Entity, network_id: u32, world: &World, snapshot: &mut NetworkStateSnapshot) -> Option<ComponentUpdate>;
}

/// Generic implementation for any component that implements NetworkedState
pub struct NetworkedComponentSyncerImpl<T: NetworkedState> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: NetworkedState> NetworkedComponentSyncerImpl<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: NetworkedState> NetworkedComponentSyncer for NetworkedComponentSyncerImpl<T> {
    fn sync_full(&self, entity: Entity, _network_id: u32, world: &World) -> Option<ComponentUpdate> {
        if let Some(component) = world.get::<T>(entity) {
            let field_updates = component.get_field_changes(None);
            if !field_updates.is_empty() {
                Some(ComponentUpdate {
                    component_name: T::get_component_name().to_string(),
                    field_updates,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
    
    fn sync_delta(&self, entity: Entity, _network_id: u32, world: &World, snapshot: &mut NetworkStateSnapshot) -> Option<ComponentUpdate> {
        if let Some(component) = world.get::<T>(entity) {
            let component_name = T::get_component_name();
            let snapshot_key = (entity, component_name);
            let current_value = serde_json::to_value(component).unwrap();
            
            let field_updates = if let Some(previous_value) = snapshot.snapshots.get(&snapshot_key) {
                if let Ok(previous_component) = serde_json::from_value::<T>(previous_value.clone()) {
                    component.get_field_changes(Some(&previous_component))
                } else {
                    component.get_field_changes(None)
                }
            } else {
                component.get_field_changes(None)
            };
            
            if !field_updates.is_empty() {
                snapshot.snapshots.insert(snapshot_key, current_value);
                Some(ComponentUpdate {
                    component_name: component_name.to_string(),
                    field_updates,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Build full sync updates using the registry approach
pub fn build_full_sync_updates_registry(
    networked_query: &Query<(Entity, &NetworkedObject)>,
    world: &World,
    registry: &NetworkedComponentRegistry,
) -> Vec<EntityUpdate> {
    let mut entity_data: HashMap<u32, Vec<ComponentUpdate>> = HashMap::new();

    // Iterate through all networked entities
    for (entity, networked_obj) in networked_query.iter() {
        // Use each registered component syncer
        for syncer in &registry.syncers {
            if let Some(component_update) = syncer.sync_full(entity, networked_obj.network_id, world) {
                entity_data.entry(networked_obj.network_id)
                    .or_default()
                    .push(component_update);
            }
        }
    }

    // Convert to EntityUpdate format
    entity_data.into_iter().map(|(network_id, components)| {
        EntityUpdate {
            entity_id: network_id,
            components,
        }
    }).collect()
}

/// Build delta updates using the registry approach
pub fn build_delta_updates_registry(
    networked_query: &Query<(Entity, &NetworkedObject)>,
    world: &World,
    registry: &NetworkedComponentRegistry,
    snapshot: &mut NetworkStateSnapshot,
) -> Vec<EntityUpdate> {
    let mut entity_data: HashMap<u32, Vec<ComponentUpdate>> = HashMap::new();

    // Iterate through all networked entities
    for (entity, networked_obj) in networked_query.iter() {
        // Use each registered component syncer
        for syncer in &registry.syncers {
            if let Some(component_update) = syncer.sync_delta(entity, networked_obj.network_id, world, snapshot) {
                entity_data.entry(networked_obj.network_id)
                    .or_default()
                    .push(component_update);
            }
        }
    }

    // Convert to EntityUpdate format
    entity_data.into_iter().map(|(network_id, components)| {
        EntityUpdate {
            entity_id: network_id,
            components,
        }
    }).collect()
}