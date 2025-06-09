//! main.rs — Bevy 0.16-compatible “Sheeple” skeleton
//! Requires bevy = "0.16", bevy_asset_loader = "0.23"

mod assets;
mod camera;
mod level;
mod sheeple;
mod states;
mod utility;

use crate::camera::CameraPlugin;
use crate::level::LevelPlugin;
use assets::ModelAssets;
use avian3d::PhysicsPlugins;
use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_asset_loader::prelude::*;
use sheeple::{GroundDetection, SheeplePlugin};
use states::States;
// ───────────────────────────────────────── constants

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

// ───────────────────────────────────────── main
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default().with_collision_hooks::<GroundDetection>())
        // .add_plugins(PhysicsDebugPlugin::default())
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
