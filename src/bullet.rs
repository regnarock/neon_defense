use bevy::{ecs::system::Command, prelude::*};

use crate::{
    enemy::Enemy,
    movable::{move_towards_target, AutoMovable},
    primitives::{
        destructible::Damage,
        target::{face_target, AutoLookAtTarget, OnTargetDespawned, Target},
    },
};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                move_towards_target::<Bullet, Enemy>,
                face_target::<Bullet, Enemy, 3>,
            ),
        );
    }
}

#[derive(Component)]
pub struct Bullet;

// Command to spawn a bullet
pub struct SpawnBullet {
    pub position: Vec3,
    pub velocity: f32,
    pub target: Entity,
    pub damage: f32,
}

impl Command for SpawnBullet {
    fn apply(self, world: &mut World) {
        // TODO: make this a resource
        let image: Handle<Image> =
            world.resource_scope(|_world, asset_server: Mut<AssetServer>| {
                asset_server.load("textures/Bullets/P02.png")
            });

        world.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(self.position.x, self.position.y, 0.0)
                    .with_scale(Vec3::new(0.8, 0.8, 1.)),
                texture: image,
                ..Default::default()
            },
            Bullet,
            Target::new(self.target, OnTargetDespawned::DespawnSelf),
            AutoMovable {
                velocity: self.velocity,
            },
            AutoLookAtTarget,
            Damage::new(self.damage),
        ));
    }
}
