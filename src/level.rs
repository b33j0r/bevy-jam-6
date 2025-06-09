// levels.rs
use crate::camera::CameraFollowTarget;
use crate::states::States;
use crate::utility::despawn;
use avian3d::prelude::*;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use image::GenericImageView;

const BLOCK_SIZE: f32 = 0.5; // size of ground/lava blocks

// ───────────────────────────────────────────── components
#[derive(Component)]
#[require(ActiveCollisionHooks::MODIFY_CONTACTS)]
pub struct Ground;

#[derive(Component)]
pub struct Lava;

#[derive(Component)]
pub struct SpawnBox;

#[derive(Component)]
pub struct Exit;

// ───────────────────────────────────────────── resources
/// Which level are we on? (1-based index)
#[derive(Resource)]
struct CurrentLevel(usize);

impl Default for CurrentLevel {
    fn default() -> Self {
        CurrentLevel(1)
    }
}

/// How many levels to cycle through
const NUM_LEVELS: usize = 3;

// ───────────────────────────────────────────── plugin
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentLevel>()
            .add_systems(OnEnter(States::Game), (spawn_level, spawn_light))
            .add_systems(
                OnExit(States::Game),
                (
                    despawn::<Ground>,
                    despawn::<Lava>,
                    despawn::<SpawnBox>,
                    despawn::<Exit>,
                    despawn::<DirectionalLight>,
                ),
            );
    }
}

// ───────────────────────────────────────────── load & spawn
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

fn spawn_level(
    mut commands: Commands,
    mut level: ResMut<CurrentLevel>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    // 1) Load PNG
    let idx = level.0;
    let path = format!("assets/levels/level{}.png", idx);
    let img = image::open(&path)
        .unwrap_or_else(|e| panic!("Failed to load {}: {}", path, e))
        .to_rgba8();
    let (w, h) = img.dimensions();

    let middle = Vec3::new(
        (w as f32 * BLOCK_SIZE) / 2.0,
        (h as f32 * BLOCK_SIZE) / 2.0,
        0.0,
    );
    commands.insert_resource(CameraFollowTarget(Some(middle)));

    // 2) Find entrance pixel
    let mut entrance_px: Option<(u32, u32)> = None;
    for (x, y, pixel) in img.enumerate_pixels() {
        if pixel.0 == [0, 0, 0, 255] {
            entrance_px = Some((x, y));
            break;
        }
    }
    let (entr_x, entr_y) = entrance_px.expect("No entrance (black pixel) found in level PNG");
    let entrance = Vec3::new(
        (entr_x as f32 + 0.5) * BLOCK_SIZE,
        (h - entr_y) as f32 * BLOCK_SIZE + 0.5 * BLOCK_SIZE,
        0.0,
    );

    // compute world offset so entrance → (0,0)
    let off_x = 0.0;
    let off_y = 0.0;

    // 3) Prepare mesh & materials
    let cube = meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE).mesh());
    let mat_gnd = mats.add(Color::srgb(0.2, 0.8, 0.2));
    let mat_lava = mats.add(Color::srgb(0.8, 0.2, 0.2));
    let mat_spawn = mats.add(Color::srgb(0.9, 0.9, 0.2));
    let mat_exit = mats.add(Color::srgb(0.2, 0.9, 0.9));

    // 4) Spawn everything, shifted so entrance is at (0,0,0)
    for (x, y, pixel) in img.enumerate_pixels() {
        let [r, g, b, a] = pixel.0;
        // world coords before offset
        let wx = (x as f32 + 0.5) * BLOCK_SIZE;
        let wy = (h - y) as f32 * BLOCK_SIZE + 0.5 * BLOCK_SIZE;

        // apply offset
        let px = wx - off_x;
        let py_base = wy - off_y;

        match (r, g, b, a) {
            (0, 255, 0, 255) => {
                // ground block
                commands.spawn((
                    Ground,
                    RigidBody::Static,
                    Collider::cuboid(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE),
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(mat_gnd.clone()),
                    Transform::from_xyz(px, py_base, 0.0),
                ));
            }
            (255, 0, 0, 255) => {
                // lava block
                commands.spawn((
                    Lava,
                    RigidBody::Static,
                    Collider::cuboid(0.5, 0.5, 0.5),
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(mat_lava.clone()),
                    Transform::from_xyz(px, py_base, 0.0),
                ));
            }
            (0, 0, 0, 255) => {
                // entrance spawn box (2×2)
                commands.spawn((
                    SpawnBox,
                    Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0).mesh())),
                    MeshMaterial3d(mat_spawn.clone()),
                    // entrance originally at wy+1.0, so base y = (wy+1.0) - off_y:
                    Transform::from_xyz(px, (wy + 0.5) - off_y, 0.0),
                ));
            }
            (255, 255, 255, 255) => {
                // exit box
                commands.spawn((
                    Exit,
                    Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0).mesh())),
                    MeshMaterial3d(mat_exit.clone()),
                    Transform::from_xyz(px, (wy + 0.5) - off_y, 0.0),
                ));
            }
            _ => {} // ignore
        }
    }

    // 5) advance level index (wrap)
    level.0 = if idx < NUM_LEVELS { idx + 1 } else { 1 };
}
