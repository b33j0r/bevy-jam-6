use crate::states::States;
use crate::utility::despawn;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

// ───────────────────────────────────────── Component
#[derive(Component)]
pub struct GameCamera;

/// Optional resource to let the camera pan toward a point.
#[derive(Resource, Default)]
pub struct CameraFollowTarget(pub Option<Vec3>);

// ───────────────────────────────────────── Plugin
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraFollowTarget>()
            .add_systems(OnEnter(States::Game), spawn_camera)
            .add_systems(Update, pan_camera.run_if(in_state(States::Game)))
            .add_systems(OnExit(States::Game), despawn::<GameCamera>);
    }
}

// ───────────────────────────────────────── Spawn
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        GameCamera,
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 12.0, // adjust to fit full level height
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// ───────────────────────────────────────── Update
fn pan_camera(target: Res<CameraFollowTarget>, mut query: Query<&mut Transform, With<GameCamera>>) {
    if let Some(target) = target.0 {
        for mut tf in &mut query {
            tf.translation.x = target.x;
            tf.translation.y = target.y + 6.0;
            tf.translation.z = target.z + 15.0;
            tf.look_at(target, Vec3::Y);
        }
    }
}
