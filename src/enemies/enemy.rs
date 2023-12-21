use crate::{
    grid::{HexCell, HexGrid},
    primitives::{
        destructible::Destructible,
        movable::AutoMovable,
        target::{
            AutoLookAtTarget, OnTargetDespawned, SourceWithTargetAccessor, SrcWithoutTargetQuery,
            Target,
        },
    },
};

use bevy::{
    ecs::system::{Command, SystemState},
    math::Vec3,
    prelude::*,
    sprite::SpriteBundle,
};
use rand::{seq::SliceRandom, thread_rng};

#[derive(Component)]
pub struct Enemy;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EnemyDef {
    Ship01,
}

#[derive(Event)]
pub struct EventSpawnedEnemy(pub Entity);

#[derive(Resource)]
pub struct EnemyAnimation(Vec<Handle<Image>>);

pub struct SpawnEnemy {
    pub position: Vec2,
    pub enemy_def: EnemyDef,
}

impl Command for SpawnEnemy {
    fn apply(self, world: &mut World) {
        match self.enemy_def {
            EnemyDef::Ship01 => {
                let mut textures: Vec<Handle<Image>> = Vec::new();
                // TODO: make this part of loading step and load dynamically
                for i in 0..9 {
                    let path = format!("textures/Ship_01/AnimIdle/ship01P000{}.png", i);
                    world.resource_scope(|_world, asset_server: Mut<AssetServer>| {
                        textures.push(asset_server.load(path));
                    });
                }

                let spawned_enemy = world
                    .spawn((
                        SpriteBundle {
                            transform: Transform::from_xyz(self.position.x, self.position.y, 0.0)
                                .with_scale(Vec3::new(1.8, 1.8, 1.)),
                            texture: textures[0].clone(),
                            ..Default::default()
                        },
                        Destructible {
                            health: 20.,
                            hitbox: 10.,
                        },
                        AutoMovable {
                            // FIXME: when velocity is too high, the enemy can go through the target
                            velocity: 20.,
                            follow_grid: true,
                        },
                        Enemy,
                    ))
                    .id();

                world.insert_resource(EnemyAnimation(textures));

                let mut q_event: SystemState<EventWriter<EventSpawnedEnemy>> =
                    SystemState::new(world);

                let mut event_writer = q_event.get_mut(world);
                event_writer.send(EventSpawnedEnemy(spawned_enemy));
            }
            _ => panic!("Unknown enemy def"),
        }
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

const TARGET_REACHED_EPSILON: f32 = 1.5;

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
