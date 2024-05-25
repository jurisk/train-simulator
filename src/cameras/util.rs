use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::prelude::{KeyCode, Res};

pub(crate) fn zx_direction(keyboard_input: &Res<ButtonInput<KeyCode>>) -> Vec3 {
    let mut direction = Vec3::ZERO;

    // Forward
    if keyboard_input.pressed(KeyCode::KeyE) {
        direction.z -= 1.0;
    }
    // Left
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.x -= 1.0;
    }
    // Backward
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.z += 1.0;
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

pub (crate) fn rotation_value(keyboard_input: &Res<ButtonInput<KeyCode>>) -> f32 {
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