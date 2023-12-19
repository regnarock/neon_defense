use bevy::{
    app::{App, Plugin},
    ecs::system::Commands,
    prelude::*,
    sprite::SpriteBundle,
    transform::components::Transform,
};
use bevy_mod_picking::picking_core::Pickable;

use crate::GameState;

pub struct CrystalPlugin;

impl Plugin for CrystalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup);
    }
}

#[derive(Component, Debug)]
pub struct Crystal;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(0.5, 0.5, 1.)),
            texture: asset_server.load("textures/RandomBuildings/B10.png"),
            ..Default::default()
        },
        Crystal,
        Name::new("Crystal"),
        Pickable::IGNORE,
    ));
}
