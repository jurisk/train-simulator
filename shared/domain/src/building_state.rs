use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::building_info::{BuildingDynamicInfo, BuildingInfo};
use crate::game_time::GameTimeDiff;
use crate::map_level::MapLevel;
use crate::resource_type::ResourceType;
use crate::{BuildingId, BuildingType, PlayerId, TileCoordsXZ, TrackType};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum CanBuildResponse {
    Ok, // Add `price` here later?
    AlreadyExists,
    Invalid,
}

// Later: Refactor to store also as a `FieldXZ` so that lookup by tile is efficient
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct BuildingState {
    buildings:            Vec<BuildingInfo>,
    // Link from each production building to the closest station
    closest_station_link: HashMap<BuildingId, BuildingId>,
}

impl Debug for BuildingState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuildingState").finish()
    }
}

impl BuildingState {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            buildings:            Vec::new(),
            closest_station_link: HashMap::new(),
        }
    }

    #[must_use]
    pub fn track_types_at(&self, tile: TileCoordsXZ) -> Vec<TrackType> {
        self.buildings_at(tile)
            .into_iter()
            .flat_map(|building| building.track_types_at(tile))
            .collect()
    }

    #[must_use]
    fn buildings_at(&self, tile: TileCoordsXZ) -> Vec<&BuildingInfo> {
        self.buildings
            .iter()
            .filter(|building| building.covers_tiles().contains(tile))
            .collect()
    }

    #[must_use]
    pub fn resource_types_accepted(&self, station_id: BuildingId) -> HashSet<ResourceType> {
        // Note - we are not checking that the building actually is a station here
        let mut results = HashSet::new();
        for (building_id, linked_station_id) in self.closest_station_link.clone() {
            if station_id == linked_station_id {
                if let Some(building) = self.find_building(building_id) {
                    if let BuildingType::Production(production_type) = building.building_type() {
                        for resource_type in production_type.resources_accepted() {
                            results.insert(resource_type);
                        }
                    }
                }
            }
        }
        results
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<BuildingInfo> {
        self.buildings.clone()
    }

    pub(crate) fn append_all(&mut self, additional: Vec<BuildingInfo>) {
        self.buildings.extend(additional);
        self.recalculate_cargo_forwarding_links();
    }

    #[allow(clippy::items_after_statements)]
    fn recalculate_cargo_forwarding_links(&mut self) {
        self.closest_station_link.clear();
        for building in &self.buildings {
            if let BuildingType::Production(_) = building.building_type() {
                let closest_station = self.find_closest_station(building);

                if let Some(closest_station) = closest_station {
                    let distance = BuildingInfo::manhattan_distance_between_closest_tiles(
                        building,
                        closest_station,
                    );
                    const CARGO_FORWARDING_DISTANCE_THRESHOLD: i32 = 2;
                    if distance <= CARGO_FORWARDING_DISTANCE_THRESHOLD {
                        self.closest_station_link
                            .insert(building.building_id(), closest_station.building_id());
                    }
                }
            }
        }
    }

    fn find_closest_station(&self, building: &BuildingInfo) -> Option<&BuildingInfo> {
        self.stations_owned_by(building.owner_id())
            .into_iter()
            .min_by_key(|station| {
                BuildingInfo::manhattan_distance_between_closest_tiles(building, station)
            })
    }

    fn stations_owned_by(&self, owner_id: PlayerId) -> Vec<&BuildingInfo> {
        self.buildings
            .iter()
            .filter(|building| {
                building.owner_id() == owner_id
                    && matches!(building.building_type(), BuildingType::Station(_))
            })
            .collect()
    }

    #[allow(clippy::missing_errors_doc, clippy::result_unit_err)]
    pub fn can_build(
        &mut self,
        requesting_player_id: PlayerId,
        building_infos: &[BuildingInfo],
        map_level: &MapLevel,
    ) -> bool {
        let valid_player_id = building_infos
            .iter()
            .all(|building_info| building_info.owner_id() == requesting_player_id);

        // TODO: Check that this is a valid building and there is enough money to build it, subtract money

        let tiles_are_free = building_infos.iter().all(|building_info| {
            self.can_build_building(requesting_player_id, building_info, map_level)
                == CanBuildResponse::Ok
        });

        valid_player_id && tiles_are_free
    }

    #[allow(clippy::collapsible_else_if)]
    pub(crate) fn can_build_building(
        &self,
        requesting_player_id: PlayerId,
        building_info: &BuildingInfo,
        map_level: &MapLevel,
    ) -> CanBuildResponse {
        // Later: Do not allow tracks that go out of bounds
        // Later: Are you doing bounds checking at all here? Can buildings be built out of bounds?
        let any_tile_out_of_bounds = building_info
            .covers_tiles()
            .to_set()
            .into_iter()
            .any(|tile| !map_level.tile_in_bounds(tile));

        if any_tile_out_of_bounds {
            return CanBuildResponse::Invalid;
        }

        let overlapping_buildings = building_info
            .covers_tiles()
            .to_set()
            .into_iter()
            .flat_map(|tile| self.buildings_at(tile))
            .collect::<Vec<_>>();

        let overlapping_other_players = overlapping_buildings
            .iter()
            .any(|building| building.owner_id() != requesting_player_id);

        let overlapping_tracks = overlapping_buildings
            .iter()
            .filter_map(|building| {
                match building.building_type() {
                    BuildingType::Track(track_type) => Some(track_type),
                    BuildingType::Station(_) | BuildingType::Production(_) => None,
                }
            })
            .collect::<HashSet<_>>();

        let has_overlapping_non_tracks = overlapping_buildings.iter().any(|building| {
            match building.building_type() {
                BuildingType::Track(_) => false,
                BuildingType::Station(_) | BuildingType::Production(_) => true,
            }
        });

        let invalid_overlaps = match building_info.building_type() {
            BuildingType::Track(_track_type) => has_overlapping_non_tracks,
            BuildingType::Station(_) | BuildingType::Production(_) => {
                !overlapping_buildings.is_empty()
            },
        };

        let has_same_track = match building_info.building_type() {
            BuildingType::Track(track_type) => overlapping_tracks.contains(&track_type),
            BuildingType::Station(_) | BuildingType::Production(_) => false,
        };

        let vertex_coords: Vec<_> = building_info
            .covers_tiles()
            .to_set()
            .into_iter()
            .flat_map(TileCoordsXZ::vertex_coords)
            .collect();

        let any_vertex_under_water = vertex_coords
            .iter()
            .any(|vertex| map_level.under_water(*vertex));

        let vertex_heights = vertex_coords
            .into_iter()
            .map(|vertex| map_level.height_at(vertex))
            .collect::<HashSet<_>>();

        let all_equal_heights = vertex_heights.len() == 1;
        let valid_heights = match building_info.building_type() {
            BuildingType::Track(track_type) => {
                let tile = building_info.reference_tile();

                // Later: Consider allowing more: https://wiki.openttd.org/en/Archive/Manual/Settings/Build%20on%20slopes .
                // Later: Consider not allowing slopes that are too steep
                track_type.connections().into_iter().all(|direction| {
                    let (a, b) = tile.vertex_coords_clockwise(direction);
                    let height_a = map_level.height_at(a);
                    let height_b = map_level.height_at(b);
                    height_a == height_b
                })
            },
            BuildingType::Station(_) | BuildingType::Production(_) => all_equal_heights,
        };

        if overlapping_other_players || invalid_overlaps || any_vertex_under_water || !valid_heights
        {
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
        building_infos: &[BuildingInfo],
        map_level: &MapLevel,
    ) -> Result<(), ()> {
        if self.can_build(requesting_player_id, building_infos, map_level) {
            self.append_all(building_infos.to_vec());
            Ok(())
        } else {
            Err(())
        }
    }

    #[must_use]
    pub fn find_building(&self, building_id: BuildingId) -> Option<&BuildingInfo> {
        self.buildings
            .iter()
            .find(|building| building.building_id() == building_id)
    }

    #[must_use]
    pub(crate) fn find_building_mut(
        &mut self,
        building_id: BuildingId,
    ) -> Option<&mut BuildingInfo> {
        self.buildings
            .iter_mut()
            .find(|building| building.building_id() == building_id)
    }

    #[must_use]
    pub fn filter_buildings_by_reference_tile(
        &self,
        reference_tile: TileCoordsXZ,
    ) -> Vec<&BuildingInfo> {
        self.buildings
            .iter()
            .filter(|building| building.reference_tile() == reference_tile)
            .collect()
    }

    pub(crate) fn advance_time_diff(&mut self, diff: GameTimeDiff) {
        for building in &mut self.buildings {
            building.advance(diff);
        }
        for (building_id, station_id) in self.closest_station_link.clone() {
            self.send_cargo_to_station(building_id, station_id);
        }
        // TODO HIGH: What about sending from station to building?
    }

    #[allow(clippy::unwrap_used)]
    fn send_cargo_to_station(&mut self, building_id: BuildingId, station_id: BuildingId) {
        let building = self.find_building_mut(building_id).unwrap();
        let cargo = building.shippable_cargo();
        let reverse = -cargo.clone();
        building.add_cargo(&reverse);

        let station = self.find_building_mut(station_id).unwrap();
        station.add_cargo(&cargo);
    }

    pub(crate) fn update_dynamic_info(
        &mut self,
        building_id: BuildingId,
        building_dynamic_info: &BuildingDynamicInfo,
    ) {
        for building in &mut self.buildings {
            if building.building_id() == building_id {
                building.update_dynamic_info(building_dynamic_info);
                return;
            }
        }
    }
}
