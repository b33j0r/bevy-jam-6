use bevy::prelude;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, prelude::States)]
pub enum States {
    #[default]
    AssetLoading,
    Game,
}