use crate::states;
use bevy::app::{App, Plugin};
use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLight};
use bevy::prelude::{default, Commands, Component, Entity, OnEnter, OnExit, Query, With};

#[derive(Component)]
pub struct GameLevel;

#[derive(Component)]
pub struct GameLight;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(states::States::Game), (init_level,))
            .add_systems(OnExit(states::States::Game), (deinit_level,));
    }
}

fn init_level(mut commands: Commands) {
    commands.spawn((
        GameLight,
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .build(),
    ));
}

fn deinit_level(mut commands: Commands, query: Query<Entity, With<GameLight>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
