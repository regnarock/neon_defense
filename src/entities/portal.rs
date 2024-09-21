use std::time::Duration;

use bevy::{ecs::system::EntityCommand, prelude::*};

use crate::{entities::enemy::SpawnEnemyCmd, loading::TextureAssets, GameState};

pub(super) struct PortalsPlugin;

impl Plugin for PortalsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (update_all_portals).run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Portal {
    // track when to spawn a new enemy
    timer: Timer,
    capacity: u32,
    delay_ms: Duration,
    spawn_pattern: SpawnPattern,
}

pub enum SpawnPattern {
    Slow,
    Immediate,
}

pub fn update_all_portals(
    mut command: Commands,
    mut portals: Query<(&mut Portal, &GlobalTransform, Entity)>,
    time: Res<Time>,
) {
    portals.for_each_mut(|mut portal| {
        portal.0.timer.tick(time.delta());
        if portal.0.timer.just_finished() {
            spawn_enemy(&mut command, portal);
        }
    });
}

fn spawn_enemy(command: &mut Commands, mut portal: (Mut<Portal>, &GlobalTransform, Entity)) {
    command.add(SpawnEnemyCmd {
        position: portal.1.translation().xy(),
    });
    portal.0.capacity -= 1;
    // despawn immediatly the portal if it was the last enemy to spawn
    // TODO: special closing animation?
    if portal.0.capacity < 1 {
        command.entity(portal.2).remove_parent().despawn();
    }
}

pub struct SpawnPortalCmd {
    pub parent_hex: Entity,
}

impl EntityCommand for SpawnPortalCmd {
    fn apply(self, id: Entity, world: &mut World) {
        world.resource_scope(|world, texture_assets: Mut<TextureAssets>| {
            println!("Spawning a new portal");
            let spawn_delay_ms = Duration::from_millis(3000);
            world
                .entity_mut(id)
                .insert((
                    SpriteBundle {
                        transform: Transform::from_scale(Vec3::new(0.5, 0.5, 1.)),
                        texture: texture_assets.portal.clone_weak(),
                        ..Default::default()
                    },
                    Portal {
                        capacity: 2,
                        delay_ms: spawn_delay_ms,
                        timer: Timer::new(spawn_delay_ms, TimerMode::Repeating),
                        spawn_pattern: SpawnPattern::Immediate,
                    },
                    Name::new("Portal"),
                ))
                .set_parent(self.parent_hex);
        });
    }
}

#[derive(Event)]
pub struct EventDespawnedPortal(pub Entity);
