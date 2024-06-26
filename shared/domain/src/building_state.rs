use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::building_info::BuildingInfo;
use crate::{BuildingType, PlayerId, TileCoordsXZ, TrackType};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum CanBuildResponse {
    Ok, // Add `price` here later?
    AlreadyExists,
    Invalid,
}

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

    pub(crate) fn append_all(&mut self, additional: Vec<BuildingInfo>) {
        self.buildings.extend(additional);
    }

    #[allow(clippy::missing_errors_doc, clippy::result_unit_err)]
    pub fn can_build(
        &mut self,
        requesting_player_id: PlayerId,
        building_infos: &[BuildingInfo],
    ) -> bool {
        let valid_player_id = building_infos
            .iter()
            .all(|building_info| building_info.owner_id == requesting_player_id);

        // TODO: Check that this is a valid building and there is enough money to build it, subtract money

        let tiles_are_free = building_infos.iter().all(|building_info| {
            self.can_build_building(requesting_player_id, building_info) == CanBuildResponse::Ok
        });

        valid_player_id && tiles_are_free
    }

    pub(crate) fn can_build_building(
        &self,
        requesting_player_id: PlayerId,
        building_info: &BuildingInfo,
    ) -> CanBuildResponse {
        // TODO: Check that terrain matches building requirements - e.g. no building on water, tracks that go out of bounds, tracks that go into water, etc.
        // TODO:    Is this even within bounds? Above water?
        // TODO:    Is it free? Is the terrain suitable?

        let overlapping_buildings = building_info
            .covers_tiles
            .to_set()
            .into_iter()
            .flat_map(|tile| self.buildings_at(tile))
            .collect::<Vec<_>>();

        let overlapping_other_players = overlapping_buildings
            .iter()
            .any(|building| building.owner_id != requesting_player_id);

        let overlapping_tracks = overlapping_buildings
            .iter()
            .filter_map(|building| {
                match building.building_type {
                    BuildingType::Track(track_type) => Some(track_type),
                    BuildingType::Station(_) | BuildingType::Production(_) => None,
                }
            })
            .collect::<HashSet<_>>();

        let has_overlapping_non_tracks = overlapping_buildings.iter().any(|building| {
            match building.building_type {
                BuildingType::Track(_) => false,
                BuildingType::Station(_) | BuildingType::Production(_) => true,
            }
        });

        let invalid_overlaps = match building_info.building_type {
            BuildingType::Track(_track_type) => has_overlapping_non_tracks,
            BuildingType::Station(_) | BuildingType::Production(_) => {
                !overlapping_buildings.is_empty()
            },
        };

        let has_same_track = match building_info.building_type {
            BuildingType::Track(track_type) => overlapping_tracks.contains(&track_type),
            BuildingType::Station(_) | BuildingType::Production(_) => false,
        };

        if overlapping_other_players || invalid_overlaps {
            CanBuildResponse::Invalid
        } else if has_same_track {
            CanBuildResponse::AlreadyExists
        } else {
            CanBuildResponse::Ok
        }
    }

    #[allow(clippy::missing_errors_doc, clippy::result_unit_err)]
    pub fn build(
        &mut self,
        requesting_player_id: PlayerId,
        building_infos: Vec<BuildingInfo>,
    ) -> Result<(), ()> {
        if self.can_build(requesting_player_id, &building_infos) {
            self.append_all(building_infos);
            Ok(())
        } else {
            Err(())
        }
    }
}
