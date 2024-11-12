use std::cmp::Ordering;
use std::iter::Sum;
use std::ops::{Add, Div, Mul};

use pathfinding::num_traits::Zero;

use crate::game_time::GameTimeDiff;
use crate::transport::transport_velocity::TransportVelocity;

#[derive(Clone, Copy, Debug)]
pub struct TrackLength(f32);

impl TrackLength {
    #[must_use]
    pub const fn new(length: f32) -> Self {
        Self(length)
    }

    #[must_use]
    pub const fn to_f32(self) -> f32 {
        self.0
    }
}

// s / v = t
impl Div<TransportVelocity> for TrackLength {
    type Output = GameTimeDiff;

    fn div(self, rhs: TransportVelocity) -> Self::Output {
        GameTimeDiff::from_seconds(self.to_f32() / rhs.tiles_per_second())
    }
}

impl Div<TrackLength> for TrackLength {
    type Output = f32;

    fn div(self, rhs: TrackLength) -> Self::Output {
        self.0 / rhs.0
    }
}

impl Add<Self> for TrackLength {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Zero for TrackLength {
    fn zero() -> Self {
        Self(0.0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
}

impl Eq for TrackLength {}

impl PartialEq<Self> for TrackLength {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd<Self> for TrackLength {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[expect(clippy::unwrap_used)]
impl Ord for TrackLength {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl Mul<f32> for TrackLength {
    type Output = TrackLength;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Sum for TrackLength {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(TrackLength::zero(), Add::add)
    }
}
