use bevy::prelude::{AnimationClip, Gltf, Handle, Resource, Scene};
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct ModelAssets {
    // Load the full GLTF
    #[asset(path = "models/Joe.glb")]
    pub joe_gltf: Handle<Gltf>,

    // Load the scene from the GLTF
    #[asset(path = "models/Joe.glb#Scene0")]
    pub joe_scene: Handle<Scene>,

    #[asset(path = "models/Joe.glb#Animation1")]
    pub blocker: Handle<AnimationClip>,

    #[asset(path = "models/Joe.glb#Animation2")]
    pub dig: Handle<AnimationClip>,

    // Optional label for animations by index
    #[asset(path = "models/Joe.glb#Animation3")]
    pub idle: Handle<AnimationClip>,

    #[asset(path = "models/Joe.glb#Animation4")]
    pub turn_right: Handle<AnimationClip>,

    #[asset(path = "models/Joe.glb#Animation5")]
    pub turn_left: Handle<AnimationClip>,

    #[asset(path = "models/Joe.glb#Animation6")]
    pub run: Handle<AnimationClip>,

    #[asset(path = "models/Joe.glb#Animation7")]
    pub walk: Handle<AnimationClip>,
}
