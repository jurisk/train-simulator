use bevy::app::{App, Plugin};
use bevy::prelude::{AmbientLight, ResMut, Startup};

use crate::lights::rotating_directional::RotatingDirectionalLightPlugin;

mod rotating_directional;

pub(crate) struct LightsPlugin;

impl Plugin for LightsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RotatingDirectionalLightPlugin);
        app.add_systems(Startup, setup_ambient_light);
    }
}

fn setup_ambient_light(mut ambient_light: ResMut<AmbientLight>) {
    ambient_light.brightness = 1_200.0;
}
