use std::{f32::consts::FRAC_PI_2, time::Duration};

use crate::{
    buildings::{self, BuildingInventory},
    entities::{bullet::SpawnBullet, enemy::Enemy},
    grid::{HexCell, HexGrid},
    primitives::{
        target::{SourceWithTargetAccessor, Target},
        view::{
            auto_remove_target_when_out_of_range, scan_for_targets_in_range, EnterViewEvent, View,
        },
    },
    GameState,
};
use bevy::{
    ecs::system::{EntityCommand, SystemState},
    math::Vec3,
    prelude::*,
    sprite::SpriteBundle,
};
use bevy_easings::{Ease, EaseFunction};

pub(super) struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(buildings::BuildingsPlugin);
        app.add_event::<EventSpawnedTower>();
        app.add_systems(
            Update,
            (
                scan_for_targets_in_range::<Turret, Enemy>,
                auto_remove_target_when_out_of_range::<Turret, Enemy>,
                process_enemy_enter_range,
                process_enemy_exit_range,
                animate_targeting,
                auto_fire,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Turret;

#[derive(Event)]
pub struct EventSpawnedTower(pub Entity);

#[derive(Component)]
pub struct AutoGun {
    next_shot: Timer,
}

impl AutoGun {
    pub fn new(fire_rate: f32) -> Self {
        let mut next_shot = Timer::from_seconds(fire_rate, TimerMode::Repeating);
        next_shot.pause();

        Self { next_shot }
    }
}

pub struct SpawnTurretCmd {
    pub parent_hex: Entity,
}

impl EntityCommand for SpawnTurretCmd {
    fn apply(self, id: Entity, world: &mut World) {
        // TODO: attach building to the turret
        let _building =
            world.resource_scope(|world, mut building_inventory: Mut<BuildingInventory>| {
                building_inventory.next(world)
            });

        let texture = world.resource_scope(|_, asset_server: Mut<AssetServer>| {
            asset_server.load("textures/DifferentTurrets/Turret01.png")
        });
        let hex_grid = world.resource::<HexGrid>();
        let hex_radius: f32 = hex_grid.layout.hex_size.length();
        let spawned_turret = world
            .entity_mut(id)
            .insert((
                SpriteBundle {
                    transform: Transform::from_scale(Vec3::new(0.5, 0.5, 1.)),
                    texture,
                    ..Default::default()
                },
                Turret,
                Name::new("Turret"),
                AutoGun::new(1.),
                View::new(2. * hex_radius),
            ))
            .set_parent(self.parent_hex)
            .id();
        // FIXME: Not sure this is useful since there is bevy::event::Changed<Turret> that could do the same thing?
        let mut q_event: SystemState<EventWriter<EventSpawnedTower>> = SystemState::new(world);

        let mut event_writer = q_event.get_mut(world);
        event_writer.send(EventSpawnedTower(spawned_turret));
    }
}

pub fn animate_targeting(
    mut commands: Commands,
    accessor: SourceWithTargetAccessor<Turret, Enemy>,
) {
    for turret in &accessor.srcs_query {
        if let Ok(enemy) = accessor.targets_query.get(turret.target.entity) {
            let direction = enemy.transform.translation - turret.global_transform.translation();
            // TODO: FRAC_PI_2 is a bit hacky, because the turret asset is rotated by 90 degrees
            let angle = direction.y.atan2(direction.x) + FRAC_PI_2;

            commands
                .entity(turret.entity)
                .insert((turret.local_transform.ease_to(
                    turret
                        .local_transform
                        .with_rotation(Quat::from_rotation_z(angle)),
                    EaseFunction::QuadraticOut,
                    bevy_easings::EasingType::Once {
                        duration: Duration::from_millis(500),
                    },
                ),));
        }
    }
}

pub fn process_enemy_enter_range(
    mut events: EventReader<EnterViewEvent>,
    mut turrets_query: Query<&mut AutoGun, With<Turret>>,
) {
    for event in events.read() {
        if let Ok(mut gun) = turrets_query.get_mut(event.entity) {
            gun.next_shot.unpause();
        }
    }
}

pub fn process_enemy_exit_range(
    mut events: EventReader<EnterViewEvent>,
    mut turrets_query: Query<&mut AutoGun, With<Turret>>,
) {
    for event in events.read() {
        if let Ok(mut gun) = turrets_query.get_mut(event.entity) {
            gun.next_shot.pause();
            gun.next_shot.reset();
        }
    }
}

pub fn auto_fire(
    mut commands: Commands,
    // make sure that the turret has a target and is in view
    mut turrets_query: Query<
        (&Transform, &Target, &mut AutoGun, &Parent),
        (With<Turret>, With<View>),
    >,
    hex_query: Query<&Transform, (Without<Turret>, With<HexCell>)>,
    time: Res<Time>,
) {
    for (global_transform, target, mut gun, parent) in &mut turrets_query {
        if gun.next_shot.tick(time.delta()).just_finished() {
            if let Ok(transform) = hex_query.get(parent.get()) {
                let spaw_bullet = SpawnBullet {
                    position: transform.translation,
                    velocity: 200.,
                    damage: 1.,
                    target: target.entity,
                };
                commands.add(spaw_bullet);
            }
        }
    }
}
