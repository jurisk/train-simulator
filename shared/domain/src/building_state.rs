use crate::{BuildingInfo, BuildingType, TileCoordsXZ, TrackType};

// Later: Refactor to store also as a `FieldXZ` so that lookup by tile is efficient
#[derive(Debug, Clone)]
pub struct BuildingState {
    buildings: Vec<BuildingInfo>,
}

impl BuildingState {
    #[must_use]
    pub fn empty() -> Self {
        Self::from_vec(vec![])
    }

    #[must_use]
    pub fn from_vec(buildings: Vec<BuildingInfo>) -> Self {
        Self { buildings }
    }

    #[must_use]
    pub fn track_types_at(&self, tile: TileCoordsXZ) -> Vec<TrackType> {
        self.buildings_at(tile)
            .into_iter()
            .filter_map(|building| {
                match building.building_type {
                    BuildingType::Track(track_type) => Some(track_type),
                    BuildingType::Station(_) | BuildingType::Production(_) => None,
                }
            })
            .collect()
    }

    #[must_use]
    pub fn buildings_at(&self, tile: TileCoordsXZ) -> Vec<&BuildingInfo> {
        self.buildings
            .iter()
            .filter(|building| building.covers_tiles.contains(tile))
            .collect()
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<BuildingInfo> {
        self.buildings.clone()
    }

    pub fn append(&mut self, additional: Vec<BuildingInfo>) {
        self.buildings.extend(additional);
    }
}
