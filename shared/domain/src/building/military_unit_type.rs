use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

// Later: Troops? Trenches? Tanks? Ships? Airplanes?
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub enum MilitaryUnitType {
    FixedArtillery,
    MovableArtillery,
}

impl Debug for MilitaryUnitType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MilitaryUnitType::FixedArtillery => write!(f, "FixedArtillery"),
            MilitaryUnitType::MovableArtillery => write!(f, "MovableArtillery"),
        }
    }
}

impl MilitaryUnitType {
    #[must_use]
    pub fn all() -> Vec<MilitaryUnitType> {
        vec![
            MilitaryUnitType::FixedArtillery,
            MilitaryUnitType::MovableArtillery,
        ]
    }
}
