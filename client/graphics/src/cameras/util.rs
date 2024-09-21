use bevy::input::ButtonInput;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::math::{Dir3, Vec3};
use bevy::prelude::{EventReader, KeyCode, Mat3, Mut, Res, Transform, info, trace, warn};

use crate::hud::PointerOverHud;

fn zx_movement(keyboard_input: &Res<ButtonInput<KeyCode>>, transform: &Transform) -> Vec3 {
    let zx_direction = zx_direction(keyboard_input);

    if zx_direction == Vec3::ZERO {
        Vec3::ZERO
    } else {
        let forward = flatten_in_y_plane(transform.forward());
        let right = flatten_in_y_plane(transform.right());

        (forward * zx_direction.z + right * zx_direction.x).normalize()
    }
}

fn zx_direction(keyboard_input: &Res<ButtonInput<KeyCode>>) -> Vec3 {
    let mut direction = Vec3::ZERO;

    // Forward
    if keyboard_input.pressed(KeyCode::KeyE) {
        direction.z += 1.0;
    }
    // Left
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.x -= 1.0;
    }
    // Backward
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.z -= 1.0;
    }
    // Right
    if keyboard_input.pressed(KeyCode::KeyF) {
        direction.x += 1.0;
    }

    direction
}

pub(crate) fn zoom_value(
    keyboard_input: &Res<ButtonInput<KeyCode>>,
    mouse_wheel: &mut EventReader<MouseWheel>,
    pointer_over_hud: &Res<PointerOverHud>,
) -> f32 {
    let mut result: f32 = 0.0;

    // Zoom out
    if keyboard_input.pressed(KeyCode::KeyA) {
        result += 1.0;
    }
    // Zoom in
    if keyboard_input.pressed(KeyCode::KeyZ) {
        result -= 1.0;
    }

    for ev in mouse_wheel.read() {
        info!("Mouse zoom: {ev:?} {result}"); // TODO: This results in very weird and choppy scrolling sometimes, needs to be rethought.
        if pointer_over_hud.get() {
            trace!("Ignoring as we are over Egui area.");
        } else {
            match ev.unit {
                MouseScrollUnit::Line => {
                    const MOUSE_SCROLL_UNIT_LINE_COEF: f32 = 100.0;
                    // Later: This results in jumping motion. We have to use camera smoothing.
                    result += ev.y * MOUSE_SCROLL_UNIT_LINE_COEF;
                },
                MouseScrollUnit::Pixel => {
                    warn!(
                        "Scroll (pixel units): vertical: {}, horizontal: {}",
                        ev.y, ev.x
                    );
                    // Later: Learn to handle those
                },
            }
        }
    }

    result
}

fn rotation_value(keyboard_input: &Res<ButtonInput<KeyCode>>) -> f32 {
    let mut result: f32 = 0.0;

    // Rotate left
    if keyboard_input.pressed(KeyCode::KeyW) {
        result += 1.0;
    }
    // Rotate right
    if keyboard_input.pressed(KeyCode::KeyR) {
        result -= 1.0;
    }

    result
}

fn flatten_in_y_plane(direction: Dir3) -> Vec3 {
    let mut result = *direction;
    result.y = 0.0;
    result.normalize()
}

// The speed of the camera movement is dependent on the height of the camera, to avoid
// scrolling too quickly when we are very zoomed in.
// TODO HIGH: When zoomed in it moves too fast, when zoomed out it moves too slow
fn camera_movement_speed(transform: &Transform) -> f32 {
    // Later: This could be improved as it still doesn't feel right.
    const CAMERA_MOVEMENT_SPEED: f32 = 4_000.0;
    CAMERA_MOVEMENT_SPEED / transform.translation.y
}

pub(crate) fn movement_and_rotation(
    delta: f32,
    keyboard_input: &Res<ButtonInput<KeyCode>>,
    transform: &mut Mut<Transform>,
) {
    let zx_movement = zx_movement(keyboard_input, transform);
    if zx_movement != Vec3::ZERO {
        let diff = zx_movement * camera_movement_speed(transform) * delta;
        transform.translation += diff;
    }

    let rotation_value = rotation_value(keyboard_input);
    if rotation_value != 0.0 {
        const CAMERA_ROTATION_SPEED: f32 = 1.0;

        let forward = transform.forward();
        let camera_position = transform.translation;
        let t = -camera_position.y / forward.y;

        // POI - "Point of interest" - the point around which we do the rotation
        let poi = camera_position + forward * t;

        // Calculate the new position around the POI
        let rotation = rotation_value * CAMERA_ROTATION_SPEED * delta;

        let relative_position = transform.translation - poi;
        let rotation_matrix = Mat3::from_rotation_y(rotation);
        let new_relative_position = rotation_matrix * relative_position;
        transform.translation = poi + new_relative_position;

        // Adjust the rotation to look at the same POI
        let new_forward = poi - transform.translation;
        let new_forward_normalized = new_forward.normalize();
        let translation = transform.translation;
        transform.look_at(translation + new_forward_normalized, Vec3::Y);
    }
}
