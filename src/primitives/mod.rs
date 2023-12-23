use bevy::prelude::*;

pub mod destructible;
pub mod ecs_extensions;
pub mod movable;
pub mod target;
pub mod view;

pub struct PrimitivesPlugin;

impl Plugin for PrimitivesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(view::ViewPlugin)
            .add_plugins(target::TargetPlugin)
            .add_plugins(destructible::DestructiblePlugin);
    }
}
