use crate::level::domain::Height;
use crate::level::terrain::land::domain::TerrainType::{Grass, Rocks, Sand};

#[repr(u32)]
pub(crate) enum TerrainType {
    Sand  = 0,
    Grass = 1,
    Rocks = 2,
}

impl TerrainType {
    pub(crate) fn from_height(height: Height) -> Self {
        if height.0 <= 9 {
            Sand
        } else if height.0 <= 15 {
            Grass
        } else {
            Rocks
        }
    }
}
