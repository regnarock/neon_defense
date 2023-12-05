use bevy::{
    ecs::{query::WorldQuery, system::SystemParam},
    prelude::*,
};

use super::target::{
    OnTargetDespawned, SrcTargetQuery, SrcWithoutTargetQuery, Target, TargetQuery,
};

pub struct ViewPlugin;

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnterViewEvent>()
            .add_event::<ExitViewEvent>()
            .add_systems(Update, debug_range);
    }
}

#[derive(Component, Debug)]
pub struct View {
    range: f32,
}

impl View {
    pub fn new(range: f32) -> Self {
        Self { range }
    }
}

#[derive(Event, Debug)]
pub struct EnterViewEvent {
    pub entity: Entity,
}

#[derive(Event, Debug)]
pub struct ExitViewEvent {
    pub entity: Entity,
}

#[derive(WorldQuery)]
#[world_query(mutable, derive(Debug))]
pub struct SrcViewWithoutTargetQuery<S, T>
where
    S: Component,
    T: Component,
{
    pub subquery: SrcWithoutTargetQuery<S, T>,
    pub view: &'static View,
}

#[derive(SystemParam)]
pub struct SourceViewWithoutTargetAccessor<'w, 's, S, T>
where
    S: Component,
    T: Component,
{
    pub srcs_query: Query<'w, 's, SrcViewWithoutTargetQuery<S, T>>,
    pub targets_query: Query<'w, 's, TargetQuery<S, T>>,
}

#[derive(WorldQuery)]
#[world_query(mutable, derive(Debug))]
pub struct SrcViewTargetQuery<S, T>
where
    S: Component,
    T: Component,
{
    pub subquery: SrcTargetQuery<S, T>,
    pub view: &'static View,
}

#[derive(SystemParam)]
pub struct SourceViewTargetAccessor<'w, 's, S, T>
where
    S: Component,
    T: Component,
{
    pub srcs_query: Query<'w, 's, SrcViewTargetQuery<S, T>>,
    pub targets_query: Query<'w, 's, TargetQuery<S, T>>,
}

pub fn scan_for_targets_in_range<S, T>(
    mut commands: Commands,
    accessor: SourceViewWithoutTargetAccessor<S, T>,
    mut enter_view_events: EventWriter<EnterViewEvent>,
) where
    S: Component,
    T: Component,
{
    for src in &accessor.srcs_query {
        let mut nearest_target = None;
        let mut nearest_distance = src.view.range;

        for target in &accessor.targets_query {
            let distance = src
                .subquery
                .transform
                .translation
                .distance(target.transform.translation);
            if distance < nearest_distance {
                nearest_target = Some(target);
                nearest_distance = distance;
            }
        }
        if let Some(target) = nearest_target {
            // TODO: change OnTargetDespawned to an event
            commands
                .entity(src.subquery.entity)
                .insert((Target::new(target.entity, OnTargetDespawned::DoNothing),));
            enter_view_events.send(EnterViewEvent {
                entity: src.subquery.entity,
            });
        }
    }
}

pub fn auto_remove_target_when_out_of_range<S, T>(
    mut commands: Commands,
    accessor: SourceViewTargetAccessor<S, T>,
    mut exit_view_events: EventWriter<ExitViewEvent>,
) where
    S: Component,
    T: Component,
{
    for src in &accessor.srcs_query {
        if let Ok(target) = accessor.targets_query.get(src.subquery.target.entity) {
            let distance = src
                .subquery
                .transform
                .translation
                .distance(target.transform.translation);
            if distance > src.view.range {
                commands.entity(src.subquery.entity).remove::<Target>();
                exit_view_events.send(ExitViewEvent {
                    entity: src.subquery.entity,
                });
            }
        }
    }
}

pub fn debug_range(mut gizmos: Gizmos, views: Query<(&View, &Transform)>) {
    for (view, transform) in &views {
        gizmos.circle_2d(transform.translation.xy(), view.range, Color::LIME_GREEN);
    }
}
