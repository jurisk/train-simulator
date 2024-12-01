use std::fmt::Debug;
use std::ops::{Add, Div, Mul};

use num_traits::One;

// Later: Infinite simulations are dangerous in case of bugs - introduce a limit

#[expect(non_snake_case)]
/// <https://en.wikipedia.org/wiki/Runge%E2%80%93Kutta_methods#The_Runge%E2%80%93Kutta_method>
pub fn rk4_method<
    T: Copy
        + Clone
        + PartialOrd
        + Add<Output = T>
        + Debug
        + Mul<T, Output = T>
        + Div<T, Output = T>
        + One,
    Y: Add<YPrim, Output = Y> + Copy + Clone + Debug,
    YPrim: Div<T, Output = YPrim> + Mul<T, Output = YPrim> + Add<Output = YPrim> + Copy + Clone,
    F,
    CF,
>(
    t_0: T,
    y_t_0: Y,
    y_prim: F,
    dt: T,
    should_stop: CF,
) -> Y
where
    F: Fn(T, Y) -> YPrim,
    CF: Fn(T, Y) -> bool,
{
    let TWO: T = T::one() + T::one();
    let SIX: T = TWO + TWO + TWO;

    let mut t_n = t_0;
    let mut y_n = y_t_0;

    loop {
        let k1 = y_prim(t_n, y_n);
        let k2 = y_prim(t_n + dt / TWO, y_n + (k1 / TWO) * dt);
        let k3 = y_prim(t_n + dt / TWO, y_n + (k2 / TWO) * dt);
        let k4 = y_prim(t_n + dt, y_n + k3 * dt);

        let slope = (k1 + k2 * TWO + k3 * TWO + k4) / SIX;

        y_n = y_n + slope * dt;
        t_n = t_n + dt;

        if should_stop(t_n, y_n) {
            break;
        }
    }

    y_n
}

/// <https://en.wikipedia.org/wiki/Euler_method>
pub fn euler_method<
    T: PartialOrd + Add<Output = T> + Copy + Clone + Debug,
    Y: Add<YPrim, Output = Y> + Copy + Clone + Debug,
    YPrim: Copy + Clone + Mul<T, Output = YPrim>,
    F,
    CF,
>(
    t_0: T,
    y_t_0: Y,
    y_prim: F,
    dt: T,
    should_stop: CF,
) -> Y
where
    F: Fn(T, Y) -> YPrim,
    CF: Fn(T, Y) -> bool,
{
    let mut t_n = t_0;
    let mut y_n = y_t_0;

    loop {
        let slope = y_prim(t_n, y_n);

        y_n = y_n + slope * dt;
        t_n = t_n + dt;

        if should_stop(t_n, y_n) {
            break;
        }
    }

    y_n
}

#[cfg(test)]
mod tests {
    use std::ops::{Add, Div, Mul};

    use crate::ode_solver::{euler_method, rk4_method};

    #[test]
    fn test_rk4() {
        let tests = vec![1., 0.25, 0.1, 0.05, 0.025, 0.0125];

        for step in tests {
            let result = rk4_method(0., 1., |_t, y| y, step, |t, _y| t >= 4.);
            println!("{step} {result}");
            assert!(result > 53.0);
            assert!(result < 58.0);
        }
    }

    // https://en.wikipedia.org/wiki/Euler_method
    #[test]
    fn test_euler_method_from_wikipedia() {
        // Actually, does not match the tests in Wikipedia for a few of the data values, but this is probably an error there
        let tests: Vec<(f32, f32)> = vec![
            (1., 16.),
            (0.25, 35.527_138),
            (0.1, 49.78518),
            (0.05, 52.039_516),
            (0.025, 51.97786),
            (0.0125, 53.261_127),
        ];

        for (step, expected_result) in tests {
            let actual_result = euler_method(0., 1., |_t, y| y, step, |t, _y| t >= 4.);
            assert!((actual_result - expected_result).abs() < 0.001);
        }
    }

