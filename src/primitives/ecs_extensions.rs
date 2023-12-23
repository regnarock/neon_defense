use bevy::prelude::*;

pub fn despawn_entities<T: Component>(
    mut commands: Commands,
    q_entites_to_despawn: Query<Entity, With<T>>,
) {
    for e in q_entites_to_despawn.iter() {
        commands.entity(e).despawn();
    }
}
