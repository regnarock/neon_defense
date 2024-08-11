use bevy::{
    app::{App, Plugin},
    ecs::system::Commands,
    prelude::*,
    sprite::SpriteBundle,
    transform::components::Transform,
};
use bevy_mod_picking::picking_core::Pickable;

use crate::{enemy::Enemy, GameState};

pub struct CrystalPlugin;

impl Plugin for CrystalPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CrystalTouched>();

        app.add_systems(OnEnter(GameState::Playing), setup);
        app.add_systems(Update, crystal_touched.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component, Debug)]
pub struct Crystal;

#[derive(Event, Debug)]
pub struct CrystalTouched;

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

pub fn crystal_touched(
    mut crystal_touched: EventWriter<CrystalTouched>,
    q_crystals: Query<&Transform, (With<Crystal>, Without<Enemy>)>,
    q_enemies: Query<&Transform, (With<Enemy>, Without<Crystal>)>,
) {
    for enemy in q_enemies.iter() {
        for crystal in q_crystals.iter() {
            if (crystal.translation - enemy.translation).length() < 10. {
                crystal_touched.send(CrystalTouched);
            }
        }
    }
}
