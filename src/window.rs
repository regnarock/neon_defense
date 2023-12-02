use bevy::{
    app::{Plugin, Update},
    ecs::{
        event::EventReader,
        system::{ResMut, Resource},
    },
    log::info,
    math::Vec2,
    render::extract_resource::ExtractResource,
    window::WindowResized,
};

pub struct GameWindowPlugin;

pub const DEFAULT_WINDOW_SIZE: Vec2 = Vec2::new(1200.0, 900.0);

impl Plugin for GameWindowPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(WindowSize {
            size: DEFAULT_WINDOW_SIZE,
        })
        .add_systems(Update, on_resize_window);
    }
}

#[derive(Resource, ExtractResource, Clone)]
pub struct WindowSize {
    pub size: Vec2,
}

fn on_resize_window(
    mut window_size: ResMut<WindowSize>,
    mut resize_reader: EventReader<WindowResized>,
) {
    for e in resize_reader.read() {
        info!("Window resized to {}x{}", e.width, e.height);
        window_size.size = Vec2::new(e.width, e.height);
    }
}
