#![allow(clippy::module_name_repetitions)]

use std::f32::consts::PI;
use std::ops::{Add, Div, Mul};

use bevy_math::{Quat, Vec3};
use shared_math::ode_solver::rk4_method;
use shared_math::search::bisection_search_for_minimum;

use crate::constants::{AIR_DENSITY, GRAVITY_VECTOR, METERS_PER_INCH};
use crate::{Angle, Distance, Speed, Time};

#[derive(Debug, Clone)]
pub struct ProjectileProperties {
    pub drag_coefficient: f32,
    pub cross_section:    f32,
    pub mass:             f32,
    pub diameter:         f32,
    pub height:           f32,
    pub start_speed:      Speed,
}

impl ProjectileProperties {
    #[must_use]
    pub const fn create_from_inches(
        diameter_in_inches: f32,
        mass: f32,
        height_in_meters: f32,
        start_speed_in_mps: f32,
    ) -> Self {
        let diameter_in_meters = diameter_in_inches * METERS_PER_INCH;
        let radius = diameter_in_meters / 2.0;
        Self {
            // From http://www.navweaps.com/index_tech/tech-073.php
            drag_coefficient: 0.3,
            cross_section: (radius * radius) * PI,
            mass,
            diameter: diameter_in_meters,
            height: height_in_meters,
            start_speed: start_speed_in_mps,
        }
    }
}

#[must_use]
pub fn best_effort_start_velocity_vector_given_start_velocity(
    from_position: Vec3,
    target_position: Vec3,
    projectile: &ProjectileProperties,
) -> Option<(Vec3, Time)> {
    const MIN_ELEVATION_IN_DEGREES: f32 = 0.;
    const MAX_ELEVATION_IN_DEGREES: f32 = 45.;

    find_angle_and_flight_time_that_hits_target(
        from_position,
        target_position,
        projectile.start_speed,
        projectile,
        MIN_ELEVATION_IN_DEGREES.to_radians(),
        MAX_ELEVATION_IN_DEGREES.to_radians(),
    )
    .map(|(elevation_angle, time)| {
        let v = velocity_vector_from_position_to_angle_with_start_speed_at_elevation_angle(
            from_position,
            target_position,
            projectile.start_speed,
            elevation_angle,
        );
        (v, time)
    })
}

#[must_use]
pub fn velocity_vector_from_position_to_angle_with_start_speed_at_elevation_angle(
    from_position: Vec3,
    target_position: Vec3,
    start_speed: Speed,
    elevation_angle: Angle,
) -> Vec3 {
    let direction = target_position - from_position;
    let velocity = direction.normalize() * start_speed;

    // From ChatGPT, and if we try to shoot straight up then we could get the wrong results as the cross product will fail
    let rotation_axis = direction.cross(Vec3::Y).normalize();
    let rotation = Quat::from_axis_angle(rotation_axis, elevation_angle);

    let result = rotation * velocity;

    debug_assert!((result.length() - start_speed).abs() < 0.001);

    result
}

#[derive(Copy, Clone, Debug)]
struct PositionAndVelocity {
    position: Vec3,
    velocity: Vec3,
}

impl Add<VelocityAndAcceleration> for PositionAndVelocity {
    type Output = Self;

    fn add(self, rhs: VelocityAndAcceleration) -> Self::Output {
        Self {
            position: self.position + rhs.velocity,
            velocity: self.velocity + rhs.acceleration,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct VelocityAndAcceleration {
    velocity:     Vec3,
    acceleration: Vec3,
}

impl Add for VelocityAndAcceleration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            velocity:     self.velocity + rhs.velocity,
            acceleration: self.acceleration + rhs.acceleration,
        }
    }
}

impl Mul<f32> for VelocityAndAcceleration {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            velocity:     self.velocity * rhs,
            acceleration: self.acceleration * rhs,
        }
    }
}

impl Div<f32> for VelocityAndAcceleration {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        self * (1. / rhs)
    }
}

#[must_use]
pub fn find_distance_and_flight_time_to_target_assuming_angle(
    from_position: Vec3,
    target_position: Vec3,
    start_speed: Speed,
    projectile: &ProjectileProperties,
    angle: Angle,
) -> (Distance, Time) {
    const DT: f32 = 0.25;

    let velocity = velocity_vector_from_position_to_angle_with_start_speed_at_elevation_angle(
        from_position,
        target_position,
        start_speed,
        angle,
    );

    let start = PositionAndVelocity {
        position: from_position,
        velocity,
    };

    let (result, t) = rk4_method(
        0.0,
        start,
        |_t, state| {
            VelocityAndAcceleration {
                velocity:     state.velocity,
                acceleration: calculate_acceleration(state.velocity, projectile),
            }
        },
        DT,
        |_t, state| {
            // Going down and below target Y plane
            state.velocity.y < 0. && state.position.y < target_position.y
        },
    );

    // Later: I think this is too imprecise as we return the position after we have crossed below the target's Y plane

    ((result.position - target_position).length(), t)
}

#[must_use]
pub fn find_angle_and_flight_time_that_hits_target(
    from_position: Vec3,
    target_position: Vec3,
    start_speed: Speed,
    projectile: &ProjectileProperties,
    min_elevation: Angle,
    max_elevation: Angle,
) -> Option<(Angle, Time)> {
    // TODO: Also return distance to target, and then check it if it is close enough and return None if not

    const EPS: Angle = 0.001;
    let (angle, times) = bisection_search_for_minimum(min_elevation, max_elevation, EPS, |angle| {
        find_distance_and_flight_time_to_target_assuming_angle(
            from_position,
            target_position,
            start_speed,
            projectile,
            angle,
        )
    });

    match times {
        (Some(a), Some(b)) => Some((angle, (a + b) / 2.0)),
        _ => None,
    }
}

#[expect(non_snake_case)]
#[must_use]
fn calculate_drag_acceleration_vector(velocity: Vec3, projectile: &ProjectileProperties) -> Vec3 {
    let drag_F_magnitude_in_N = 0.5
        * projectile.drag_coefficient // dimensionless
        * AIR_DENSITY // kg/m^3
        * velocity.length_squared() // (m/s)^2
        * projectile.cross_section; // m^2

    let drag_acceleration = drag_F_magnitude_in_N / projectile.mass; // Acceleration due to drag
    -velocity.normalize() * drag_acceleration
}

#[must_use]
pub fn calculate_acceleration(velocity: Vec3, projectile: &ProjectileProperties) -> Vec3 {
    let drag = calculate_drag_acceleration_vector(velocity, projectile);
    GRAVITY_VECTOR + drag
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_distance_to_target_assuming_angle() {
        // Define input parameters
        let from_position = Vec3::new(0.0, 0.0, 0.0);
        let target_position = Vec3::new(12.0, 10.0, 30.0);
        let start_speed = 100.0;
        let projectile = ProjectileProperties::create_from_inches(16.0, 1225.0, 1.829, 762.0);
        let angle = 30f32.to_radians();

        let (distance, time) = find_distance_and_flight_time_to_target_assuming_angle(
            from_position,
            target_position,
            start_speed,
            &projectile,
            angle,
        );

        println!("{distance} {time}");
        assert!(distance >= 975.0);
        assert!(distance < 980.0);
        assert!(time >= 14.9);
        assert!(time <= 15.1);
    }
}
