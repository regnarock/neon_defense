use std::time::Duration;

use crate::{
    movable::{move_towards_target, AutoMovable},
    primitives::{
        destructible::Destructible,
        target::{
            face_target, AutoLookAtTarget, OnTargetDespawned, SourceWithoutTargetAccessor, Target,
        },
    },
    turret::Turret,
    GameState,
};
use bevy::{
    ecs::system::Command, math::Vec3, prelude::*, sprite::SpriteBundle,
    time::common_conditions::on_timer,
};

pub struct EnemyPlugin;

const FIXED_TIMESTEP: f32 = 0.1;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_enemy)
            .add_systems(
                Update,
                (
                    target_turret,
                    face_target::<Enemy, Turret, 0>,
                    move_towards_target::<Enemy, Turret>,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (animate.run_if(
                    on_timer(Duration::from_secs_f32(FIXED_TIMESTEP))
                        .and_then(resource_exists::<EnemyAnimation>()),
                ))
                .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct Enemy;

#[derive(Resource)]
pub struct EnemyAnimation(Vec<Handle<Image>>);

pub struct SpawnEnemy {
    pub position: Vec2,
}

impl Command for SpawnEnemy {
    fn apply(self, world: &mut World) {
        let mut textures: Vec<Handle<Image>> = Vec::new();
        // TODO: make this a resource
        for i in 0..9 {
            let path = format!("textures/Ship_01/AnimIdle/ship01P000{}.png", i);
            world.resource_scope(|_world, asset_server: Mut<AssetServer>| {
                textures.push(asset_server.load(path));
            });
        }

        world.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(self.position.x, self.position.y, 0.0)
                    .with_scale(Vec3::new(1.8, 1.8, 1.)),
                texture: textures[0].clone(),
                ..Default::default()
            },
            Enemy,
            Destructible {
                health: 20.,
                hitbox: 10.,
            },
            AutoMovable { velocity: 20. },
        ));

        world.insert_resource(EnemyAnimation(textures));
    }
}

pub fn target_turret(
    mut commands: Commands,
    mut accessor: SourceWithoutTargetAccessor<Enemy, Turret>,
) {
    accessor.srcs_query.for_each_mut(|enemy| {
        let mut nearest_turret = None;
        let mut nearest_distance = f32::MAX;
        for turret in accessor.targets_query.iter() {
            let distance = enemy
                .transform
                .translation
                .distance(turret.transform.translation);
            if distance < nearest_distance {
                nearest_turret = Some(turret.entity);
                nearest_distance = distance;
            }
        }
        if let Some(turret) = nearest_turret {
            commands.get_entity(enemy.entity).unwrap().insert((
                Target::new(turret, OnTargetDespawned::DoNothing),
                AutoLookAtTarget,
            ));
        }
    });
}

pub fn animate(
    animations: Res<EnemyAnimation>,
    mut enemy_query: Query<&mut Handle<Image>, With<Enemy>>,
    // TODO: move that inside enemy to have a different animation for each enemy
    mut frame_id: Local<usize>,
) {
    enemy_query.for_each_mut(|mut sprite| {
        if *frame_id >= animations.0.len() {
            *sprite = animations.0[animations.0.len() * 2 - *frame_id - 1].clone();
        } else {
            *sprite = animations.0[*frame_id].clone();
        }
        *frame_id = (*frame_id + 1) % (animations.0.len() * 2);
    });
}

pub fn spawn_enemy(mut commands: Commands) {
    commands.add(SpawnEnemy {
        position: Vec2 { x: 500.0, y: -100. },
    });
}
