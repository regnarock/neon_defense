mod actions;
mod audio;
mod board;
mod bullet;
mod enemy;
mod loading;
mod menu;
mod movable;
mod primitives;
mod turret;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::enemy::EnemyPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::movable::MovementPlugin;
use crate::turret::TurretPlugin;

use actions::cursor::CursorPlugin;
use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use board::BoardPlugin;
use bullet::BulletPlugin;
use primitives::{destructible::DestructiblePlugin, target::TargetPlugin};

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>().add_plugins((
            LoadingPlugin,
            MenuPlugin,
            ActionsPlugin,
            InternalAudioPlugin,
            TurretPlugin,
            EnemyPlugin,
            BulletPlugin,
            MovementPlugin,
            DestructiblePlugin,
            // TODO: remove and replace usage with bevy_mod_picking::PickingPlugin
            CursorPlugin,
            BoardPlugin,
            DefaultPickingPlugins,
            TargetPlugin,
        ));

        #[cfg(debug_assertions)]
        {
            //app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}
