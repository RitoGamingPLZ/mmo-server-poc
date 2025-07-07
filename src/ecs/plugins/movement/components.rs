use bevy::prelude::*;
use crate::{networked_component, impl_from_source};

#[derive(Component, Debug, Clone, Copy)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct DesiredVelocity {
    pub x: f32,
    pub y: f32,
}

impl Default for DesiredVelocity {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Friction {
    pub coefficient: f32,
}

impl Default for Friction {
    fn default() -> Self {
        Self { coefficient: 0.95 }
    }
}

// Networked versions of components with auto-sync
networked_component! {
    pub struct NetworkedVelocity {
        #[threshold = 0.01]
        pub x: f32,
        #[threshold = 0.01]
        pub y: f32,
    }
}

impl_from_source!(NetworkedVelocity, Velocity, {x, y});