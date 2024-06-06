use bevy::input::ButtonInput;
use bevy::prelude::{
    App, Camera, Commands, Component, Entity, KeyCode, Plugin, Query, Res, Update,
};
use bevy_mod_raycast::deferred::RaycastSource;

use crate::cameras::orthographic::OrthographicCameraPlugin;
use crate::cameras::perspective::PerspectiveCameraPlugin;

mod orthographic;
mod perspective;
mod util;

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
        // Not sure if this works with game states correctly, but it can be solved later
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
    mut query: Query<(Entity, &mut Camera, &CameraComponent)>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        if let Some((_entity, _, CameraComponent { id: current_camera })) =
            query.iter().find(|(_, camera, _)| camera.is_active)
        {
            let next_camera = current_camera.next();
            for (entity, mut camera, camera_type_component) in &mut query {
                let is_active = camera_type_component.id == next_camera;
                camera.is_active = is_active;

                // For bevy_mod_raycast
                let mut entity_commands = commands.entity(entity);
                if is_active {
                    entity_commands.insert(RaycastSource::<()>::new_cursor());
                } else {
                    entity_commands.remove::<RaycastSource<()>>();
                }
            }
        }
    }
}
