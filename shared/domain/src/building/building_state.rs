#![allow(clippy::missing_errors_doc, clippy::result_unit_err)]

use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};

use log::warn;
use serde::{Deserialize, Serialize};
use shared_util::direction_xz::DirectionXZ;

use crate::building::building_info::{
    BuildingDynamicInfo, BuildingInfo, WithOwner, WithTileCoverage,
};
use crate::building::industry_building_info::IndustryBuildingInfo;
use crate::building::industry_type::IndustryType;
use crate::building::station_info::StationInfo;
use crate::building::track_info::TrackInfo;
use crate::cargo_map::CargoOps;
use crate::game_time::GameTimeDiff;
use crate::map_level::map_level::MapLevel;
use crate::resource_type::ResourceType;
use crate::tile_coverage::TileCoverage;
use crate::{IndustryBuildingId, PlayerId, StationId, TileCoordsXZ, TrackId, TrackType};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum CanBuildResponse {
    Ok, // Add `price` here later?
    AlreadyExists,
    Invalid,
}

// Later: There is a dual nature here to both be the "validator" (check if something can be built) and the "state" (store what has been built).
// Later: Refactor to store also as a `FieldXZ` so that lookup by tile is efficient
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct BuildingState {
    tracks:               Vec<TrackInfo>,
    industry_buildings:   HashMap<IndustryBuildingId, IndustryBuildingInfo>,
    stations:             HashMap<StationId, StationInfo>,
    // Link from each industry building to the closest station
    // Later: Should these be 1:1, N:1 or N:M correspondence between industry & station? Is it a problem if a station can accept & provide the same good and thus does not need trains?
    closest_station_link: HashMap<IndustryBuildingId, StationId>,
}

impl Debug for BuildingState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BuildingState({} tracks, {} industry buildings, {} stations)",
            self.tracks.len(),
            self.industry_buildings.len(),
            self.stations.len()
        )
    }
}

