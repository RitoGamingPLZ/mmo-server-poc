// Auto-networking macros for components
// This allows components to declare their networking behavior directly

#[macro_export]
macro_rules! networked_component {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[threshold = $threshold:expr])?
                $field_vis:vis $field:ident: $field_type:ty
            ),* $(,)?
        }
    ) => {
        // Generate the original struct
        $(#[$attr])*
        #[derive(Component, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        $vis struct $name {
            $(
                $field_vis $field: $field_type,
            )*
        }

        // Auto-implement NetworkedState
        impl crate::ecs::plugins::network::networked_state::NetworkedState for $name {
            fn get_field_changes(&self, previous: Option<&Self>) -> Vec<crate::ecs::plugins::network::networked_state::FieldUpdate> {
                let mut changes = Vec::new();
                
                if let Some(prev) = previous {
                    $(
                        networked_component!(@check_field self, prev, $field, changes, $($threshold)?);
                    )*
                } else {
                    // New entity - include all fields
                    $(
                        changes.push(crate::ecs::plugins::network::networked_state::FieldUpdate {
                            field_name: stringify!($field).to_string(),
                            value: serde_json::to_value(&self.$field).unwrap(),
                        });
                    )*
                }
                
                changes
            }
            
            fn apply_field_update(&mut self, update: &crate::ecs::plugins::network::networked_state::FieldUpdate) {
                match update.field_name.as_str() {
                    $(
                        stringify!($field) => {
                            if let Ok(value) = serde_json::from_value(update.value.clone()) {
                                self.$field = value;
                            }
                        }
                    )*
                    _ => {}
                }
            }
            
            fn get_component_name() -> &'static str {
                stringify!($name)
            }
        }
    };
    
    // Helper for checking field changes with optional threshold
    (@check_field $self:expr, $prev:expr, $field:ident, $changes:expr, $threshold:expr) => {
        if ($self.$field - $prev.$field).abs() > $threshold {
            $changes.push(crate::ecs::plugins::network::networked_state::FieldUpdate {
                field_name: stringify!($field).to_string(),
                value: serde_json::to_value(&$self.$field).unwrap(),
            });
        }
    };
    (@check_field $self:expr, $prev:expr, $field:ident, $changes:expr,) => {
        if ($self.$field - $prev.$field).abs() > 0.01 {
            $changes.push(crate::ecs::plugins::network::networked_state::FieldUpdate {
                field_name: stringify!($field).to_string(),
                value: serde_json::to_value(&$self.$field).unwrap(),
            });
        }
    };
}

#[macro_export]
macro_rules! auto_sync_networked {
    ($app:expr, $networked_type:ty, $source_type:ty) => {
        $app.add_systems(bevy::prelude::FixedUpdate, 
            move |mut commands: bevy::prelude::Commands,
                  missing_query: bevy::prelude::Query<(bevy::prelude::Entity, &$source_type), 
                      (bevy::prelude::With<crate::ecs::plugins::network::NetworkedObject>, bevy::prelude::Without<$networked_type>)>,
                  mut update_query: bevy::prelude::Query<(&$source_type, &mut $networked_type), 
                      (bevy::prelude::With<crate::ecs::plugins::network::NetworkedObject>, bevy::prelude::Changed<$source_type>)>| {
                
                // Add networked component to entities that don't have it
                for (entity, source) in missing_query.iter() {
                    commands.entity(entity).insert(<$networked_type>::from(source));
                }
                
                // Update networked component when source changes
                for (source, mut networked) in update_query.iter_mut() {
                    *networked = <$networked_type>::from(source);
                }
            }
        );
    };
}

#[macro_export]
macro_rules! impl_from_source {
    ($networked_type:ty, $source_type:ty, {$($field:ident),*}) => {
        impl From<&$source_type> for $networked_type {
            fn from(source: &$source_type) -> Self {
                Self {
                    $($field: source.$field,)*
                }
            }
        }
    };
}