use bevy::prelude::*;
use crate::{networked_component, impl_from_source};

// Regular component
#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Mana {
    pub current: f32,
    pub max: f32,
}

// Networked versions - they auto-register themselves!
networked_component! {
    pub struct NetworkedHealth {
        #[threshold = 0.1]  // Only sync if change > 0.1 HP
        pub current: f32,
        #[threshold = 0.1]
        pub max: f32,
    }
}

networked_component! {
    pub struct NetworkedMana {
        #[threshold = 0.1]  // Only sync if change > 0.1 mana
        pub current: f32,
        #[threshold = 0.1]
        pub max: f32,
    }
}

// Conversion implementations
impl_from_source!(NetworkedHealth, Health, {current, max});
impl_from_source!(NetworkedMana, Mana, {current, max});

// Auto-registration - just call this once in your plugin
pub fn register_combat_components() {
    use crate::register_all_networked_components;
    register_all_networked_components!(NetworkedHealth, NetworkedMana);
}