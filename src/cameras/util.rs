use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::prelude::{Direction3d, KeyCode, Res, Transform};

pub(crate) fn zx_movement(
    keyboard_input: &Res<ButtonInput<KeyCode>>,
    transform: &Transform,
) -> Vec3 {
    let zx_direction = zx_direction(keyboard_input);

    if zx_direction == Vec3::ZERO {
        Vec3::ZERO
    } else {
        let forward = flatten_in_y_plane(transform.forward());
        let right = flatten_in_y_plane(transform.right());

        (forward * zx_direction.z + right * zx_direction.x).normalize()
    }
}

pub(crate) fn zx_direction(keyboard_input: &Res<ButtonInput<KeyCode>>) -> Vec3 {
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

pub(crate) fn zoom_value(keyboard_input: &Res<ButtonInput<KeyCode>>) -> f32 {
    let mut result: f32 = 0.0;

    // Zoom out
    if keyboard_input.pressed(KeyCode::KeyA) {
        result += 1.0;
    }
    // Zoom in
    if keyboard_input.pressed(KeyCode::KeyZ) {
        result -= 1.0;
    }

    result
}

pub(crate) fn rotation_value(keyboard_input: &Res<ButtonInput<KeyCode>>) -> f32 {
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

pub(crate) fn flatten_in_y_plane(direction: Direction3d) -> Vec3 {
    let mut result = *direction;
    result.y = 0.0;
    result.normalize()
}
