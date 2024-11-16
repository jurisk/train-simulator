#![allow(clippy::module_name_repetitions)]

use std::ops::{Add, Div, Mul, Sub, SubAssign};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
pub struct GameTimeDiff(f32);

impl GameTimeDiff {
    pub const ZERO: Self = Self(0.0);

    #[must_use]
    pub fn from_seconds(seconds: f32) -> Self {
        Self(seconds)
    }

    #[must_use]
    pub fn to_seconds(&self) -> f32 {
        self.0
    }
}

// Later: Consider making discrete for better predictability and better suitability for AI? But we want smooth movement of transports.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default, PartialEq)]
pub struct GameTime(f32);

impl GameTime {
    #[must_use]
    pub fn new() -> Self {
        Self(0.0)
    }

    #[must_use]
    pub fn from_seconds(seconds: f32) -> Self {
        Self(seconds)
    }
}

impl Add<GameTimeDiff> for GameTimeDiff {
    type Output = GameTimeDiff;

    fn add(self, rhs: GameTimeDiff) -> Self::Output {
        GameTimeDiff(self.0 + rhs.0)
    }
}

impl Add<GameTimeDiff> for GameTime {
    type Output = Self;

    fn add(self, rhs: GameTimeDiff) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for GameTime {
    type Output = GameTimeDiff;

    fn sub(self, rhs: Self) -> Self::Output {
        GameTimeDiff(self.0 - rhs.0)
    }
}

impl Sub for GameTimeDiff {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for GameTimeDiff {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Div<GameTimeDiff> for GameTimeDiff {
    type Output = f32;

    fn div(self, rhs: GameTimeDiff) -> Self::Output {
        self.0 / rhs.0
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub struct TimeFactor(f32);

impl TimeFactor {
    #[must_use]
    pub fn new(time_factor: f32) -> Self {
        Self(time_factor)
    }
}

impl Default for TimeFactor {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Mul<TimeFactor> for GameTimeDiff {
    type Output = GameTimeDiff;

    fn mul(self, rhs: TimeFactor) -> Self::Output {
        GameTimeDiff(self.0 * rhs.0)
    }
}
