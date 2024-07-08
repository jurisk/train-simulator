use std::fmt::{Debug, Formatter};
use std::ops::Mul;

use serde::{Deserialize, Serialize};

use crate::game_time::GameTimeDiff;
use crate::track_length::TrackLength;

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct TransportVelocity {
    tiles_per_second: f32,
}

// s = v * t
impl Mul<GameTimeDiff> for TransportVelocity {
    type Output = TrackLength;

    fn mul(self, rhs: GameTimeDiff) -> Self::Output {
        TrackLength::new(self.tiles_per_second * rhs.to_seconds())
    }
}

impl Debug for TransportVelocity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.tiles_per_second)
    }
}

impl TransportVelocity {
    #[must_use]
    pub fn new(tiles_per_second: f32) -> Self {
        Self { tiles_per_second }
    }

    #[must_use]
    pub fn tiles_per_second(self) -> f32 {
        self.tiles_per_second
    }
}
