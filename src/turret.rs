use crate::GameState;
use bevy::{
    prelude::{App, AssetServer, Commands, OnEnter, Plugin, Res, Transform},
    sprite::SpriteBundle,
};

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_turret);
    }
}

fn spawn_turret(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(SpriteBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .with_scale(bevy::math::Vec3::new(0.5, 0.5, 1.)),
        texture: asset_server.load("textures/DifferentTurrets/Turret01.png"),
        ..Default::default()
    });
}
