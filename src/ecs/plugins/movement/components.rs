use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}