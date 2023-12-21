use bevy::{
    ecs::system::Command,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, MaterialMesh2dBundle},
};

use crate::grid::HexMeshResource;

use super::enemy::{EnemyDef, SpawnEnemy};

#[derive(Component)]
pub struct Portal {
    enemy_def: EnemyDef,
    delay_s: f32,
    remaining: usize,
    spawn_timer: Timer,
}

pub struct SpawnPortal {
    pub position: Vec2,
    pub spawn_n_times: usize,
    pub delay_s: f32,
    pub enemy_def: EnemyDef,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct PortalMaterial {
    #[uniform(0)]
    pub color: Vec4,
}

impl Material2d for PortalMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/portal.wgsl".into()
    }
}

impl Command for SpawnPortal {
    fn apply(self, world: &mut World) {
        let mesh = world
            .get_resource::<HexMeshResource>()
            .unwrap()
            .mesh
            .clone();
        let color = Color::rgb(212.0 / 255.0, 0., 1.0);
        let asset = PortalMaterial {
            color: color.into(),
        };
        let material = world.resource_scope(
            |_world: &mut World, mut materials: Mut<Assets<PortalMaterial>>| materials.add(asset),
        );

        world.spawn((
            MaterialMesh2dBundle {
                mesh: mesh.clone().into(),
                material,
                transform: Transform::from_xyz(self.position.x, self.position.y, 0.0),
                ..default()
            },
            Portal {
                delay_s: self.delay_s,
                remaining: self.spawn_n_times,
                spawn_timer: Timer::from_seconds(self.delay_s, TimerMode::Repeating),
                enemy_def: self.enemy_def,
            },
        ));
    }
}

pub fn update_portals(
    mut commands: Commands,
    mut q_portals: Query<(Entity, &Transform, &mut Portal)>,
    time: Res<Time>,
) {
    for (portal_entity, portal_transform, mut portal) in &mut q_portals {
        if portal.remaining > 0 {
            if portal.spawn_timer.tick(time.delta()).just_finished() {
                portal.remaining -= 1;
                commands.add(SpawnEnemy {
                    position: portal_transform.translation.truncate(),
                    enemy_def: portal.enemy_def,
                });
            }
        } else {
            commands.entity(portal_entity).despawn();
        }
    }
}
