use bevy_math::Vec3;

pub const GRAVITY: f32 = -9.80665; // Gravitational constant (in m/s^2).
pub const GRAVITY_VECTOR: Vec3 = Vec3::new(0.0, GRAVITY, 0.0);

pub const AIR_DENSITY: f32 = 1.2041; // At standard ambient temperature and pressure (20 °C and 101.325 kPa), from https://en.wikipedia.org/wiki/Density_of_air

pub const METERS_PER_INCH: f32 = 0.0254;
