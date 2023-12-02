mod actions;
mod audio;
mod board;
mod bullet;
mod crystal;
mod enemy;
mod grid;
mod loading;
mod menu;
mod primitives;
mod turret;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::enemy::EnemyPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::turret::TurretPlugin;

use actions::cursor::CursorPlugin;
use bevy::app::App;
use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use bullet::BulletPlugin;
use crystal::CrystalPlugin;
use grid::GridPlugin;
use primitives::{destructible::DestructiblePlugin, target::TargetPlugin};

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Menu,
    Playing,
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
            GridPlugin,
            DestructiblePlugin,
            // TODO: remove and replace usage with bevy_mod_picking::PickingPlugin
            CursorPlugin,
            DefaultPickingPlugins,
            TargetPlugin,
            CrystalPlugin,
        ));

        #[cfg(debug_assertions)]
        {
            //app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}
