use bevy::input::ButtonInput;
use bevy::prelude::{App, Camera, Component, KeyCode, Plugin, Query, Res, Update};

use crate::cameras::orthographic::OrthographicCameraPlugin;
use crate::cameras::perspective::PerspectiveCameraPlugin;

mod orthographic;
mod perspective;

pub(crate) struct CameraPlugin;

#[derive(Default, Eq, PartialEq, Copy, Clone)]
enum CameraId {
    #[default]
    Orthographic,
    Perspective,
}

impl CameraId {
    fn next(self) -> Self {
        match self {
            CameraId::Orthographic => CameraId::Perspective,
            CameraId::Perspective => CameraId::Orthographic,
        }
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OrthographicCameraPlugin);
        app.add_plugins(PerspectiveCameraPlugin);
        app.add_systems(Update, switch_camera);
    }
}

#[derive(Component, Default)]
struct CameraComponent {
    id: CameraId,
}

#[allow(clippy::needless_pass_by_value)]
fn switch_camera(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Camera, &CameraComponent)>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        if let Some((_, CameraComponent { id: current_camera })) =
            query.iter().find(|(camera, _)| camera.is_active)
        {
            let next_camera = current_camera.next();
            for (mut camera, camera_type_component) in &mut query {
                camera.is_active = camera_type_component.id == next_camera;
            }
        }
    }
}
