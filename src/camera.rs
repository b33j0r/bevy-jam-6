use crate::states::States;
use crate::utility::despawn;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::PrimaryWindow;

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
            .add_systems(
                Update,
                (pan_camera, mouse_zoom_camera, mouse_pan_camera).run_if(in_state(States::Game)),
            )
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
                viewport_height: 32.0, // adjust to fit full level height
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// ───────────────────────────────────────── Update
fn pan_camera(
    mut target: ResMut<CameraFollowTarget>,
    mut query: Query<&mut Transform, With<GameCamera>>,
) {
    if let Some(target) = target.0.take() {
        for mut tf in &mut query {
            tf.translation.x = target.x;
            tf.translation.y = target.y + 6.0;
            tf.translation.z = target.z + 15.0;
            tf.look_at(target, Vec3::Y);
        }
    }
}

fn mouse_zoom_camera(
    mut commands: Commands,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<GameCamera>>,
) {
    let zoom_speed = 0.5;
    let min_zoom = 0.1;
    let max_zoom = 5.0;

    for event in mouse_wheel_events.read() {
        for mut tf in &mut query {
            let mut zoom = tf.scale.x - event.y * zoom_speed;
            zoom = zoom.clamp(min_zoom, max_zoom);
            tf.scale = Vec3::splat(zoom);
            commands.insert_resource(CameraFollowTarget(Some(tf.translation)));
        }
    }
}

fn mouse_pan_camera(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    mut transforms: Query<&mut Transform, With<GameCamera>>,
    mut last_cursor: Local<Option<Vec2>>,
) {
    let window = windows.single().expect("Failed to get primary window");

    if mouse_button_input.pressed(MouseButton::Right) {
        if let Some(current_pos) = window.cursor_position() {
            if let Some(previous_pos) = *last_cursor {
                let delta = current_pos - previous_pos;

                let (camera, camera_transform) =
                    cameras.single().expect("Expected exactly one GameCamera");

                let ray_origin = camera
                    .viewport_to_world(camera_transform, previous_pos)
                    .expect("Failed to compute ray for previous cursor position")
                    .origin;
                let ray_target = camera
                    .viewport_to_world(camera_transform, current_pos)
                    .expect("Failed to compute ray for current cursor position")
                    .origin;

                let world_delta = ray_origin - ray_target;

                for mut transform in &mut transforms {
                    transform.translation += world_delta;
                }
            }
            *last_cursor = Some(current_pos);
        }
    } else {
        *last_cursor = None;
    }
}
