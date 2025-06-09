use bevy::prelude::{Commands, Component, Entity, Query, With};

// utility
pub fn despawn<M: Component>(mut commands: Commands, q: Query<Entity, With<M>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}
