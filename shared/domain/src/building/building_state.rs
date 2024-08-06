use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};

use log::warn;
use serde::{Deserialize, Serialize};

use crate::building::building_info::{BuildingDynamicInfo, BuildingInfo};
use crate::building::track_info::TrackInfo;
use crate::game_time::GameTimeDiff;
use crate::map_level::MapLevel;
use crate::resource_type::ResourceType;
use crate::{
    BuildingId, BuildingType, IndustryBuildingId, PlayerId, StationId, TileCoordsXZ, TrackType,
};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum CanBuildResponse {
    Ok, // Add `price` here later?
    AlreadyExists,
    Invalid,
}

// Later: Refactor to store also as a `FieldXZ` so that lookup by tile is efficient
// TODO HIGH: Refactor `Vec<BuildingInfo>` into something common?
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct BuildingState {
    tracks:               Vec<TrackInfo>,
    industry_buildings:   Vec<BuildingInfo>,
    stations:             Vec<BuildingInfo>,
    // Link from each industry building to the closest station
    // Later: Should these be 1:1, N:1 or N:M correspondence between industry & station? Is it a problem if a station can accept & provide the same good and thus does not need trains?
    closest_station_link: HashMap<IndustryBuildingId, StationId>,
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
            tracks:               Vec::new(),
            industry_buildings:   Vec::new(),
            stations:             Vec::new(),
            closest_station_link: HashMap::new(),
        }
    }

    #[must_use]
    pub fn track_types_at(&self, tile: TileCoordsXZ) -> Vec<TrackType> {
        let mut results = vec![];
        for station in self.stations_at(tile) {
            for track in station.station_track_types_at(tile) {
                results.push(track);
            }
        }
        for track in self.tracks_at(tile) {
            results.push(track.track_type);
        }
        results
    }

    #[must_use]
    fn buildings_at(&self, tile: TileCoordsXZ) -> Vec<&BuildingInfo> {
        self.stations
            .iter()
            .chain(self.industry_buildings.iter())
            .filter(|building| building.covers_tiles().contains(tile))
            .collect()
    }

    #[must_use]
    fn stations_at(&self, tile: TileCoordsXZ) -> Vec<&BuildingInfo> {
        self.stations
            .iter()
            .filter(|station| station.covers_tiles().contains(tile))
            .collect()
    }

    #[must_use]
    pub fn tracks_at(&self, tile: TileCoordsXZ) -> Vec<&TrackInfo> {
        self.tracks
            .iter()
            .filter(|track| track.tile == tile)
            .collect()
    }

    #[must_use]
    pub fn station_at(&self, tile: TileCoordsXZ) -> Option<&BuildingInfo> {
        let results: Vec<_> = self.stations_at(tile);

        if results.len() > 1 {
            warn!("Found multiple stations at tile {:?}: {:?}", tile, results);
            None
        } else {
            results.first().copied()
        }
    }

    #[must_use]
    pub fn resource_types_accepted_by_station(
        &self,
        station_id: BuildingId,
    ) -> HashSet<ResourceType> {
        // Note - we are not checking that the building actually is a station here
        let mut results = HashSet::new();
        for (industry_building_id, linked_station_id) in self.closest_station_link.clone() {
            if station_id == linked_station_id {
                if let Some(building) = self.find_industry_building(industry_building_id) {
                    if let BuildingType::Industry(industry_type) = building.building_type() {
                        for resource_type in industry_type.resources_accepted() {
                            results.insert(resource_type);
                        }
                    }
                }
            }
        }
        results
    }

    #[must_use]
    pub fn all_stations(&self) -> &Vec<BuildingInfo> {
        &self.stations
    }

    #[must_use]
    pub fn all_industry_buildings(&self) -> &Vec<BuildingInfo> {
        &self.industry_buildings
    }

    #[must_use]
    pub fn track_infos(&self) -> Vec<TrackInfo> {
        // TODO: Stop cloning all the time?
        self.tracks.clone()
    }

    pub fn append_industry_buildings(&mut self, additional: Vec<BuildingInfo>) {
        self.industry_buildings.extend(additional);
        self.recalculate_cargo_forwarding_links();
    }

    pub fn append_stations(&mut self, additional: Vec<BuildingInfo>) {
        self.stations.extend(additional);
        self.recalculate_cargo_forwarding_links();
    }

    pub fn append_tracks(&mut self, additional: Vec<TrackInfo>) {
        self.tracks.extend(additional);
    }

    #[allow(clippy::items_after_statements)]
    fn recalculate_cargo_forwarding_links(&mut self) {
        self.closest_station_link.clear();
        for building in &self.industry_buildings {
            if let BuildingType::Industry(_) = building.building_type() {
                if let Some((closest_station, distance)) = self.find_closest_station(building) {
                    const CARGO_FORWARDING_DISTANCE_THRESHOLD: i32 = 1;
                    if distance <= CARGO_FORWARDING_DISTANCE_THRESHOLD {
                        self.closest_station_link
                            .insert(building.building_id(), closest_station.building_id());
                    }
                }
            }
        }
    }

    fn find_closest_station(&self, building: &BuildingInfo) -> Option<(&BuildingInfo, i32)> {
        self.stations_owned_by(building.owner_id())
            .into_iter()
            .map(|station| {
                (
                    station,
                    BuildingInfo::manhattan_distance_between_closest_tiles(building, station),
                )
            })
            .min_by_key(|(_, distance)| *distance)
    }

    fn stations_owned_by(&self, owner_id: PlayerId) -> Vec<&BuildingInfo> {
        self.stations
            .iter()
            .filter(|building| {
                building.owner_id() == owner_id
                    && matches!(building.building_type(), BuildingType::Station(_))
            })
            .collect()
    }

    #[allow(clippy::missing_errors_doc, clippy::result_unit_err)]
    pub fn can_build_buildings(
        &mut self,
        requesting_player_id: PlayerId,
        building_infos: &[BuildingInfo],
        map_level: &MapLevel,
    ) -> bool {
        let valid_player_id = building_infos
            .iter()
            .all(|building_info| building_info.owner_id() == requesting_player_id);

        // TODO: Check that this is a valid building and there is enough money to build it, subtract money

        let can_build = building_infos.iter().all(|building_info| {
            self.can_build_building(requesting_player_id, building_info, map_level)
                == CanBuildResponse::Ok
        });

        valid_player_id && can_build
    }

    #[allow(clippy::missing_errors_doc, clippy::result_unit_err)]
    pub fn can_build_tracks(
        &mut self,
        requesting_player_id: PlayerId,
        track_infos: &[TrackInfo],
        map_level: &MapLevel,
    ) -> bool {
        let valid_player_id = track_infos
            .iter()
            .all(|track_info| track_info.owner_id == requesting_player_id);

        let can_build = track_infos.iter().all(|track_info| {
            self.can_build_track(requesting_player_id, track_info, map_level)
                == CanBuildResponse::Ok
        });

        valid_player_id && can_build
    }

    pub(crate) fn can_build_track(
        &self,
        requesting_player_id: PlayerId,
        track_info: &TrackInfo,
        map_level: &MapLevel,
    ) -> CanBuildResponse {
        // Later: Do not allow tracks that go out of bounds
        if !map_level.tile_in_bounds(track_info.tile) {
            return CanBuildResponse::Invalid;
        }

        // TODO HIGH:   Actually, if the attempt is to build a track over tracks provided by a station,
        //              we should allow it as CanBuildResponse::AlreadyExists
        let overlapping_buildings = self.buildings_at(track_info.tile);
        let invalid_overlaps = !overlapping_buildings.is_empty();

        let overlapping_other_players = overlapping_buildings
            .iter()
            .any(|building| building.owner_id() != requesting_player_id);

        let overlapping_tracks = self
            .tracks_at(track_info.tile)
            .into_iter()
            .map(|other_track| other_track.track_type)
            .collect::<HashSet<_>>();

        let has_same_track = overlapping_tracks.contains(&track_info.track_type);

        let vertex_coords = track_info.tile.vertex_coords();

        let any_vertex_under_water = vertex_coords
            .into_iter()
            .any(|vertex| map_level.under_water(vertex));

        // Later: Consider allowing more: https://wiki.openttd.org/en/Archive/Manual/Settings/Build%20on%20slopes .
        // Later: Consider not allowing slopes that are too steep
        let valid_heights = track_info
            .track_type
            .connections()
            .into_iter()
            .all(|direction| {
                let (a, b) = track_info.tile.vertex_coords_clockwise(direction);
                let height_a = map_level.height_at(a);
                let height_b = map_level.height_at(b);
                height_a == height_b
            });

        if overlapping_other_players || invalid_overlaps || any_vertex_under_water || !valid_heights
        {
            CanBuildResponse::Invalid
        } else if has_same_track {
            CanBuildResponse::AlreadyExists
        } else {
            CanBuildResponse::Ok
        }
    }

    #[allow(clippy::collapsible_else_if)]
    pub(crate) fn can_build_building(
        &self,
        requesting_player_id: PlayerId,
        building_info: &BuildingInfo,
        map_level: &MapLevel,
    ) -> CanBuildResponse {
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

        let invalid_overlaps = !overlapping_buildings.is_empty();

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
            BuildingType::Station(_) | BuildingType::Industry(_) => all_equal_heights,
        };

        if overlapping_other_players || invalid_overlaps || any_vertex_under_water || !valid_heights
        {
            CanBuildResponse::Invalid
        } else {
            CanBuildResponse::Ok
        }
    }

    #[allow(clippy::missing_errors_doc, clippy::result_unit_err)]
    pub(crate) fn build_industry_buildings(
        &mut self,
        requesting_player_id: PlayerId,
        building_infos: &[BuildingInfo],
        map_level: &MapLevel,
    ) -> Result<(), ()> {
        if self.can_build_buildings(requesting_player_id, building_infos, map_level) {
            self.append_industry_buildings(building_infos.to_vec());
            Ok(())
        } else {
            Err(())
        }
    }

    #[allow(clippy::missing_errors_doc, clippy::result_unit_err)]
    pub(crate) fn build_stations(
        &mut self,
        requesting_player_id: PlayerId,
        building_infos: &[BuildingInfo],
        map_level: &MapLevel,
    ) -> Result<(), ()> {
        if self.can_build_buildings(requesting_player_id, building_infos, map_level) {
            self.append_stations(building_infos.to_vec());
            Ok(())
        } else {
            Err(())
        }
    }

    #[allow(clippy::missing_errors_doc, clippy::result_unit_err)]
    pub fn build_tracks(
        &mut self,
        requesting_player_id: PlayerId,
        track_infos: &[TrackInfo],
        map_level: &MapLevel,
    ) -> Result<(), ()> {
        if self.can_build_tracks(requesting_player_id, track_infos, map_level) {
            self.append_tracks(track_infos.to_vec());
            Ok(())
        } else {
            Err(())
        }
    }

    #[must_use]
    pub fn find_station(&self, station_id: StationId) -> Option<&BuildingInfo> {
        self.stations
            .iter()
            .find(|building| building.building_id() == station_id)
    }

    #[must_use]
    pub fn find_industry_building(
        &self,
        industry_building_id: IndustryBuildingId,
    ) -> Option<&BuildingInfo> {
        self.industry_buildings
            .iter()
            .find(|building| building.building_id() == industry_building_id)
    }

    #[must_use]
    pub(crate) fn find_industry_building_mut(
        &mut self,
        industry_building_id: IndustryBuildingId,
    ) -> Option<&mut BuildingInfo> {
        self.industry_buildings
            .iter_mut()
            .find(|industry_building| industry_building.building_id() == industry_building_id)
    }

    #[must_use]
    pub(crate) fn find_station_mut(&mut self, station_id: StationId) -> Option<&mut BuildingInfo> {
        self.stations
            .iter_mut()
            .find(|building| building.building_id() == station_id)
    }

    pub(crate) fn advance_time_diff(&mut self, diff: GameTimeDiff) {
        for industry_building in &mut self.industry_buildings {
            industry_building.advance_industry_building(diff);
        }
        for (industry_building_id, station_id) in self.closest_station_link.clone() {
            self.exchange_cargo(industry_building_id, station_id);
        }
    }

    #[allow(clippy::unwrap_used)]
    fn exchange_cargo(&mut self, industry_building_id: IndustryBuildingId, station_id: StationId) {
        let industry_building = self.find_industry_building(industry_building_id).unwrap();
        let industry_building_inputs = industry_building.industry_transform_inputs();
        let cargo_from_building_to_station = industry_building.industry_building_shippable_cargo();

        let station = self.find_station(station_id).unwrap();
        let cargo_from_station_to_building =
            station
                .station_shippable_cargo()
                .filter(|(resource_type, _cargo_amount)| {
                    industry_building_inputs.contains(&resource_type)
                });

        let building_mut = self
            .find_industry_building_mut(industry_building_id)
            .unwrap();
        building_mut.remove_cargo(&cargo_from_building_to_station);
        building_mut.add_cargo(&cargo_from_station_to_building);

        let station_mut = self.find_station_mut(station_id).unwrap();
        station_mut.add_cargo(&cargo_from_building_to_station);
        station_mut.remove_cargo(&cargo_from_station_to_building);
    }

    pub(crate) fn update_dynamic_infos(
        &mut self,
        industry_building_dynamic_infos: &HashMap<IndustryBuildingId, BuildingDynamicInfo>,
        station_dynamic_infos: &HashMap<StationId, BuildingDynamicInfo>,
    ) {
        for (industry_building_id, building_dynamic_info) in industry_building_dynamic_infos {
            self.update_industry_building_dynamic_info(
                *industry_building_id,
                building_dynamic_info,
            );
        }

        for (station_id, building_dynamic_info) in station_dynamic_infos {
            self.update_station_dynamic_info(*station_id, building_dynamic_info);
        }
    }

    fn update_industry_building_dynamic_info(
        &mut self,
        industry_building_id: IndustryBuildingId,
        building_dynamic_info: &BuildingDynamicInfo,
    ) {
        if let Some(building) = self.find_industry_building_mut(industry_building_id) {
            building.update_dynamic_info(building_dynamic_info);
        } else {
            warn!(
                "Could not find industry building with id {:?}",
                industry_building_id
            );
        }
    }

    fn update_station_dynamic_info(
        &mut self,
        station_id: StationId,
        building_dynamic_info: &BuildingDynamicInfo,
    ) {
        if let Some(building) = self.find_station_mut(station_id) {
            building.update_dynamic_info(building_dynamic_info);
        } else {
            warn!("Could not find station with id {:?}", station_id);
        }
    }
}
