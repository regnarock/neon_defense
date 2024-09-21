mod actions;
mod audio;
mod buildings;
mod entities;
mod game_over;
mod grid;
mod inventory;
mod loading;
mod menu;
mod overload;
mod primitives;
mod random;
mod window;

use actions::cursor::CursorPlugin;
use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_vector_shapes::Shape2dPlugin;

use actions::ActionsPlugin;
use audio::InternalAudioPlugin;
use entities::EntityPlugin;
use game_over::GameOverPlugin;
use grid::GridPlugin;
use loading::LoadingPlugin;
use menu::MenuPlugin;
use overload::OverloadPlugin;
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
            EntityPlugin,
            GridPlugin,
            // TODO: remove and replace usage with bevy_mod_picking::PickingPlugin
            CursorPlugin,
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
