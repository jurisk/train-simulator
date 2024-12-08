use bevy::input::ButtonInput;
use bevy::prelude::{
    App, Camera, Commands, Component, Entity, Event, KeyCode, Plugin, PostStartup, Query,
    RayCastPickable, Res, Update, info,
};
use shared_domain::tile_coords_xz::TileCoordsXZ;

use crate::cameras::perspective::PerspectiveCameraPlugin;
use crate::cameras::top_down::TopDownCameraPlugin;

mod perspective;
mod top_down;
mod util;

pub(crate) struct CameraPlugin;

#[derive(Eq, PartialEq, Copy, Clone, Debug, Default)]
enum CameraId {
    #[default]
    Perspective,
    TopDown,
}

#[derive(Event)]
pub enum CameraControlEvent {
    FocusOnTile(TileCoordsXZ),
}

impl CameraId {
    fn next(self) -> Self {
        match self {
            CameraId::TopDown => CameraId::Perspective,
            CameraId::Perspective => CameraId::TopDown,
        }
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // Not sure if this works with game states correctly, but it can be solved later
        app.add_plugins(TopDownCameraPlugin);
        app.add_plugins(PerspectiveCameraPlugin);
        app.add_systems(Update, switch_camera);
        app.add_systems(PostStartup, enable_default_camera);
        app.add_event::<CameraControlEvent>();
    }
}

#[derive(Component)]
struct CameraComponent {
    id: CameraId,
}

fn enable_default_camera(
    mut query: Query<(Entity, &mut Camera, &CameraComponent)>,
    mut commands: Commands,
) {
    let camera_id = CameraId::default();
    switch_to_camera(&mut query, &mut commands, camera_id);
}

#[expect(clippy::needless_pass_by_value)]
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
            switch_to_camera(&mut query, &mut commands, next_camera);
        }
    }
}

fn switch_to_camera(
    query: &mut Query<(Entity, &mut Camera, &CameraComponent)>,
    commands: &mut Commands,
    next_camera: CameraId,
) {
    info!("Switching to camera: {next_camera:?}");
    for (entity, mut camera, camera_type_component) in query {
        let is_active = camera_type_component.id == next_camera;
        camera.is_active = is_active;

        let mut entity_commands = commands.entity(entity);
        if is_active {
            entity_commands.insert(RayCastPickable);
        } else {
            entity_commands.remove::<RayCastPickable>();
        }
    }
}
