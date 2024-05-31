use crate::level::domain::Height;
use crate::level::terrain::land::domain::TerrainType::{Grass, Rocks, Sand, SeaBottom};

#[repr(u32)]
pub(crate) enum TerrainType {
    SeaBottom = 0,
    Sand      = 1,
    Grass     = 2,
    Rocks     = 3,
}

impl TerrainType {
    pub(crate) fn from_height(height: Height) -> Self {
        if height.0 <= 7 {
            SeaBottom
        } else if height.0 <= 9 {
            Sand
        } else if height.0 <= 15 {
            Grass
        } else {
            Rocks
        }
    }
}
