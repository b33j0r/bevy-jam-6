//! main.rs — Bevy 0.16-compatible “Sheeple” skeleton
//! Requires bevy = "0.16", bevy_asset_loader = "0.23"

mod assets; // contains `ModelAssets` exactly as you posted
mod states; // enum States { AssetLoading, Game }

use assets::ModelAssets;
use states::States;
use std::time::Duration;

use bevy::ecs::error::info;
use bevy::{
    asset::AssetMetaCheck, math::primitives::Cuboid, pbr::CascadeShadowConfigBuilder, prelude::*,
};
use bevy_asset_loader::prelude::*;

// ───────────────────────────────────────── constants
const GROUND_HALF_EXTENT: i32 = 7;
const SPAWN_PERIOD: f32 = 1.0;
const GRAVITY: f32 = -9.81;
const WALK_SPEED: f32 = 2.0;

// ───────────────────────────────────────── resources
#[derive(Resource, Default)]
struct Score {
    spawned: u32,
    saved: u32,
    died: u32,
}

#[derive(Resource)]
struct SpawnTimer(Timer);

#[derive(Resource)]
struct AnimGraph {
    graph: Handle<AnimationGraph>,
    nodes: Vec<AnimationNodeIndex>, // idle = 0, run = 1, … extend later
}

// ───────────────────────────────────────── components / markers
#[derive(Component)]
struct Ground;
#[derive(Component)]
struct SpawnBox;
#[derive(Component)]
struct Sheeple;
#[derive(Component)]
struct GameCamera;

/// simple kinematic data
#[derive(Component)]
struct Kinematic {
    vel: Vec3,
    on_ground: bool,
}

// ───────────────────────────────────────── main
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .init_state::<States>()
        .add_loading_state(
            LoadingState::new(States::AssetLoading)
                .continue_to_state(States::Game)
                .load_collection::<ModelAssets>(),
        )
        .insert_resource(Score::default())
        .insert_resource(SpawnTimer(Timer::from_seconds(
            SPAWN_PERIOD,
            TimerMode::Repeating,
        )))
        .add_plugins(CameraPlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(SheeplePlugin)
        .run();
}

// ───────────────────────────────────────── Camera
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
        Transform::from_xyz(0.0, 6.0, 15.0).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
    ));
}

// ───────────────────────────────────────── Level
pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(States::Game),
            (spawn_light, spawn_ground, spawn_box),
        )
        .add_systems(
            OnExit(States::Game),
            (
                despawn::<Ground>,
                despawn::<SpawnBox>,
                despawn::<DirectionalLight>,
            ),
        );
    }
}
fn spawn_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -50_f32.to_radians(),
            30_f32.to_radians(),
            0.0,
        )),
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 50.0,
            ..default()
        }
        .build(),
    ));
}
fn spawn_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::default().mesh());
    let mat = mats.add(Color::srgb(0.2, 0.8, 0.2));
    for x in -GROUND_HALF_EXTENT..GROUND_HALF_EXTENT {
        commands.spawn((
            Ground,
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(x as f32 + 0.5, 0.5, 0.0),
        ));
    }
}
fn spawn_box(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::default().mesh());
    let mat = mats.add(Color::srgb(0.9, 0.9, 0.2));
    commands.spawn((
        SpawnBox,
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::from_xyz(0.0, 8.0, 0.0),
    ));
}

// ───────────────────────────────────────── Sheeple
pub struct SheeplePlugin;
impl Plugin for SheeplePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(States::Game), build_graph)
            .add_systems(
                Update,
                (
                    spawn_sheeple,
                    graft_player,
                    start_animation_once_ready,
                    gravity,
                    walk,
                    check_exit,
                )
                    .run_if(in_state(States::Game)),
            )
            .add_systems(OnExit(States::Game), despawn::<Sheeple>);
    }
}

// Build the animation graph FROM asset-loader handles.
fn build_graph(
    mut commands: Commands,
    models: Res<ModelAssets>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let (graph, nodes) = AnimationGraph::from_clips([
        models.dig.clone(),
        models.blocker.clone(),
        models.idle.clone(),
        models.turn_right.clone(),
        models.turn_left.clone(),
        models.run.clone(),
        models.walk.clone(),
    ]);
    commands.insert_resource(AnimGraph {
        graph: graphs.add(graph),
        nodes,
    });
}

// Spawn a sheeple each timer tick
fn spawn_sheeple(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
    models: Res<ModelAssets>,
    mut score: ResMut<Score>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    commands.spawn((
        Sheeple,
        SceneRoot(models.joe_scene.clone()),
        Transform {
            translation: Vec3::new(0.0, 7.5, 0.0),
            rotation: Quat::from_rotation_y(90_f32.to_radians()),
            scale: Vec3::ONE,
        },
        GlobalTransform::default(),
        Kinematic {
            vel: Vec3::ZERO,
            on_ground: false,
        },
    ));
    score.spawned += 1;
}

#[derive(Component)]
struct AnimationStartup {
    node: AnimationNodeIndex,
}

fn start_animation_once_ready(
    mut commands: Commands,
    mut players: Query<(
        Entity,
        &mut AnimationPlayer,
        &AnimationStartup,
        &mut AnimationTransitions,
    )>,
) {
    for (entity, mut player, startup, mut transitions) in &mut players {
        debug!("Starting animation for entity: {:?}", entity);
        transitions
            .play(&mut player, startup.node, Duration::ZERO)
            .repeat();

        commands.entity(entity).remove::<AnimationStartup>();
    }
}

// When the AnimationPlayer is added, graft the graph + start idle
fn graft_player(
    mut commands: Commands,
    graph: Res<AnimGraph>,
    q: Query<(Entity, &AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, _player) in &q {
        commands
            .entity(entity)
            .insert(AnimationGraphHandle(graph.graph.clone()))
            .insert((
                // Delay animation startup until after graph is attached
                AnimationStartup {
                    node: graph.nodes[5],
                },
                AnimationTransitions::default(),
            ));
    }
}

// ── physics
fn gravity(mut q: Query<(&mut Transform, &mut Kinematic)>, time: Res<Time>) {
    for (mut tf, mut kin) in &mut q {
        if !kin.on_ground {
            kin.vel.y += GRAVITY * time.delta_secs();
        }
        tf.translation += kin.vel * time.delta_secs();
        if tf.translation.y <= 1.0 {
            tf.translation.y = 1.0;
            kin.vel.y = 0.0;
            kin.on_ground = true;
        }
    }
}
fn walk(mut q: Query<(&mut Transform, &Kinematic)>, time: Res<Time>) {
    for (mut tf, kin) in &mut q {
        if kin.on_ground {
            tf.translation.x += WALK_SPEED * time.delta_secs();
        }
    }
}
fn check_exit(
    mut commands: Commands,
    mut score: ResMut<Score>,
    q: Query<(Entity, &Transform), With<Sheeple>>,
) {
    for (e, tf) in &q {
        if tf.translation.x > GROUND_HALF_EXTENT as f32 {
            score.saved += 1;
            commands.entity(e).despawn();
        } else if tf.translation.y < -5.0 {
            score.died += 1;
            commands.entity(e).despawn();
        }
    }
}

// utility
fn despawn<M: Component>(mut commands: Commands, q: Query<Entity, With<M>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}
