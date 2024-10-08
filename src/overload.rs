use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

use crate::window::WindowSize;
use crate::{entities::enemy::EventSpawnedEnemy, entities::turret::EventSpawnedTower, GameState};

pub struct OverloadPlugin;

impl Plugin for OverloadPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OverloadDepleted>();

        app.add_systems(Update, draw_ui);
        app.add_systems(Update, update_overload);
        app.add_systems(Update, react_to_spawned_enemy);
        app.add_systems(Update, spend_overload_on_tower_spawnned);

        app.add_systems(OnEnter(GameState::Playing), setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Overload(0.5f32));
}

/// Basically the HP bar, but it decreases naturally over time
///   and increases when enemies are killed
///   and decreases when towers are built
///   always between 0 and 1
#[derive(Component, Reflect, Debug)]
pub struct Overload(pub f32);

const OVERLOAD_DEPLETED_THRESHOLD: f32 = 0.001;

#[derive(Event)]
pub struct OverloadDepleted;

trait Lerp {
    fn lerp_to(&self, rhs: &Color, gradient: f32) -> Color;
}

impl Lerp for Color {
    fn lerp_to(&self, rhs: &Color, gradient: f32) -> Color {
        let h_self = self.as_hsla();
        let gradient = gradient.clamp(0.0, 1.0);

        h_self.with_h(h_self.h() + (rhs.h() - h_self.h()) * gradient)
    }
}

fn draw_overload_bar(painter: &mut ShapePainter, hp: f32) {
    let total_height = 0.15;
    let total_width = 2.0;
    let gap_for_gauge = 0.02;

    // translate to leave space for the overload bar
    painter.translate(Vec3::Y * (-total_height / 2.0 - gap_for_gauge));

    painter.corner_radii = Vec4::splat(10.0);

    let thickness = 0.005;
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
    painter.color = Color::RED.lerp_to(&Color::GREEN, hp);

    let min_width: f32 = total_height - (thickness + gap_for_gauge) * 2.0;
    let max_width: f32 = total_width - (gap_for_gauge + thickness) * 2.0;
    let width: f32 = min_width + (max_width - min_width) * hp;

    painter.rect(Vec2::new(width, min_width));
}

fn draw_ui(mut painter: ShapePainter, q_overload: Query<&Overload>, window_size: Res<WindowSize>) {
    let Ok(overload) = q_overload.get_single() else {
        return;
    };
    // translate to the center-top of the screen
    painter.translate(Vec3::Y * window_size.size.y / 2.0);
    painter.scale(Vec3::ONE * 300.0);

    draw_overload_bar(&mut painter, overload.0);
}

fn update_overload(
    time: Res<Time>,
    mut q_overload: Query<&mut Overload>,
    mut event_writer: EventWriter<OverloadDepleted>,
) {
    let Ok(mut overload) = q_overload.get_single_mut() else {
        return;
    };
    //dbg!(&overload);
    overload.0 = (overload.0 - 0.03 * time.delta_seconds()).clamp(0.0, 1.0);
    if overload.0 < OVERLOAD_DEPLETED_THRESHOLD {
        event_writer.send(OverloadDepleted);
    }
}

fn react_to_spawned_enemy(
    mut event: EventReader<EventSpawnedEnemy>,
    mut q_overload: Query<&mut Overload>,
) {
    let Ok(mut overload) = q_overload.get_single_mut() else {
        return;
    };
    for _e in event.read() {
        overload.0 = (overload.0 + 0.1).clamp(0.0, 1.0);
    }
}

fn spend_overload_on_tower_spawnned(
    mut event: EventReader<EventSpawnedTower>,
    mut q_overload: Query<&mut Overload>,
) {
    let Ok(mut overload) = q_overload.get_single_mut() else {
        return;
    };
    for _e in event.read() {
        overload.0 = (overload.0 - 0.1).clamp(0.0, 1.0);
    }
}
