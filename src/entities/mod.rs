use bevy::prelude::*;
use bullet::BulletPlugin;
use crystal::CrystalPlugin;
use enemy::EnemyPlugin;
use portal::PortalsPlugin;
use turret::TurretPlugin;

pub mod bullet;
pub mod crystal;
pub mod enemy;
pub mod portal;
pub mod turret;

pub(super) struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BulletPlugin,
            CrystalPlugin,
            EnemyPlugin,
            PortalsPlugin,
            TurretPlugin,
        ));
    }
}
