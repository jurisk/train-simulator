use serde::{Deserialize, Serialize};

use crate::{BuildingInfo, BuildingType, PlayerId, TileCoordsXZ, TrackType};

// Later: Refactor to store also as a `FieldXZ` so that lookup by tile is efficient
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    fn buildings_at(&self, tile: TileCoordsXZ) -> Vec<&BuildingInfo> {
        self.buildings
            .iter()
            .filter(|building| building.covers_tiles.contains(tile))
            .collect()
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<BuildingInfo> {
        self.buildings.clone()
    }

    // TODO HIGH: Make `private` after refactoring that allows this
    pub fn append_all(&mut self, additional: Vec<BuildingInfo>) {
        self.buildings.extend(additional);
    }

    #[allow(clippy::missing_errors_doc, clippy::result_unit_err)]
    pub fn build(
        &mut self,
        requesting_player_id: PlayerId,
        building_infos: Vec<BuildingInfo>,
    ) -> Result<(), ()> {
        let valid_player_id = building_infos
            .iter()
            .all(|building_info| building_info.owner_id == requesting_player_id);

        // TODO: Check that this is a valid building and there is enough money to build it, subtract money
        // TODO: Check that terrain matches building requirements - e.g. no building on water, tracks that go out of bounds, tracks that go into water, etc.

        let tiles_are_free = building_infos.iter().all(|building_infos| {
            building_infos
                .covers_tiles
                .to_set()
                .into_iter()
                .all(|tile| {
                    // Later: Actually, we should allow adding a track to tracks if such a track type are not already present!
                    self.buildings_at(tile).is_empty()
                })
        });

        if valid_player_id && tiles_are_free {
            self.append_all(building_infos);
            Ok(())
        } else {
            Err(())
        }
    }
}
