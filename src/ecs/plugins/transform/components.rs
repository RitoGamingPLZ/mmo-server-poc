use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkPosition {
    pub x: f32,
    pub y: f32,
}