impl BuildingState {
    #[must_use]
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            tracks:               Vec::new(),
            industry_buildings:   HashMap::new(),
            stations:             HashMap::new(),
            closest_station_link: HashMap::new(),
        }
    }

    // TODO HIGH: Optimize this as it is called often
    #[must_use]
    pub fn track_types_with_connection(
        &self,
        tile: TileCoordsXZ,
        connection: DirectionXZ,
    ) -> impl IntoIterator<Item = TrackType> {
        self.track_types_at(tile)
            .into_iter()
            .filter(move |track_type| track_type.connections().contains(&connection))
    }

    // TODO HIGH: This is frequently called and should be optimised
    #[must_use]
    pub fn track_types_at(&self, tile: TileCoordsXZ) -> Vec<TrackType> {
        let mut results = vec![];
        if let Some(station) = self.station_at(tile) {
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
    fn building_at(&self, tile: TileCoordsXZ) -> Option<&dyn BuildingInfo> {
        let station = self.station_at(tile);
        let industry_building = self.industry_building_at(tile);
        match (station, industry_building) {
            (Some(station), None) => Some(station),
            (None, Some(industry_building)) => Some(industry_building),
            (Some(station), Some(industry_building)) => {
                warn!(
                    "Found both a station and an industry building at tile {:?}: {:?} {:?}",
                    tile, station, industry_building
                );
                None
            },
            (None, None) => None,
        }
    }

    #[must_use]
    pub fn tracks_at(&self, tile: TileCoordsXZ) -> Vec<&TrackInfo> {
        self.tracks
            .iter()
            .filter(|track| track.tile == tile)
            .collect()
    }

    #[must_use]
    pub fn station_at(&self, tile: TileCoordsXZ) -> Option<&StationInfo> {
        let results: Vec<_> = self
            .all_stations()
            .into_iter()
            .filter(|station| station.covers_tiles().contains(tile))
            .collect();

        if results.len() > 1 {
            warn!("Found multiple stations at tile {:?}: {:?}", tile, results);
            None
        } else {
            results.first().copied()
        }
    }

    #[must_use]
    pub fn industry_building_at(&self, tile: TileCoordsXZ) -> Option<&IndustryBuildingInfo> {
        let results: Vec<_> = self
            .all_industry_buildings()
            .into_iter()
            .filter(|industry_building| industry_building.covers_tiles().contains(tile))
            .collect();

        if results.len() > 1 {
            warn!(
                "Found multiple industry buildings at tile {:?}: {:?}",
                tile, results
            );
            None
        } else {
            results.first().copied()
        }
    }

    #[must_use]
    pub fn resource_types_accepted_by_station(
        &self,
        station_id: StationId,
    ) -> HashSet<ResourceType> {
        // Note - we are not checking that the building actually is a station here
        let mut results = HashSet::new();
        for (industry_building_id, linked_station_id) in self.closest_station_link.clone() {
            if station_id == linked_station_id {
                if let Some(building) = self.find_industry_building(industry_building_id) {
                    for resource_type in building.industry_type().input_resource_types() {
                        results.insert(resource_type);
                    }
                }
            }
        }
        results
    }

    #[must_use]
    pub fn all_stations(&self) -> impl IntoIterator<Item = &StationInfo> {
        self.stations.values()
    }

    #[must_use]
    pub fn find_players_stations(&self, player_id: PlayerId) -> Vec<&StationInfo> {
        self.all_stations()
            .into_iter()
            .filter(|station| station.owner_id() == player_id)
            .collect()
    }

    #[must_use]
    pub fn all_industry_buildings(&self) -> impl IntoIterator<Item = &IndustryBuildingInfo> {
        self.industry_buildings.values()
    }

    #[must_use]
    pub fn all_tracks(&self) -> &Vec<TrackInfo> {
        &self.tracks
    }

    pub fn append_industry_building(&mut self, industry_building: IndustryBuildingInfo) {
        self.industry_buildings
            .insert(industry_building.id(), industry_building);
        self.recalculate_cargo_forwarding_links();
    }

    pub fn append_station(&mut self, station: StationInfo) {
        self.stations.insert(station.id(), station);
        self.recalculate_cargo_forwarding_links();
    }

    pub fn append_tracks(&mut self, additional: Vec<TrackInfo>) {
        self.tracks.extend(additional);
    }

    fn recalculate_cargo_forwarding_links(&mut self) {
        self.closest_station_link.clear();
        for building in self.industry_buildings.values() {
            if let Some((closest_station, distance)) = self.find_closest_station(building) {
                const CARGO_FORWARDING_DISTANCE_THRESHOLD: i32 = 1;
                if distance <= CARGO_FORWARDING_DISTANCE_THRESHOLD {
                    self.closest_station_link
                        .insert(building.id(), closest_station.id());
                }
            }
        }
    }

    fn find_closest_station(&self, building: &IndustryBuildingInfo) -> Option<(&StationInfo, i32)> {
        self.find_players_stations(building.owner_id())
            .into_iter()
            .map(|station| {
                (
                    station,
                    TileCoverage::manhattan_distance_between_closest_tiles(
                        &building.covers_tiles(),
                        &station.covers_tiles(),
                    ),
                )
            })
            .min_by_key(|(_, distance)| *distance)
    }

    #[must_use]
    pub fn find_players_industry_buildings_without_linked_stations(
        &self,
        player_id: PlayerId,
    ) -> Vec<&IndustryBuildingInfo> {
        self.all_industry_buildings()
            .into_iter()
            .filter(|building| {
                building.owner_id() == player_id
                    && !self.closest_station_link.contains_key(&building.id())
            })
            .collect()
    }

    pub fn can_build_building<T: BuildingInfo>(
        &self,
        requesting_player_id: PlayerId,
        building_info: &T,
        map_level: &MapLevel,
    ) -> bool {
        let valid_player_id = building_info.owner_id() == requesting_player_id;

        // TODO: Check that this is a valid building and there is enough money to build it, subtract money

        let can_build = self.can_build_for_coverage(&building_info.covers_tiles(), map_level)
            == CanBuildResponse::Ok;

        valid_player_id && can_build
    }

    // TODO: Needs test coverage
    pub(crate) fn can_build_track(
        &self,
        requesting_player_id: PlayerId,
        tile: TileCoordsXZ,
        track_type: TrackType,
    ) -> CanBuildResponse {
        let overlapping_station = self.station_at(tile);
        let has_same_track_from_station = if let Some(station) = overlapping_station {
            station.station_track_types_at(tile).contains(&track_type)
        } else {
            false
        };

        let overlapping_industry = self.industry_building_at(tile);
        let invalid_station_overlap = !has_same_track_from_station && overlapping_station.is_some();
        let invalid_industry_overlap = overlapping_industry.is_some();
        let invalid_overlaps = invalid_industry_overlap || invalid_station_overlap;

        // TODO HIGH: This is rather inefficient and should be improved
        let overlapping_tracks = self.tracks_at(tile);

        let overlapping_other_players_tracks = overlapping_tracks
            .iter()
            .any(|track| track.owner_id() != requesting_player_id);

        let has_same_track = {
            let overlapping_tracks_from_tracks = overlapping_tracks
                .into_iter()
                .map(|other_track| other_track.track_type)
                .collect::<HashSet<_>>();

            let has_same_track_from_tracks = overlapping_tracks_from_tracks.contains(&track_type);

            has_same_track_from_tracks || has_same_track_from_station
        };

        if overlapping_other_players_tracks || invalid_overlaps {
            CanBuildResponse::Invalid
        } else if has_same_track {
            CanBuildResponse::AlreadyExists
        } else {
            CanBuildResponse::Ok
        }
    }

    pub(crate) fn can_build_for_coverage(
        &self,
        tile_coverage: &TileCoverage,
        map_level: &MapLevel,
    ) -> CanBuildResponse {
        let any_tile_out_of_bounds = tile_coverage
            .to_set()
            .into_iter()
            .any(|tile| !map_level.tile_in_bounds(tile));

        if any_tile_out_of_bounds {
            return CanBuildResponse::Invalid;
        }

        let overlapping_buildings = tile_coverage
            .to_set()
            .into_iter()
            .filter_map(|tile| self.building_at(tile))
            .collect::<Vec<_>>();

        let invalid_overlaps = !overlapping_buildings.is_empty();

        let vertex_coords: Vec<_> = tile_coverage
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

        let valid_heights = vertex_heights.len() == 1;

        if invalid_overlaps || any_vertex_under_water || !valid_heights {
            CanBuildResponse::Invalid
        } else {
            CanBuildResponse::Ok
        }
    }

    pub(crate) fn build_industry_building(
        &mut self,
        requesting_player_id: PlayerId,
        industry_building_info: &IndustryBuildingInfo,
        map_level: &MapLevel,
    ) -> Result<(), ()> {
        if self.can_build_building(requesting_player_id, industry_building_info, map_level) {
            self.append_industry_building(industry_building_info.clone());
            Ok(())
        } else {
            Err(())
        }
    }

    pub(crate) fn build_station(
        &mut self,
        requesting_player_id: PlayerId,
        station_info: &StationInfo,
        map_level: &MapLevel,
    ) -> Result<(), ()> {
        if self.can_build_building(requesting_player_id, station_info, map_level) {
            self.append_station(station_info.clone());
            Ok(())
        } else {
            Err(())
        }
    }

    #[must_use]
    pub fn find_station(&self, station_id: StationId) -> Option<&StationInfo> {
        self.stations.get(&station_id)
    }

    #[must_use]
    pub fn find_industry_building(
        &self,
        industry_building_id: IndustryBuildingId,
    ) -> Option<&IndustryBuildingInfo> {
        self.all_industry_buildings()
            .into_iter()
            .find(|building| building.id() == industry_building_id)
    }

    #[must_use]
    pub fn find_industry_building_by_owner_and_type(
        &self,
        owner_id: PlayerId,
        industry_type: IndustryType,
    ) -> Vec<&IndustryBuildingInfo> {
        self.all_industry_buildings()
            .into_iter()
            .filter(|building| {
                building.owner_id() == owner_id && building.industry_type() == industry_type
            })
            .collect()
    }

    #[must_use]
    pub(crate) fn find_industry_building_mut(
        &mut self,
        industry_building_id: IndustryBuildingId,
    ) -> Option<&mut IndustryBuildingInfo> {
        self.industry_buildings.get_mut(&industry_building_id)
    }

    #[must_use]
    pub(crate) fn find_station_mut(&mut self, station_id: StationId) -> Option<&mut StationInfo> {
        self.stations.get_mut(&station_id)
    }

    pub(crate) fn advance_time_diff(&mut self, diff: GameTimeDiff) {
        for industry_building in &mut self.industry_buildings.values_mut() {
            industry_building.advance_industry_building(diff);
        }
        for (industry_building_id, station_id) in self.closest_station_link.clone() {
            self.exchange_cargo(industry_building_id, station_id);
        }
    }

    #[expect(clippy::unwrap_used)]
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

    pub fn remove_industry_building(&mut self, industry_building_id: IndustryBuildingId) {
        self.industry_buildings.remove(&industry_building_id);
        self.recalculate_cargo_forwarding_links();
    }

    pub fn remove_station(&mut self, station_id: StationId) {
        self.stations.remove(&station_id);
        self.recalculate_cargo_forwarding_links();
    }

    pub fn attempt_to_remove_track(
        &mut self,
        requesting_player_id: PlayerId,
        track_id: TrackId,
    ) -> Result<(), ()> {
        // TODO: Check there are no trains on (or near?) these tracks
        let track = self
            .tracks
            .iter()
            .find(|track| track.id() == track_id)
            .ok_or(())?;

        if track.owner_id() == requesting_player_id {
            self.remove_track(track_id);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn attempt_to_remove_industry_building(
        &mut self,
        requesting_player_id: PlayerId,
        industry_building_id: IndustryBuildingId,
    ) -> Result<(), ()> {
        let industry_building = self
            .industry_buildings
            .get(&industry_building_id)
            .ok_or(())?;

        if industry_building.owner_id() == requesting_player_id {
            self.remove_industry_building(industry_building_id);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn attempt_to_remove_station(
        &mut self,
        requesting_player_id: PlayerId,
        station_id: StationId,
    ) -> Result<(), ()> {
        // TODO: Check there are no trains on (or near?) this station
        let station = self.find_station(station_id).ok_or(())?;
        if station.owner_id() == requesting_player_id {
            self.remove_station(station_id);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn remove_track(&mut self, track_id: TrackId) {
        self.tracks.retain(|track| track.id() != track_id);
    }
}
