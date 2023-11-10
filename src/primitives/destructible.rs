use bevy::prelude::*;

use super::target::Target;

pub struct DestructiblePlugin;

impl Plugin for DestructiblePlugin {
    fn build(&self, app: &mut App) {
        // TODO: run only in game
        app.add_systems(Update, (apply_damage, destroy_if_no_health));
    }
}

#[derive(Component)]
pub struct Destructible {
    pub health: f32,
    pub hitbox: f32,
}

#[derive(Component)]
pub struct Damage(f32);

impl Damage {
    pub fn new(damage: f32) -> Self {
        Self(damage)
    }
}

pub fn apply_damage(
    mut commands: Commands,
    bullets_query: Query<(&Damage, &Transform, &Target, Entity)>,
    mut enemies_query: Query<(&mut Destructible, &Transform)>,
) {
    for (damage, damage_transform, target, dmg_entity) in bullets_query.iter() {
        if let Ok((mut destructible, enemy_transform)) = enemies_query.get_mut(target.entity) {
            let distance = enemy_transform
                .translation
                .distance(damage_transform.translation);
            if distance < 10.0 {
                destructible.health -= damage.0;
                commands.entity(dmg_entity).despawn();
                println!("Enemy health: {}", destructible.health);
            }
        }
    }
}

pub fn destroy_if_no_health(
    mut commands: Commands,
    mut enemies_query: Query<(Entity, &Destructible)>,
) {
    for (entity, destructible) in enemies_query.iter_mut() {
        if destructible.health <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
