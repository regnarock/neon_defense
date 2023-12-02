use bevy::{
    ecs::{query::WorldQuery, system::SystemParam},
    prelude::*,
};

use crate::primitives::target::{Target, TargetQuery};

#[derive(Component)]
pub struct AutoMovable {
    pub velocity: f32,
    pub follow_grid: bool,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct MovablesWithTargetQuery<M, T>
where
    M: bevy::prelude::Component,
    T: bevy::prelude::Component,
{
    pub transform: &'static mut Transform,
    pub target: &'static Target,
    pub auto_movable: &'static AutoMovable,
    _filter: (With<M>, Without<T>),
}

#[derive(SystemParam)]
pub struct MovableWithTargetAccessor<'w, 's, M, T>
where
    M: Component,
    T: Component,
{
    pub movables: Query<'w, 's, MovablesWithTargetQuery<M, T>>,
    pub targets: Query<'w, 's, TargetQuery<M, T>>,
}

pub fn move_towards_target<M: Component, T: Component>(
    mut accessor: MovableWithTargetAccessor<M, T>,
    time: Res<Time>,
) {
    for mut movable in accessor.movables.iter_mut() {
        if let Ok(target) = accessor.targets.get(movable.target.entity) {
            let direction = (target.transform.translation.xy()
                - movable.transform.translation.xy())
            .normalize();
            let movement =
                (direction * (movable.auto_movable.velocity * time.delta_seconds())).extend(0.0);
            movable.transform.translation += movement;
        } else {
            warn!(
                "{} points to an unexisting target {}",
                std::any::type_name::<M>(),
                std::any::type_name::<T>(),
            );
        }
    }
}
