use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputCommand {
    Move { direction: Vec2 },
    Stop,
}

#[derive(Event)]
pub struct InputCommandEvent {
    pub player_id: u32,
    pub command: InputCommand,
}

#[derive(Resource, Default)]
pub struct InputBuffer {
    pub commands: HashMap<u32, InputCommand>,
}