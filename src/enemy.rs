use std::time::Duration;

use crate::{
    grid::{HexCell, HexGrid},
    primitives::{
        destructible::Destructible,
        movable::{move_towards_target, AutoMovable},
        target::{
            face_target, AutoLookAtTarget, OnTargetDespawned, SourceWithTargetAccessor,
            SrcWithoutTargetQuery, Target,
        },
    },
    GameState,
};

use bevy::{
    ecs::system::Command, math::Vec3, prelude::*, sprite::SpriteBundle,
    time::common_conditions::on_timer,
};
use rand::{seq::SliceRandom, thread_rng};

pub struct EnemyPlugin;

// TODO: make this a config resource
const FIXED_TIMESTEP: f32 = 0.1;
const TARGET_REACHED_EPSILON: f32 = 1.5;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                face_target::<Enemy, HexCell, 0>,
                move_towards_target::<Enemy, HexCell>,
                move_towards_center,
                remove_reached_target,
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
            AutoMovable {
                velocity: 20.,
                follow_grid: true,
            },
        ));

        world.insert_resource(EnemyAnimation(textures));
    }
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

pub fn move_towards_center(
    mut commands: Commands,
    mut enemies: Query<SrcWithoutTargetQuery<Enemy, HexCell>>,
    hexes: Query<&HexCell>,
    grid: Res<HexGrid>,
    _time: Res<Time>,
) {
    for enemy in &mut enemies {
        let mut all_neighbors = grid
            .layout
            .world_pos_to_hex(enemy.transform.translation.xy())
            .all_neighbors();
        all_neighbors.shuffle(&mut thread_rng());
        let target_position = all_neighbors
            .iter()
            .filter_map(|hex| {
                grid.entities
                    .get(hex)
                    .and_then(|e| hexes.get(*e).ok().map(|cell| (hex, cell.dist)))
            })
            .min_by(|(_, dist1), (_, dist2)| dist1.cmp(dist2));

        if let Some((target_hex, _dist)) = target_position {
            let hex_entity = grid.entities[target_hex];
            commands.entity(enemy.entity).insert((
                Target::new(hex_entity, OnTargetDespawned::DoNothing),
                AutoLookAtTarget,
            ));
        }
    }
}

pub fn remove_reached_target(
    mut commands: Commands,
    accessor: SourceWithTargetAccessor<Enemy, HexCell>,
) {
    for enemy in &accessor.srcs_query {
        if let Ok(target) = accessor.targets_query.get(enemy.target.entity) {
            let distance = target
                .transform
                .translation
                .distance(enemy.transform.translation);
            if distance <= TARGET_REACHED_EPSILON {
                commands.entity(enemy.entity).remove::<Target>();
            }
        }
    }
}
