use std::f32::consts::FRAC_PI_2;

use bevy::{
    ecs::{query::WorldQuery, system::SystemParam},
    prelude::*,
};

use crate::GameState;

pub struct TargetPlugin;

impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        //app.add_systems(Update, systems);
        app.add_systems(
            Update,
            detect_target_removed.run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component, Debug)]
pub struct Target {
    pub entity: Entity,
    callback_on_despawn: OnTargetDespawned,
}

impl Target {
    pub fn new(entity: Entity, callback: OnTargetDespawned) -> Self {
        Self {
            entity,
            callback_on_despawn: callback,
        }
    }
}

impl Default for Target {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            callback_on_despawn: OnTargetDespawned::DoNothing,
        }
    }
}

// TODO: replace with a system that reacts to an event of target despawned
#[derive(Debug)]
pub enum OnTargetDespawned {
    DoNothing,
    DespawnSelf,
    //Custom(BoxedSystem<In = (Commands)>),
}

#[derive(Component)]
pub struct AutoLookAtTarget;

#[derive(WorldQuery)]
#[world_query(mutable, derive(Debug))]
pub struct SrcTargetQuery<S, T>
where
    S: Component,
    T: Component,
{
    pub global_transform: &'static mut GlobalTransform,
    pub local_transform: &'static mut Transform,
    pub target: &'static Target,
    pub entity: Entity,
    __filter: (With<S>, Without<T>),
}

#[derive(WorldQuery)]
#[world_query(mutable, derive(Debug))]
pub struct SrcWithoutTargetQuery<S, T>
where
    S: Component,
    T: Component,
{
    pub global_transform: &'static GlobalTransform,
    pub local_transform: &'static Transform,
    pub entity: Entity,
    __filter: (With<S>, Without<T>, Without<Target>),
}

#[derive(WorldQuery)]
#[world_query(derive(Debug))]
pub struct TargetQuery<S, T>
where
    S: Component,
    T: Component,
{
    pub transform: &'static Transform,
    pub global_transform: &'static GlobalTransform,
    pub entity: Entity,
    __filter: (With<T>, Without<S>),
}

#[derive(SystemParam)]
pub struct SourceWithTargetAccessor<'w, 's, S, T>
where
    S: Component,
    T: Component,
{
    pub srcs_query: Query<'w, 's, SrcTargetQuery<S, T>>,
    pub targets_query: Query<'w, 's, TargetQuery<S, T>>,
}

#[derive(SystemParam)]
pub struct SourceWithoutTargetAccessor<'w, 's, S, T>
where
    S: Component,
    T: Component,
{
    pub srcs_query: Query<'w, 's, SrcWithoutTargetQuery<S, T>>,
    pub targets_query: Query<'w, 's, TargetQuery<S, T>>,
}

// TODO: change genetic const parameter to configuration through a resource
pub fn face_target<S, T, const PI_2_OFFSET: usize>(mut params: SourceWithTargetAccessor<S, T>)
where
    S: Component,
    T: Component,
{
    for mut source in params.srcs_query.iter_mut() {
        if let Ok(target) = params.targets_query.get(source.target.entity) {
            let direction = target.transform.translation - source.global_transform.translation();
            let angle = direction.y.atan2(direction.x) + PI_2_OFFSET as f32 * FRAC_PI_2;
            source.local_transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

pub fn detect_target_removed(mut command: Commands, targets: Query<(&Target, Entity)>) {
    for (target, entity) in targets.iter() {
        if command.get_entity(target.entity).is_none() {
            match target.callback_on_despawn {
                OnTargetDespawned::DoNothing => {}
                OnTargetDespawned::DespawnSelf => {
                    command.entity(entity).despawn_recursive();
                }
            }
            command.entity(entity).remove::<Target>();
        }
    }
}
