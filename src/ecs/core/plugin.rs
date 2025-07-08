use bevy::prelude::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, _app: &mut App) {
        // Core plugin handles basic game resources only
        // Position and networking sync moved to TransformPlugin
    }
}