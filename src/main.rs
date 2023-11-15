// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use bevy::DefaultPlugins;
use bevy_easings::EasingsPlugin;
use bevy_game::GamePlugin; // ToDo: Replace bevy_game with your new crate name.

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(1., 1., 1.)))
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Neon Defense".to_string(),
                        resolution: (1200., 900.).into(),
                        // Bind to canvas included in `index.html`
                        canvas: Some("#bevy".to_owned()),
                        // Tells wasm not to override default event handling, like F5 and Ctrl+R
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    // FIXME: change to dev only
                    watch_for_changes_override: Some(true),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            EasingsPlugin,
        ))
        .add_plugins(GamePlugin)
        .run();
}
