mod actions;
mod audio;
mod board;
mod buildings;
mod bullet;
mod crystal;
mod enemy;
mod grid;
mod inventory;
mod loading;
mod menu;
mod menu_playing;
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
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_vector_shapes::Shape2dPlugin;
use bullet::BulletPlugin;
use crystal::CrystalPlugin;
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

#[derive(Reflect, Component)]
pub struct MarkerGameStatePlaying;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_plugins((
                LoadingPlugin,
                GameWindowPlugin,
                Shape2dPlugin::default(),
                MenuPlugin,
                ActionsPlugin,
                InternalAudioPlugin,
                TurretPlugin,
                EnemyPlugin,
                BulletPlugin,
                GridPlugin,
                // TODO: remove and replace usage with bevy_mod_picking::PickingPlugin
                CursorPlugin,
                DefaultPickingPlugins,
                CrystalPlugin,
                PrimitivesPlugin,
                OverloadPlugin,
            ))
            .add_plugins((menu_playing::MenuPlayingPlugin, WorldInspectorPlugin::new()))
            .add_systems(
                OnExit(GameState::Playing),
                primitives::ecs_extensions::despawn_entities::<MarkerGameStatePlaying>,
            );

        #[cfg(debug_assertions)]
        {
            //app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}
