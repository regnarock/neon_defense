use std::time::Duration;

use bevy::{prelude::*, sprite::Material2dPlugin, time::common_conditions::on_timer};

use crate::{
    grid::HexCell,
    primitives::{movable::move_towards_target, target::face_target},
    GameState,
};

use self::{
    enemy::{
        animate, move_towards_center, remove_reached_target, Enemy, EnemyAnimation,
        EventSpawnedEnemy,
    },
    portals::{update_portals, PortalMaterial},
};

pub mod enemy;
pub mod portals;

pub struct EnemiesPlugin;

// TODO: make this a config resource
const FIXED_TIMESTEP: f32 = 0.1;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        // Enemy component related systems and resources
        app.add_event::<EventSpawnedEnemy>();
        // enemies updates
        app.add_systems(
            Update,
            (
                face_target::<Enemy, HexCell, 0>,
                move_towards_target::<Enemy, HexCell>,
                move_towards_center,
                remove_reached_target,
            )
                .run_if(in_state(GameState::Playing)),
        )
        // enemies animations
        .add_systems(
            FixedUpdate,
            (animate.run_if(
                on_timer(Duration::from_secs_f32(FIXED_TIMESTEP))
                    .and_then(resource_exists::<EnemyAnimation>()),
            ))
            .run_if(in_state(GameState::Playing)),
        )
        // portals updates
        .add_systems(Update, update_portals);

        // Portal component related systems and resources
        app.add_plugins(Material2dPlugin::<PortalMaterial>::default());
    }
}
