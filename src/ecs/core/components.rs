use bevy::prelude::*;
use crate::{networked_component, impl_from_source};

#[derive(Component, Debug, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}


// Networked version of Position
networked_component! {
    pub struct NetworkedPosition {
        #[threshold = 0.01]
        pub x: f32,
        #[threshold = 0.01]
        pub y: f32,
    }
}

impl_from_source!(NetworkedPosition, Position, {x, y});