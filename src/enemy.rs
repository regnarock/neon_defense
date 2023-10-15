use bevy::{
    prelude::{
        in_state, App, AssetServer, Assets, Commands, Component, Handle, Image, IntoSystemConfigs,
        OnEnter, Plugin, Query, Res, ResMut, Resource, Transform, Update, With,
    },
    sprite::{ColorMaterial, Sprite, SpriteBundle},
};

use crate::GameState;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_enemy)
            .add_systems(Update, (animate).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct Enemy;

pub fn spawn_enemy(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let textures = (0..8)
        .map(|i| {
            materials.add(ColorMaterial::from(
                asset_server.load(format!("textures/Ship_01/AnimIdle/ship01P000{}.png", i)),
            ))
        })
        .collect::<Vec<_>>();

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(100.0, 100.0, 0.0),
            texture: materials
                .get(&textures[0])
                .unwrap()
                .texture
                .clone()
                .unwrap(),
            ..Default::default()
        },
        Enemy,
    ));

    commands.insert_resource(EnemyAnimation(textures));
}

pub fn animate(
    animations: Res<EnemyAnimation>,
    mut enemy_query: Query<&mut Handle<ColorMaterial>, With<Enemy>>,
) {
    enemy_query.for_each_mut(|mut sprite| {
        *sprite = animations.0[0].clone();
    });
}

#[derive(Resource)]
pub struct EnemyAnimation(Vec<Handle<ColorMaterial>>);
