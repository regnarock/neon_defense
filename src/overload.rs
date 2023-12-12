use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;
use std::f32::consts::TAU;

use crate::{enemy::EventSpawnedEnemy, turret::EventSpawnedTower, GameState};

pub struct OverloadPlugin;

impl Plugin for OverloadPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Shape2dPlugin::default());
        app.add_systems(Update, draw_ui);
        app.add_systems(Update, update_overload);
        app.add_systems(Update, react_to_spawned_enemy);
        app.add_systems(Update, react_to_spawned_tower);

        app.add_systems(OnEnter(GameState::Playing), setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Overload(0f32));
}

#[derive(Component, Reflect, Debug)]
pub struct Overload(pub f32);

trait Pastel {
    fn pastel(&self) -> Color;
}

impl Pastel for Color {
    fn pastel(&self) -> Color {
        (*self + Color::WHITE * 0.25).with_a(1.0)
    }
}

fn draw_health_bar(painter: &mut ShapePainter, hp: f32) {
    let total_height = 0.15;
    let total_width = 1.05;
    let gap_for_gauge = 0.02;

    painter.corner_radii = Vec4::splat(10.0);

    let thickness = 0.01;
    painter.thickness = thickness;

    painter.hollow = false;
    painter.color = Color::BLACK;
    painter.rect(Vec2::new(total_width - 0.01, total_height - 0.01));
    painter.rect(Vec2::new(
        total_width + gap_for_gauge * 2.0,
        total_height + gap_for_gauge * 2.0,
    ));

    painter.hollow = true;
    painter.color = Color::WHITE;
    painter.rect(Vec2::new(total_width, total_height));

    painter.hollow = false;
    painter.color = Color::GREEN * hp + Color::RED * (1. - hp);

    let min_width: f32 = total_height - (thickness + gap_for_gauge) * 2.0;
    let max_width: f32 = total_width - (gap_for_gauge + thickness) * 2.0;
    let width: f32 = min_width + (max_width - min_width) * hp;

    painter
        .translate(Vec3::X * ((-total_width + width + (gap_for_gauge + thickness) * 2.0) / 2f32));
    painter.rect(Vec2::new(width, min_width));
}

fn draw_ui(mut painter: ShapePainter, q_overload: Query<&Overload>) {
    let Ok(overload) = q_overload.get_single() else {
        return;
    };
    painter.translate(Vec3::Y * 400f32);
    painter.scale(Vec3::ONE * 300.0);

    draw_health_bar(&mut painter, overload.0);
}

fn update_overload(time: Res<Time>, mut q_overload: Query<&mut Overload>) {
    let Ok(mut overload) = q_overload.get_single_mut() else {
        return;
    };
    //dbg!(&overload);
    overload.0 = (overload.0 + 0.05 * time.delta_seconds()).clamp(0.0, 1.0);
}

fn react_to_spawned_enemy(
    mut event: EventReader<EventSpawnedEnemy>,
    mut q_overload: Query<&mut Overload>,
) {
    let Ok(mut overload) = q_overload.get_single_mut() else {
        return;
    };
    for e in event.read() {
        dbg!("overload--");
        overload.0 = (overload.0 - 0.5).clamp(0.0, 1.0);
    }
}
fn react_to_spawned_tower(
    mut event: EventReader<EventSpawnedTower>,
    mut q_overload: Query<&mut Overload>,
) {
    let Ok(mut overload) = q_overload.get_single_mut() else {
        return;
    };
    for e in event.read() {
        dbg!("overload++");
        overload.0 = (overload.0 + 0.5).clamp(0.0, 1.0);
    }
}