    #[derive(Debug, PartialEq, Copy, Clone)]
    struct Vec3 {
        x: f32,
        y: f32,
        z: f32,
    }

    const GRAVITY_VECTOR: Vec3 = Vec3 {
        x: 0.0,
        y: -10.0,
        z: 0.0,
    };

    const START_POSITION: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    const START_VELOCITY: Vec3 = Vec3 {
        x: 1.0,
        y: 50.0,
        z: 2.0,
    };

    impl Add for Vec3 {
        type Output = Vec3;

        fn add(self, rhs: Self) -> Self::Output {
            Self {
                x: self.x + rhs.x,
                y: self.y + rhs.y,
                z: self.z + rhs.z,
            }
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    struct PositionAndVelocity {
        position: Vec3,
        velocity: Vec3,
    }

    impl PositionAndVelocity {
        fn derive(self) -> VelocityAndAcceleration {
            VelocityAndAcceleration {
                velocity:     self.velocity,
                acceleration: GRAVITY_VECTOR,
            }
        }
    }

    impl Mul<f32> for Vec3 {
        type Output = Vec3;

        fn mul(self, rhs: f32) -> Self::Output {
            Self {
                x: self.x * rhs,
                y: self.y * rhs,
                z: self.z * rhs,
            }
        }
    }

    impl Div<f32> for Vec3 {
        type Output = Vec3;

        fn div(self, rhs: f32) -> Self::Output {
            self * (1.0 / rhs)
        }
    }

    impl Add<VelocityAndAcceleration> for PositionAndVelocity {
        type Output = PositionAndVelocity;

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
        type Output = VelocityAndAcceleration;

        fn add(self, rhs: Self) -> Self::Output {
            Self {
                velocity:     self.velocity + rhs.velocity,
                acceleration: self.acceleration + rhs.acceleration,
            }
        }
    }

    impl Mul<f32> for VelocityAndAcceleration {
        type Output = VelocityAndAcceleration;

        fn mul(self, rhs: f32) -> Self::Output {
            Self {
                velocity:     self.velocity * rhs,
                acceleration: self.acceleration * rhs,
            }
        }
    }

    impl Div<f32> for VelocityAndAcceleration {
        type Output = VelocityAndAcceleration;

        fn div(self, rhs: f32) -> Self::Output {
            Self {
                velocity:     self.velocity / rhs,
                acceleration: self.acceleration / rhs,
            }
        }
    }

    const EXPECTED_RESULT: PositionAndVelocity = PositionAndVelocity {
        position: Vec3 {
            x: 10.0,
            y: 0.0,
            z: 20.0,
        },
        velocity: Vec3 {
            x: 1.0,
            y: -50.0,
            z: 2.0,
        },
    };

    fn compare_vec3(actual: Vec3, expected: Vec3, epsilon: f32) {
        assert!((actual.x - expected.x).abs() < epsilon);
        assert!((actual.y - expected.y).abs() < epsilon);
        assert!((actual.z - expected.z).abs() < epsilon);
    }

    #[test]
    fn test_euler_with_vec3() {
        let result = euler_method(
            0f32,
            PositionAndVelocity {
                position: START_POSITION,
                velocity: START_VELOCITY,
            },
            |_t, p_and_v| p_and_v.derive(),
            0.01,
            |_, y| y.position.y < 0.0,
        );

        println!("{result:?}");

        compare_vec3(result.position, EXPECTED_RESULT.position, 0.75);
        compare_vec3(result.velocity, EXPECTED_RESULT.velocity, 0.75);
    }

    #[test]
    fn test_rk4_with_vec3() {
        let result = rk4_method(
            0f32,
            PositionAndVelocity {
                position: START_POSITION,
                velocity: START_VELOCITY,
            },
            |_t, p_and_v| p_and_v.derive(),
            0.01,
            |_, y| y.position.y < 0.0,
        );

        println!("{result:?}");

        compare_vec3(result.position, EXPECTED_RESULT.position, 0.75);
        compare_vec3(result.velocity, EXPECTED_RESULT.velocity, 0.75);
    }
}
