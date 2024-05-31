use bevy::app::{App, Plugin};

use crate::lights::rotating_directional::RotatingDirectionalLightPlugin;

mod rotating_directional;

pub(crate) struct LightsPlugin;

impl Plugin for LightsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RotatingDirectionalLightPlugin);
    }
}
