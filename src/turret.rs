use std::{f32::consts::FRAC_PI_2, time::Duration};

use crate::{
    buildings::{self},
    bullet::SpawnBullet,
    enemy::Enemy,
    grid::HexGrid,
    primitives::{
        target::{SourceWithTargetAccessor, Target},
        view::{
            auto_remove_target_when_out_of_range, scan_for_targets_in_range, EnterViewEvent, View,
        },
    },
    random::RandomDeterministic,
    GameState,
};
use bevy::{
    ecs::system::{EntityCommand, SystemState},
    math::Vec3,
    prelude::*,
    sprite::SpriteBundle,
};
use bevy_easings::{Ease, EaseFunction};

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(buildings::Plugin);
        app.add_systems(
            Update,
            (
                scan_for_targets_in_range::<Turret, Enemy>,
                auto_remove_target_when_out_of_range::<Turret, Enemy>,
                process_enemy_enter_range,
                process_enemy_exit_range,
                animate_targeting,
                //auto_fire,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Turret {
    pub parent_hex: Entity,
}

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

pub struct SpawnTurret {
    pub position: Vec2,
    pub at_hex: Entity,
}

type InventoryState<'w, 'q> = (
    Query<
        'w,
        'q,
        (
            &'w mut RandomDeterministic,
            &'w mut crate::inventory::Inventory<buildings::Building>,
        ),
    >,
    Query<'w, 'q, &'w buildings::Building>,
);

impl EntityCommand for SpawnTurret {
    fn apply(self, id: Entity, world: &mut World) {
        let mut state: SystemState<InventoryState> = SystemState::new(world);

        let mut new_item = || {
            let (mut q_inventory, q_items) = state.get_mut(world);

            let (mut rng, mut inventory) = q_inventory.single_mut();

            let Some(first_item) = inventory.items.front().cloned() else {
                return None;
            };
            let Ok(_item_to_build) = q_items.get(first_item) else {
                return None;
            };
            // TODO: check if we can build item_to_build (cooldown, space available, currency, ...)
            // TODO: send an event if not possible.
            // TODO: pay "price" ?
            inventory.items.pop_front();

            let new_item = buildings::get_random_building(&mut rng);
            let _new_item = world.spawn(new_item).id();
            Some((first_item, first_item))
        };

        let Some((item_built, new_item)) = new_item() else {
            return;
        };
        // TODO: reuse that entity to merge it with turret entity ?
        world.despawn(item_built);
        let (mut q_inventory, _q_items) = state.get_mut(world);
        let (_rng, mut inventory) = q_inventory.single_mut();

        inventory.items.push_back(new_item);

        let texture = world.resource_scope(|_, asset_server: Mut<AssetServer>| {
            asset_server.load("textures/DifferentTurrets/Turret01.png")
        });
        let hex_grid = world.resource::<HexGrid>();
        let hex_radius: f32 = hex_grid.layout.hex_size.length();
        world.entity_mut(id).insert((
            SpriteBundle {
                transform: Transform::from_xyz(self.position.x, self.position.y, 0.)
                    .with_scale(Vec3::new(0.5, 0.5, 1.)),
                texture,
                ..Default::default()
            },
            Turret {
                parent_hex: self.at_hex,
            },
            Name::new("Turret"),
            AutoGun::new(1.),
            View::new(2. * hex_radius),
        ));
    }
}

pub fn animate_targeting(
    mut commands: Commands,
    accessor: SourceWithTargetAccessor<Turret, Enemy>,
) {
    for turret in &accessor.srcs_query {
        if let Ok(enemy) = accessor.targets_query.get(turret.target.entity) {
            let direction = enemy.transform.translation - turret.transform.translation;
            // TODO: FRAC_PI_2 is a bit hacky, because the turret asset is rotated by 90 degrees
            let angle = direction.y.atan2(direction.x) + FRAC_PI_2;

            commands
                .entity(turret.entity)
                .insert((turret.transform.ease_to(
                    turret.transform.with_rotation(Quat::from_rotation_z(angle)),
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
    mut turrets_query: Query<(&Transform, &Target, &mut AutoGun), (With<Turret>, With<View>)>,
    time: Res<Time>,
) {
    for (transform, target, mut gun) in &mut turrets_query {
        if gun.next_shot.tick(time.delta()).just_finished() {
            let spaw_bullet = SpawnBullet {
                position: transform.translation,
                velocity: 200.,
                damage: 10.,
                target: target.entity,
            };
            commands.add(spaw_bullet);
        }
    }
}
