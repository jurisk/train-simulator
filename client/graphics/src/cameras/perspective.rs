use bevy::app::{App, Update};
use bevy::core::Name;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseWheel;
use bevy::log::info;
use bevy::math::Vec3;
use bevy::prelude::{
    default, in_state, ButtonInput, Camera, Camera3dBundle, Commands, EventReader,
    IntoSystemConfigs, KeyCode, Plugin, PostUpdate, Query, Res, Startup, Time, Transform,
};
use bevy::render::view::ColorGrading;
use bevy_egui::EguiContexts;

use crate::cameras::util::{movement_and_rotation, zoom_value};
use crate::cameras::{CameraComponent, CameraControlEvent, CameraId};
use crate::constants::UP;
use crate::game::map_level::terrain::land::tiled_mesh_from_height_map_data::Tiles;
use crate::states::ClientState;

pub(crate) struct PerspectiveCameraPlugin;

impl Plugin for PerspectiveCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_camera);
        // Has to be PostUpdate as otherwise Egui areas are not calculated yet
        app.add_systems(PostUpdate, move_camera);
        app.add_systems(
            Update,
            process_camera_control_events.run_if(in_state(ClientState::Playing)),
        );
    }
}

#[expect(clippy::needless_pass_by_value)]
fn process_camera_control_events(
    mut events: EventReader<CameraControlEvent>,
    mut query: Query<(&mut Transform, &CameraComponent, &Camera)>,
    tiles: Res<Tiles>,
) {
    for event in events.read() {
        match event {
            CameraControlEvent::FocusOnTile(tile_coords) => {
                info!("Focusing on tile: {:?}", tile_coords);
                let tiles = tiles.as_ref();
                let tile = &tiles.tiles[*tile_coords];
                let target = tile.quad.average_position();

                // TODO: This is far from ideal as we lost the current rotation and height...
                // TODO: Needs camera smoothing... possibly home-grown.
                let new_transform = transform_from_target(target, 40.0);

                for (mut transform, camera_component, camera) in &mut query {
                    if camera_component.id == CameraId::Perspective && camera.is_active {
                        *transform = new_transform;
                    }
                }
            },
        }
    }
}

fn transform_from_target(target: Vec3, height: f32) -> Transform {
    const ANGLE_COEF: f32 = 0.5;
    let diff = Vec3::new(0f32, height, height * ANGLE_COEF);
    let from = target + diff;
    Transform::from_translation(from).looking_at(target, UP)
}

fn create_camera(mut commands: Commands) {
    let transform = transform_from_target(Vec3::ZERO, 200.0);

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                is_active: false,
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::None,
            color_grading: ColorGrading { ..default() },
            transform,
            ..default()
        },
        CameraComponent {
            id: CameraId::Perspective,
        },
        // RaycastSource::<()>::new_cursor(), // For bevy_mod_raycast
        Name::new("Perspective Camera"),
    ));
}

#[expect(clippy::needless_pass_by_value)]
fn move_camera(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut query: Query<(&mut Transform, &CameraComponent, &Camera)>,
    mut egui_contexts: EguiContexts,
) {
    for (mut transform, camera_component, camera) in &mut query {
        if camera_component.id == CameraId::Perspective && camera.is_active {
            movement_and_rotation(time.delta_seconds(), &keyboard_input, &mut transform);

            // TODO: Consider doing zooming using the FOV, as per https://bevy-cheatbook.github.io/graphics/camera.html
            let zoom_value = zoom_value(&keyboard_input, &mut mouse_wheel, &mut egui_contexts);
            if zoom_value != 0.0 {
                const CAMERA_ZOOM_SPEED: f32 = 80.0;
                let forward = transform.forward();
                transform.translation +=
                    forward * zoom_value * CAMERA_ZOOM_SPEED * time.delta_seconds();
            }
        }
    }
}
