use bevy::app::App;
use bevy::prelude::{Component, Plugin};

use crate::cameras::orthographic::OrthographicCameraPlugin;
use crate::cameras::perspective::PerspectiveCameraPlugin;

mod orthographic;
mod perspective;

const ORTHOGRAPHIC_PROJECTION: bool = false;

pub(crate) struct CameraPlugin;

// TODO: Camera switching
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        if ORTHOGRAPHIC_PROJECTION {
            app.add_plugins(OrthographicCameraPlugin);
        } else {
            app.add_plugins(PerspectiveCameraPlugin);
        }
    }
}

#[derive(Component, Default)]
struct ControllableCamera {}
