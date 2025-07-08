use bevy::prelude::*;
use crate::ecs::plugins::input::components::*;
use crate::ecs::plugins::input::systems::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InputCommandEvent>()
            .insert_resource(InputBuffer::default())
            .add_systems(Update, (
                input_validation_system,
                input_event_system,
                input_processing_system,
                // reset_desired_velocity_system,
            ).chain());
    }
}