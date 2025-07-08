use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::ecs::plugins::network::NetworkedComponent;

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

#[derive(Component, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkVelocity {
    pub x: f32,
    pub y: f32,
}

impl NetworkedComponent for NetworkVelocity {
    fn component_name() -> &'static str { "velocity" }
}