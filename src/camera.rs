use crate::states;
use bevy::app::{App, Plugin};
use bevy::math::Vec3;
use bevy::prelude::{
    Camera3d, Commands, Component, Entity, OnEnter, OnExit, Query, Transform, With,
};

#[derive(Component)]
pub struct GameCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(states::States::Game), (init_camera,))
            .add_systems(OnExit(states::States::Game), (deinit_camera,));
    }
}

fn init_camera(mut commands: Commands) {
    commands.spawn((
        GameCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
    ));
}

fn deinit_camera(mut commands: Commands, query: Query<Entity, With<GameCamera>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
