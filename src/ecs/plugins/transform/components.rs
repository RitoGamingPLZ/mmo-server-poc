use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::ecs::plugins::network::NetworkedComponent;

#[derive(Component, Debug, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkPosition {
    pub x: f32,
    pub y: f32,
}

impl NetworkedComponent for NetworkPosition {
    fn component_name() -> &'static str { "position" }
}