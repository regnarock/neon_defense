use bevy::{
    app::{App, Plugin},
    ecs::{
        query::WorldQuery,
        system::{Commands, SystemParam},
    },
    prelude::*,
    sprite::SpriteBundle,
    transform::components::Transform,
};

use crate::{enemy::Enemy, GameState};

pub struct CrystalPlugin;

impl Plugin for CrystalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup);
        app.add_systems(Update, crystal_touched.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component, Debug)]
pub struct Crystal;

// Using marker component instead of event
//   allows "only-once" consumption through the ECS change detection
#[derive(Component, Debug)]
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
    ));
}

#[derive(WorldQuery)]
struct UntouchedCrystalQuery {
    entity: Entity,
    transform: &'static Transform,
    _without: Without<CrystalTouched>,
    _with: With<Crystal>,
}

#[derive(SystemParam)]
pub struct CrystalTouchedParams<'w, 's> {
    commands: Commands<'w, 's>,
    q_crystal: Query<'w, 's, UntouchedCrystalQuery>,
    q_enemies: Query<'w, 's, &'static Transform, (With<Enemy>, Without<Crystal>)>,
}

pub fn crystal_touched(mut params: CrystalTouchedParams) {
    for enemy in params.q_enemies.iter() {
        for crystal in params.q_crystal.iter_mut() {
            if (crystal.transform.translation - enemy.translation).length() < 10. {
                params
                    .commands
                    .entity(crystal.entity)
                    .insert(CrystalTouched);
            }
        }
    }
}
