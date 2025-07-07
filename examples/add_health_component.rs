// Example: How to add a new NetworkedHealth component

// Step 1: Define your component (in your actual component file)
use crate::networked_component;

#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

// Step 2: Create networked version using the macro
networked_component! {
    pub struct NetworkedHealth {
        #[threshold = 0.1]  // Only sync if change > 0.1 HP
        pub current: f32,
        #[threshold = 0.1]
        pub max: f32,
    }
}

// Step 3: Implement conversion
impl_from_source!(NetworkedHealth, Health, {current, max});

// Step 4: Register in component_registry.rs - just add ONE line:
/*
register_networked_components!(
    NetworkedPosition,
    NetworkedVelocity,
    NetworkedHealth     // <- Add this line
);
*/

// Step 5: Add auto-sync in your plugin
/*
auto_sync_networked!(app, NetworkedHealth, Health);
*/

// That's it! Now NetworkedHealth is automatically synchronized with:
// - Automatic change detection (only syncs when health changes by >0.1)
// - Delta updates (only changed fields are sent)
// - Full sync for new clients
// - Message compression
// - Zero performance overhead when health doesn't change

fn main() {
    println!("This is just an example file showing how easy it is to add networked components!");
}