use crate::states::States;
use crate::utility::despawn;
use bevy::app::{App, Plugin};
use bevy::math::Vec3;
use bevy::prelude::{Camera3d, Commands, Component, OnEnter, OnExit, Transform};

// ───────────────────────────────────────── Camera
#[derive(Component)]
pub struct GameCamera;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(States::Game), spawn_camera)
            .add_systems(OnExit(States::Game), despawn::<GameCamera>);
    }
}
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        GameCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 6.0, 30.0).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
    ));
}
