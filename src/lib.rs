mod actions;
mod audio;
mod board;
mod buildings;
mod bullet;
mod crystal;
mod enemy;
mod game_over;
mod grid;
mod inventory;
mod loading;
mod menu;
mod overload;
mod primitives;
mod random;
mod turret;
mod window;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::enemy::EnemyPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::overload::OverloadPlugin;
use crate::turret::TurretPlugin;

use actions::cursor::CursorPlugin;
use bevy::app::App;

use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_vector_shapes::Shape2dPlugin;
use bullet::BulletPlugin;
use crystal::CrystalPlugin;
use game_over::GameOverPlugin;
use grid::GridPlugin;
use primitives::PrimitivesPlugin;
use window::GameWindowPlugin;

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
            (
                LoadingPlugin,
                GameWindowPlugin,
                Shape2dPlugin::default(),
                DefaultPickingPlugins,
                InternalAudioPlugin,
            ),
            MenuPlugin,
            ActionsPlugin,
            TurretPlugin,
            EnemyPlugin,
            BulletPlugin,
            GridPlugin,
            // TODO: remove and replace usage with bevy_mod_picking::PickingPlugin
            CursorPlugin,
            CrystalPlugin,
            PrimitivesPlugin,
            OverloadPlugin,
            GameOverPlugin,
        ));

        #[cfg(debug_assertions)]
        {
            //app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}
