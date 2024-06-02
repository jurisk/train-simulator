use serde::{Deserialize, Serialize};

#[repr(u32)]
pub enum TerrainType {
    Sand  = 0,
    Grass = 1,
    Rocks = 2,
}

impl TerrainType {
    #[must_use]
    pub fn default_from_height(height: Height) -> Self {
        if height.0 <= 9 {
            TerrainType::Sand
        } else if height.0 <= 15 {
            TerrainType::Grass
        } else {
            TerrainType::Rocks
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Height(pub u8);

#[derive(Serialize, Deserialize, Clone)]
pub struct Terrain {
    pub vertex_count_x: usize,
    pub vertex_count_z: usize,

    // TODO: Move to GridXZ, which makes the details private
    pub vertex_heights: Vec<Vec<Height>>,
}

impl Terrain {
    #[must_use]
    pub fn tile_count_x(&self) -> usize {
        self.vertex_count_x - 1
    }

    #[must_use]
    pub fn tile_count_z(&self) -> usize {
        self.vertex_count_z - 1
    }

    fn is_valid(&self) -> bool {
        self.vertex_heights.len() == self.vertex_count_z
            && self
                .vertex_heights
                .iter()
                .all(|row| row.len() == self.vertex_count_x)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Water {
    pub between: (Height, Height),
}

impl Water {
    fn is_valid(&self) -> bool {
        let (below, above) = &self.between;
        below.0 + 1 == above.0
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MapLevel {
    pub terrain: Terrain,
    pub water:   Water,
}

impl MapLevel {
    // Could eventually move to some `Validated` instead
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.terrain.is_valid() && self.water.is_valid()
    }
}
