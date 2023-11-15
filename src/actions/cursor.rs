use bevy::{
    core_pipeline::fxaa::{Fxaa, FxaaPlugin},
    prelude::*,
    window::PrimaryWindow,
};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_cursor)
            .add_systems(Update, my_cursor_system);
    }
}

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
pub struct CursorScreenPos(pub Vec2);

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

fn setup_cursor(mut commands: Commands) {
    // TODO: maybe move to a better place?
    commands.spawn((Camera2dBundle::default(), MainCamera, Fxaa::default()));
    commands.init_resource::<CursorScreenPos>();
}

fn my_cursor_system(
    mut mycoords: ResMut<CursorScreenPos>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mycoords.0 = world_position;
    }
}
