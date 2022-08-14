use bevy::prelude::*;
use std::f32::consts::PI;

pub fn direction(transform: &Transform) -> (f32, f32) {
    let rot = transform.rotation;
    let rot = if rot.z >= 0.0 && rot.w > 0.0 {
        rot.angle_between(Quat::IDENTITY)
    } else {
        2.0 * PI - rot.angle_between(Quat::IDENTITY)
    };
    let (x, y) = rot.sin_cos();
    (-x, y)
}
