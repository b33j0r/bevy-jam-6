use crate::assets::ModelAssets;
use crate::level::{Ground, SpawnBox};
use crate::states::States;
use crate::{utility, AnimGraph, Score, SpawnTimer, WALK_SPEED};
use avian3d::prelude::{
    Collider, CollisionHooks, ContactPair, LinearVelocity, LockedAxes, PhysicsSet, RigidBody,
};
use bevy::animation::AnimationPlayer;
use bevy::app::{App, Plugin, Update};
use bevy::asset::Assets;
use bevy::ecs::children;
use bevy::ecs::system::SystemParam;
use bevy::log::{debug, info};
use bevy::math::{Quat, Vec3};
use bevy::prelude::*;
use std::time::Duration;

// ───────────────────────────────────────── Sheeple
#[derive(Component)]
struct Sheeple;

pub struct SheeplePlugin;

impl Plugin for SheeplePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(States::Game), build_graph)
            .add_systems(
                Update,
                clear_on_ground
                    .run_if(in_state(States::Game))
                    .before(PhysicsSet::StepSimulation),
            )
            .add_systems(
                Update,
                (
                    spawn_sheeple,
                    graft_player,
                    start_animation_once_ready,
                    walk,
                    check_exit,
                )
                    .run_if(in_state(States::Game)),
            )
            .add_systems(OnExit(States::Game), utility::despawn::<Sheeple>);
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
    query: Query<&Transform, With<SpawnBox>>,
) -> Result<(), BevyError> {
    if !timer.0.tick(time.delta()).just_finished() {
        return Ok(());
    }
    let spawn_box_tf = query.single()?;

    commands.spawn((
        Sheeple,
        Transform {
            translation: spawn_box_tf.translation,
            rotation: Quat::from_rotation_y(90_f32.to_radians()),
            scale: Vec3::ONE,
        },
        Collider::capsule(0.25, 1.0),
        GlobalTransform::default(),
        RigidBody::Dynamic,
        LockedAxes::new().lock_rotation_x().lock_rotation_z(),
        children![(
            SceneRoot(models.joe_scene.clone()),
            Transform::from_xyz(0.0, -0.75, 0.0)
        )],
    ));
    score.spawned += 1;
    Ok(())
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

#[derive(Component)]
struct OnGround;

#[derive(SystemParam)]
pub struct GroundDetection<'w, 's> {
    sheeple_query: Query<'w, 's, Entity, With<Sheeple>>,
    ground_query: Query<'w, 's, Entity, With<Ground>>,
}

impl CollisionHooks for GroundDetection<'_, '_> {
    fn modify_contacts(&self, contacts: &mut ContactPair, commands: &mut Commands) -> bool {
        let (sheep, ground) = if self.sheeple_query.get(contacts.collider1).is_ok()
            && self.ground_query.get(contacts.collider2).is_ok()
        {
            (contacts.collider1, contacts.collider2)
        } else if self.sheeple_query.get(contacts.collider2).is_ok()
            && self.ground_query.get(contacts.collider1).is_ok()
        {
            (contacts.collider2, contacts.collider1)
        } else {
            return true;
        };

        // Insert OnGround marker
        commands.entity(sheep).insert(OnGround);
        true
    }
}

fn clear_on_ground(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LinearVelocity), With<OnGround>>,
) {
    for (e, mut vel) in &mut query {
        commands.entity(e).remove::<OnGround>();
        vel.x = -30.0;
    }
}

fn walk(mut query: Query<(&mut LinearVelocity, &OnGround), With<Sheeple>>) {
    for (mut vel, _) in &mut query {
        vel.x = WALK_SPEED;
    }
}

fn check_exit(
    mut commands: Commands,
    mut score: ResMut<Score>,
    q: Query<(Entity, &Transform), With<Sheeple>>,
) {
    for (e, tf) in &q {
        if tf.translation.y < -10.0 {
            info!("Sheeple fell off the map, despawning: {:?}", e);
            commands.entity(e).despawn();
            score.died += 1;
        }
    }
}
