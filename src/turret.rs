use std::{f32::consts::FRAC_PI_2, time::Duration};

use crate::{
    bullet::SpawnBullet,
    enemy::Enemy,
    primitives::target::{OnTargetDespawned, SourceWithoutTargetAccessor, Target},
    GameState,
};
use bevy::{ecs::system::Command, math::Vec3, prelude::*, sprite::SpriteBundle};
use bevy_easings::{Ease, EaseFunction};

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (target_nearest_enemy, auto_fire).run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Turret;

#[derive(Component)]
pub struct AutoGun {
    next_shot: Timer,
    range: f32,
}

impl AutoGun {
    pub fn new(fire_rate: f32, range: f32) -> Self {
        let mut next_shot = Timer::from_seconds(fire_rate, TimerMode::Repeating);
        next_shot.pause();

        Self { next_shot, range }
    }
}

pub struct SpawnTurret {
    pub at_hex: Entity,
}

impl Command for SpawnTurret {
    fn apply(self, world: &mut World) {
        let texture = world.resource_scope(|_, asset_server: Mut<AssetServer>| {
            asset_server.load("textures/DifferentTurrets/Turret01.png")
        });
        world
            .spawn((
                SpriteBundle {
                    transform: Transform::from_scale(Vec3::new(0.5, 0.5, 1.)),
                    texture,
                    ..Default::default()
                },
                Turret,
                Name::new("Turret"),
                //AutoGun::new(1., 400.),
            ))
            .set_parent(self.at_hex);
    }
}

pub fn target_nearest_enemy(
    mut commands: Commands,
    mut accessor: SourceWithoutTargetAccessor<Turret, Enemy>,
) {
    for turret in accessor.srcs_query.iter_mut() {
        let mut nearest_enemy = None;
        let mut nearest_distance = f32::MAX;

        for enemy in accessor.targets_query.iter_mut() {
            let distance = turret
                .transform
                .translation
                .distance(enemy.transform.translation);
            if distance < nearest_distance {
                nearest_enemy = Some(enemy);
                nearest_distance = distance;
            }
        }

        if let Some(enemy) = nearest_enemy {
            // calculate the angle to the enemy and change the rotation of the turret with easing
            let direction = enemy.transform.translation - turret.transform.translation;
            let angle = direction.y.atan2(direction.x) + FRAC_PI_2;

            commands.entity(turret.entity).insert((
                Target::new(enemy.entity, OnTargetDespawned::DoNothing),
                turret.transform.ease_to(
                    turret.transform.with_rotation(Quat::from_rotation_z(angle)),
                    EaseFunction::QuadraticOut,
                    bevy_easings::EasingType::Once {
                        duration: Duration::from_millis(500),
                    },
                ),
            ));
        }
    }
}

pub fn auto_fire(
    mut commands: Commands,
    mut turrets_query: Query<(&Transform, &Target, &mut AutoGun), With<Turret>>,
    targets_query: Query<&Transform, With<Enemy>>,
    time: Res<Time>,
) {
    turrets_query.for_each_mut(|(transform, target, mut gun)| {
        // first, check if the target is in range
        if let Ok(rhs) = targets_query.get(target.entity) {
            let distance = transform.translation.distance(rhs.translation);

            if distance > gun.range {
                gun.next_shot.pause();
            } else {
                let spaw_bullet = SpawnBullet {
                    position: transform.translation,
                    velocity: 200.,
                    damage: 10.,
                    target: target.entity,
                };
                if gun.next_shot.paused() {
                    gun.next_shot.reset();
                    gun.next_shot.unpause();
                    commands.add(spaw_bullet);
                } else if gun.next_shot.tick(time.delta()).just_finished() {
                    commands.add(spaw_bullet);
                }
            }
        }
    });
}
