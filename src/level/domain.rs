use bevy::prelude::Resource;

#[derive(Debug)]
pub struct Height(pub u8);

pub struct Terrain {
    pub size_x:     usize,
    pub size_z:     usize,
    pub height_map: Vec<Vec<Height>>,
}

impl Terrain {
    fn new(heights: Vec<Vec<u8>>) -> Terrain {
        let size_z = heights.len();
        let size_x = heights[0].len();

        for (row_idx, row) in heights.iter().enumerate() {
            assert_eq!(
                row.len(),
                size_x,
                "Invalid row length {} for row {row_idx}, should be {size_x}",
                row.len()
            );
        }

        let height_map = heights
            .into_iter()
            .map(|row| row.into_iter().map(Height).collect())
            .collect();

        Self {
            size_x,
            size_z,
            height_map,
        }
    }
}

pub struct Water {
    pub between: (Height, Height),
}

impl Water {
    fn new(above: u8, below: u8) -> Self {
        assert_eq!(
            above + 1,
            below,
            "Incorrect above {above:?} and below {below:?} - difference should be 1"
        );
        Self {
            between: (Height(above), Height(below)),
        }
    }
}

#[derive(Resource)]
pub struct Level {
    pub terrain: Terrain,
    pub water:   Water,
}

impl Level {
    pub(crate) fn new(terrain_heights: Vec<Vec<u8>>, water_above: u8, water_below: u8) -> Self {
        let terrain = Terrain::new(terrain_heights);
        let water = Water::new(water_above, water_below);

        Self { terrain, water }
    }
}
