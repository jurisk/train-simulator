use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Height(pub u8);

#[derive(Serialize, Deserialize, Clone)]
pub struct Terrain {
    // size_x and size_z is for vertices, the number of tiles is smaller by 1 in each dimension
    pub size_x:     usize,
    pub size_z:     usize,
    pub height_map: Vec<Vec<Height>>,
}

impl Terrain {
    fn is_valid(&self) -> bool {
        self.height_map.len() == self.size_z
            && self.height_map.iter().all(|row| row.len() == self.size_x)
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
pub struct Level {
    pub terrain: Terrain,
    pub water:   Water,
}

impl Level {
    // Could eventually move to some `Validated` instead
    pub(crate) fn is_valid(&self) -> bool {
        self.terrain.is_valid() && self.water.is_valid()
    }
}