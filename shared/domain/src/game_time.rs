#![allow(clippy::module_name_repetitions)]

use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default, PartialEq)]
pub struct GameTimeDiff(f32);

impl GameTimeDiff {
    #[must_use]
    pub fn from_seconds(seconds: f32) -> Self {
        Self(seconds)
    }

    #[must_use]
    pub fn to_seconds(&self) -> f32 {
        self.0
    }
}

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