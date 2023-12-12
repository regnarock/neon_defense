use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;
use std::f32::consts::TAU;

pub struct OverloadPlugin;

impl Plugin for OverloadPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Shape2dPlugin::default());
        app.add_systems(Update, draw_ui);
    }
}

pub struct Overload(pub f32);

trait Pastel {
    fn pastel(&self) -> Color;
}

impl Pastel for Color {
    fn pastel(&self) -> Color {
        (*self + Color::WHITE * 0.25).with_a(1.0)
    }
}

fn draw_ui(time: Res<Time>, mut painter: ShapePainter) {
    let start_pos = painter.transform;
    let diag_vec = Vec3::X + Vec3::Y;
    painter.scale(Vec3::ONE * 30.0);

    painter.hollow = true;
    painter.cap = Cap::Round;
    painter.thickness = 0.5;
    painter.color = Color::WHITE.pastel() * 1.4;
    //painter.translate(-diag_vec * 1.6);
    let angle: f32 = TAU * (1. / 3.);
    painter.arc(0.9, -angle, angle);
    painter.color = Color::CRIMSON.pastel() * 1.4;
    painter.thickness = 0.3;
    painter.arc(0.8, -angle, angle * (time.elapsed_seconds() * 3f32).sin());
}
