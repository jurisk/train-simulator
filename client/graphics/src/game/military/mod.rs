use bevy::app::App;
use bevy::prelude::{IntoSystemConfigs, Plugin, Update, in_state};

use crate::states::ClientState;

pub struct MilitaryPlugin;

impl Plugin for MilitaryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            build_military_building_when_mouse_released.run_if(in_state(ClientState::Playing)),
        );
    }
}

fn build_military_building_when_mouse_released() {
    // TODO HIGH: Get military unit type from selected mode and try to build
}
