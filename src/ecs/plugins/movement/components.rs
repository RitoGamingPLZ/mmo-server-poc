use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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

