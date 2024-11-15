use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::ops::{AddAssign, Sub};

use bevy_math::Vec3;
use log::warn;
use serde::{Deserialize, Serialize};

// Later: Consider moving to u16 together with making GameTime discrete
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct ProgressWithinTile(f32);

impl ProgressWithinTile {
    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    pub fn new(progress: f32) -> Self {
        assert!(
            (0.0 ..= 1.0).contains(&progress),
            "Progress must be between 0.0 and 1.0, but was {progress}"
        );
        Self(progress)
    }

    #[must_use]
    pub fn from_point_between_two_points(start_end: (Vec3, Vec3), point: Vec3) -> Self {
        let (start, end) = start_end;
        let value = (point - start).length() / (end - start).length();
        Self::new(value)
    }

    #[must_use]
    pub fn just_entering() -> Self {
        Self(0.0)
    }

    #[must_use]
    pub fn about_to_exit() -> Self {
        Self(1.0)
    }

    #[must_use]
    pub fn as_f32(self) -> f32 {
        self.0
    }
}

impl Debug for ProgressWithinTile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl Eq for ProgressWithinTile {}

impl PartialOrd for ProgressWithinTile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProgressWithinTile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or_else(|| panic!("Failed to compare {self:?} and {other:?}"))
    }
}

impl Sub for ProgressWithinTile {
    type Output = ProgressWithinTile;

    fn sub(self, rhs: Self) -> Self::Output {
        ProgressWithinTile::new(self.0 - rhs.0)
    }
}

impl AddAssign<f32> for ProgressWithinTile {
    fn add_assign(&mut self, rhs: f32) {
        self.0 += rhs;
        if self.0 > 1.0 {
            warn!("Adding {rhs} to ProgressWithinTile made it out of bounds: {self:?}");
        }
    }
}
