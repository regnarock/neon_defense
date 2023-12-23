use bevy::math::vec2;
use bevy::prelude::*;
use bevy_easings::{custom_ease_system, CustomComponentEase, EaseFunction, EasingType};
use bevy_vector_shapes::prelude::*;

use crate::window::WindowSize;
use crate::MarkerGameStatePlaying;
use crate::{enemy::EventSpawnedEnemy, turret::EventSpawnedTower, GameState};

pub struct OverloadPlugin;

impl Plugin for OverloadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, custom_ease_system::<OverloadPosition>);
        app.add_systems(Update, draw_ui);
        app.add_systems(Update, update_overload);
        app.add_systems(Update, react_to_spawned_enemy);
        app.add_systems(Update, react_to_spawned_tower);

        app.add_systems(OnEnter(GameState::Playing), setup);
    }
}

#[derive(Component, Default)]
struct OverloadPosition(Vec2);

impl bevy_easings::Lerp for OverloadPosition {
    type Scalar = f32;

    fn lerp(&self, other: &Self, scalar: &Self::Scalar) -> Self {
        OverloadPosition(self.0.lerp(other.0, *scalar))
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Overload(0.5f32),
        OverloadPosition(vec2(0f32, 100f32)),
        OverloadPosition(vec2(0f32, 100f32)).ease_to(
            OverloadPosition(vec2(0f32, 0f32)),
            EaseFunction::QuadraticIn,
            EasingType::Once {
                duration: std::time::Duration::from_secs_f32(0.5f32),
            },
        ),
        MarkerGameStatePlaying,
    ));
}

/// Basically the HP bar, but it decreases naturally over time
///   and increases when enemies are killed
///   and decreases when towers are built
///   always between 0 and 1
#[derive(Component, Reflect, Debug)]
pub struct Overload(pub f32);

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

fn draw_ui(
    mut painter: ShapePainter,
    q_overload: Query<(&Overload, &OverloadPosition)>,
    window_size: Res<WindowSize>,
) {
    let Ok((overload, position)) = q_overload.get_single() else {
        return;
    };
    // translate to the center-top of the screen
    painter.translate(Vec3::Y * window_size.size.y / 2.0);
    info!("{:?}", position.0);
    painter.translate(position.0.extend(0f32));
    painter.scale(Vec3::ONE * 300.0);

    draw_overload_bar(&mut painter, overload.0);
}

fn update_overload(time: Res<Time>, mut q_overload: Query<&mut Overload>) {
    let Ok(mut overload) = q_overload.get_single_mut() else {
        return;
    };
    overload.0 = (overload.0 - 0.03 * time.delta_seconds()).clamp(0.0, 1.0);
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

fn react_to_spawned_tower(
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
