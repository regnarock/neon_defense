use bevy::prelude::*;

use crate::{entities::crystal::CrystalTouched, overload::OverloadDepleted};

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, detect_end_of_game);
    }
}

pub fn detect_end_of_game(
    _command: Commands,
    mut crystal_touched: EventReader<CrystalTouched>,
    mut overload_depleted: EventReader<OverloadDepleted>,
) {
    for _ in overload_depleted.read() {
        info!("Game over: Overload depleted");
    }
    for _ in crystal_touched.read() {
        info!("Game over: Crystal touched");
    }
}
