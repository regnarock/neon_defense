use bevy::{ecs::query::WorldQuery, prelude::*};

use crate::crystal::CrystalTouched;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, detect_end_of_game);
    }
}

// FIXME: There seem to be a bug in the change detection system.
//  When using `Query<GameOverQuery>` instead of `Query<Entity, Added<CrystalTouched>>`, which should be equivalent,
// the system is instead triggered each tick as if we had used `Query<GameOverQuery, Has<CrystalTouched>>`
#[derive(WorldQuery)]
pub struct GameOverQuery {
    entity: Entity,
    _crystal_is_dead: Added<CrystalTouched>,
}

pub fn detect_end_of_game(_command: Commands, q_game_over: Query<Entity, Added<CrystalTouched>>) {
    if !q_game_over.is_empty() {
        info!("Game over: Crystal touched");
    }
}
