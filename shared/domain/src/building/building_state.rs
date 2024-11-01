#![allow(clippy::missing_errors_doc, clippy::result_unit_err)]

use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};

use log::{trace, warn};
use serde::{Deserialize, Serialize};
use shared_util::bool_ops::BoolResultOps;
use shared_util::direction_xz::DirectionXZ;
use shared_util::grid_xz::GridXZ;

use crate::building::building_info::{
    BuildingDynamicInfo, BuildingInfo, WithBuildingDynamicInfoMut, WithCostToBuild, WithOwner,
    WithTileCoverage,
};
use crate::building::industry_building_info::IndustryBuildingInfo;
use crate::building::industry_type::IndustryType;
use crate::building::military_building_info::MilitaryBuildingInfo;
use crate::building::station_info::StationInfo;
use crate::building::track_info::TrackInfo;
use crate::building::track_state::{MaybeTracksOnTile, TrackState};
use crate::building::{BuildCosts, BuildError};
use crate::cargo_amount::CargoAmount;
use crate::cargo_map::{CargoMap, CargoOps, WithCargo, WithCargoMut};
use crate::game_time::GameTimeDiff;
use crate::resource_type::ResourceType;
use crate::tile_coverage::TileCoverage;
use crate::transport::track_type_set::TrackTypeSet;
use crate::{
    IndustryBuildingId, MilitaryBuildingId, PlayerId, StationId, TileCoordsXZ, TrackId, TrackType,
};

#[derive(PartialEq, Clone, Debug)]
pub enum CanBuildResponse {
    Ok,
    AlreadyExists,
    Invalid(BuildError),
}

// TODO: You have too many grids... I think you should merge the various grids that show what buildings / tracks are at that tile
// Later: There is a dual nature here to both be the "validator" (check if something can be built) and the "state" (store what has been built).
// Later: Refactor to store also as a `FieldXZ` so that lookup by tile is efficient
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct BuildingState {
    tracks:                  TrackState,
    industry_buildings:      HashMap<IndustryBuildingId, IndustryBuildingInfo>,
    tile_industry_buildings: GridXZ<TileCoordsXZ, Option<IndustryBuildingId>>,
    military_buildings:      HashMap<MilitaryBuildingId, MilitaryBuildingInfo>,
    tile_military_buildings: GridXZ<TileCoordsXZ, Option<MilitaryBuildingId>>,
    stations:                HashMap<StationId, StationInfo>,
    tile_stations:           GridXZ<TileCoordsXZ, Option<StationId>>,
    station_track_types_at:  GridXZ<TileCoordsXZ, TrackTypeSet>,
    // Link from each industry building to the closest station
    // Later: Should these be 1:1, N:1 or N:M correspondence between industry & station? Is it a problem if a station can accept & provide the same good and thus does not need trains?
    closest_station_link:    HashMap<IndustryBuildingId, StationId>,
}

impl Debug for BuildingState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BuildingState({} industry buildings, {} stations)",
            self.industry_buildings.len(),
            self.stations.len()
        )
    }
}

impl BuildingState {
    #[must_use]
    pub fn new(size_x: usize, size_z: usize) -> Self {
        Self {
            tracks:                  TrackState::new(size_x, size_z),
            industry_buildings:      HashMap::new(),
            tile_industry_buildings: GridXZ::filled_with(size_x, size_z, None),
            military_buildings:      HashMap::new(),
            tile_military_buildings: GridXZ::filled_with(size_x, size_z, None),
            stations:                HashMap::new(),
            tile_stations:           GridXZ::filled_with(size_x, size_z, None),
            station_track_types_at:  GridXZ::filled_with(size_x, size_z, TrackTypeSet::new()),
            closest_station_link:    HashMap::new(),
        }
    }

    #[expect(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub fn gift_initial_construction_yard(&mut self, player_id: PlayerId, tile: TileCoordsXZ) {
        // Later: We could have the initial cargo a parameter and have it in the scenario. Or not.
        // Later: Having this public is wrong, but we use it from somewhat unrelated tests.
        let construction_yard_id = IndustryBuildingId::random();
        let construction_yard = IndustryBuildingInfo::new(
            player_id,
            construction_yard_id,
            tile,
            IndustryType::ConstructionYard,
        );
        let () = self
            .build_industry_building(&construction_yard, BuildCosts::none())
            .unwrap();
        let construction_yard = self
            .find_industry_building_mut(construction_yard_id)
            .unwrap();
        let mut dynamic_info = construction_yard.dynamic_info_mut();
        let cargo = dynamic_info.cargo_mut();

        // TODO HIGH: Lower these to only cover initial supply chains for Concrete, Steel, Timber
        cargo.add(ResourceType::Concrete, CargoAmount::new(480.0));
        cargo.add(ResourceType::Steel, CargoAmount::new(480.0));
        cargo.add(ResourceType::Timber, CargoAmount::new(240.0));
    }

    // TODO: Optimize this as it is called often
    #[must_use]
    pub fn track_types_with_connection(
        &self,
        tile: TileCoordsXZ,
        connection: DirectionXZ,
    ) -> impl IntoIterator<Item = TrackType> {
        let track_types = self.track_types_at(tile);
        TrackType::matching_direction(connection)
            .into_iter()
            .filter(move |track_type| track_types.contains(*track_type))
    }

    #[must_use]
    pub fn track_types_at(&self, tile: TileCoordsXZ) -> TrackTypeSet {
        let from_track = self.tracks.track_types_at(tile);
        if from_track.is_empty() {
            self.station_track_types_at.get_or_default(tile)
        } else {
            from_track
        }
    }

    #[must_use]
    pub fn building_at(&self, tile: TileCoordsXZ) -> Option<&dyn BuildingInfo> {
        let station = self.station_at(tile);
        let industry_building = self.industry_building_at(tile);
        let military_building = self.military_building_at(tile);
        match (station, industry_building, military_building) {
            (Some(station), None, None) => Some(station),
            (None, Some(industry_building), None) => Some(industry_building),
            (None, None, Some(military_building)) => Some(military_building),
            (None, None, None) => None,
            _ => {
                warn!(
                    "Found invalid building state at {:?}: {:?} {:?} {:?}",
                    tile, station, industry_building, military_building
                );
                None
            },
        }
    }

    #[must_use]
    pub fn tracks_at(&self, tile: TileCoordsXZ) -> MaybeTracksOnTile {
        self.tracks.tracks_at(tile)
    }

    #[must_use]
    pub fn station_at(&self, tile: TileCoordsXZ) -> Option<&StationInfo> {
        match self.tile_stations.get(tile) {
            Some(Some(station_id)) => self.stations.get(station_id),
            _ => None,
        }
    }

    #[must_use]
    pub fn industry_building_at(&self, tile: TileCoordsXZ) -> Option<&IndustryBuildingInfo> {
        match self.tile_industry_buildings.get(tile) {
            Some(Some(industry_building_id)) => self.industry_buildings.get(industry_building_id),
            _ => None,
        }
    }

    #[must_use]
    pub fn military_building_at(&self, tile: TileCoordsXZ) -> Option<&MilitaryBuildingInfo> {
        match self.tile_military_buildings.get(tile) {
            Some(Some(military_building_id)) => self.military_buildings.get(military_building_id),
            _ => None,
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
    pub fn all_track_infos(&self) -> Vec<TrackInfo> {
        self.tracks.all_track_infos()
    }

    pub fn append_industry_building(&mut self, industry_building: IndustryBuildingInfo) {
        for tile in industry_building.covers_tiles().to_set() {
            self.tile_industry_buildings[tile] = Some(industry_building.id());
        }
        self.industry_buildings
            .insert(industry_building.id(), industry_building);
        self.recalculate_cargo_forwarding_links();
    }

    pub fn append_military_building(&mut self, military_building: MilitaryBuildingInfo) {
        for tile in military_building.covers_tiles().to_set() {
            self.tile_military_buildings[tile] = Some(military_building.id());
        }
        self.military_buildings
            .insert(military_building.id(), military_building);
    }

    pub fn append_station(&mut self, station: StationInfo) {
        for tile in station.covers_tiles().to_set() {
            self.tile_stations[tile] = Some(station.id());
            for track_type in station.station_track_types_at(tile) {
                if let Some(found) = self.station_track_types_at.get_mut(tile) {
                    found.insert(track_type);
                }
            }
        }
        self.stations.insert(station.id(), station);
        self.recalculate_cargo_forwarding_links();
    }

    pub fn append_tracks(&mut self, additional: Vec<TrackInfo>) {
        self.tracks.append_tracks(additional);
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
    pub fn find_linked_station(
        &self,
        industry_building_id: IndustryBuildingId,
    ) -> Option<&StationInfo> {
        self.closest_station_link
            .get(&industry_building_id)
            .and_then(|station_id| self.stations.get(station_id))
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

    pub fn can_build_with_coverage<T: WithTileCoverage>(
        &self,
        building_info: &T,
    ) -> Result<(), BuildError> {
        self.can_build_for_coverage(&building_info.covers_tiles())?;
        Ok(())
    }

    pub(crate) fn build_tracks(&mut self, tracks: Vec<TrackInfo>, costs: BuildCosts) {
        self.append_tracks(tracks);
        self.pay_costs(costs);
    }

    // TODO: Needs test coverage
    pub(crate) fn can_build_track(
        &self,
        owner_id: PlayerId,
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

        let overlapping_tracks = self.tracks_at(tile);

        let overlapping_other_players_tracks = overlapping_tracks
            .owner_id()
            .into_iter()
            .any(|player_id| player_id != owner_id);

        let has_same_track = {
            let has_same_track_from_tracks = overlapping_tracks.track_types().contains(track_type);

            has_same_track_from_tracks || has_same_track_from_station
        };

        if overlapping_other_players_tracks || invalid_overlaps {
            CanBuildResponse::Invalid(BuildError::InvalidOverlap)
        } else if has_same_track {
            CanBuildResponse::AlreadyExists
        } else {
            // We aren't checking here if we can pay it - as this gets called often from track planning
            CanBuildResponse::Ok
        }
    }

    pub(crate) fn can_pay_costs(
        &self,
        player_id: PlayerId,
        costs: &BuildCosts,
    ) -> Result<(), BuildError> {
        for (industry_building_id, cargo_map) in &costs.costs {
            if let Some(industry_building) = self.industry_buildings.get(industry_building_id) {
                if industry_building.owner_id() != player_id {
                    // That's unexpected - why would the costs include other player's buildings?
                    return Err(BuildError::UnknownError);
                }
                if !industry_building.cargo().is_superset_of(cargo_map) {
                    return Err(BuildError::NotEnoughResources);
                }
            } else {
                return Err(BuildError::UnknownError);
            }
        }
        Ok(())
    }

    pub(crate) fn can_pay_known_cost<T: WithTileCoverage>(
        &self,
        player_id: PlayerId,
        something: &T,
        providing_industry_type: IndustryType,
        cost: CargoMap,
    ) -> Result<BuildCosts, BuildError> {
        let coverage = something.covers_tiles();
        if let Some(supply_range) = providing_industry_type.supply_range_in_tiles() {
            for building in
                self.find_industry_building_by_owner_and_type(player_id, providing_industry_type)
            {
                let distance = TileCoverage::manhattan_distance_between_closest_tiles(
                    &coverage,
                    &building.covers_tiles(),
                );
                if distance <= supply_range {
                    trace!(
                        "Supply building at distance {distance} with supply range {supply_range} has cargo {:?} and we need cost {cost:?}",
                        building.cargo()
                    );
                    if building.cargo().is_superset_of(&cost) {
                        // Later. We currently return the first one that satisfies the conditions - we could instead return the closest one, or the one with most resources.
                        return Ok(BuildCosts::single(building.id(), cost));
                    }
                }
            }

            Err(BuildError::NotEnoughResources)
        } else {
            Err(BuildError::UnknownError)
        }
    }

    pub(crate) fn can_pay_cost<T: WithCostToBuild + WithTileCoverage>(
        &self,
        player_id: PlayerId,
        something: &T,
    ) -> Result<BuildCosts, BuildError> {
        let (providing_industry_type, cost) = something.cost_to_build();

        self.can_pay_known_cost(player_id, something, providing_industry_type, cost)
    }

    pub fn can_build_for_coverage(&self, tile_coverage: &TileCoverage) -> Result<(), BuildError> {
        let invalid_overlaps = tile_coverage.to_set().into_iter().any(|tile| {
            self.building_at(tile).is_some() || self.tracks_at(tile) != MaybeTracksOnTile::Empty
        });

        invalid_overlaps.then_err_unit(|| BuildError::InvalidOverlap)
    }

    pub(crate) fn build_industry_building(
        &mut self,
        industry_building_info: &IndustryBuildingInfo,
        costs: BuildCosts,
    ) -> Result<(), BuildError> {
        self.can_build_with_coverage(industry_building_info)?;
        self.pay_costs(costs);
        self.append_industry_building(industry_building_info.clone());
        Ok(())
    }

    pub(crate) fn build_military_building(
        &mut self,
        military_building_info: &MilitaryBuildingInfo,
        costs: BuildCosts,
    ) -> Result<(), BuildError> {
        self.can_build_with_coverage(military_building_info)?;
        self.pay_costs(costs);
        self.append_military_building(military_building_info.clone());
        Ok(())
    }

    pub(crate) fn pay_costs(&mut self, costs: BuildCosts) {
        for (industry_building_id, cargo_map) in costs.costs {
            if let Some(industry_building) = self.industry_buildings.get_mut(&industry_building_id)
            {
                industry_building.remove_cargo(&cargo_map);
            } else {
                warn!("Could not find industry building with id {industry_building_id:?}");
            }
        }
    }

    pub(crate) fn build_station(
        &mut self,
        station_info: &StationInfo,
        costs: BuildCosts,
    ) -> Result<(), BuildError> {
        self.can_build_with_coverage(station_info)?;
        self.pay_costs(costs);
        self.append_station(station_info.clone());
        Ok(())
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
        if let Some(removed) = self.industry_buildings.remove(&industry_building_id) {
            for tile in removed.covers_tiles().to_set() {
                self.tile_industry_buildings[tile] = None;
            }
        }
        self.recalculate_cargo_forwarding_links();
    }

    pub fn remove_military_building(&mut self, military_building_id: MilitaryBuildingId) {
        if let Some(removed) = self.military_buildings.remove(&military_building_id) {
            for tile in removed.covers_tiles().to_set() {
                self.tile_military_buildings[tile] = None;
            }
        }
    }

    pub fn remove_station(&mut self, station_id: StationId) {
        if let Some(removed) = self.stations.remove(&station_id) {
            for tile in removed.covers_tiles().to_set() {
                self.tile_stations[tile] = None;
                if let Some(found) = self.station_track_types_at.get_mut(tile) {
                    found.clear();
                }
            }
        }
        self.recalculate_cargo_forwarding_links();
    }

    pub fn attempt_to_remove_tracks(
        &mut self,
        requesting_player_id: PlayerId,
        track_ids: &[TrackId],
    ) -> Result<(), ()> {
        self.tracks
            .attempt_to_remove_tracks(requesting_player_id, track_ids)
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

    pub fn attempt_to_remove_military_building(
        &mut self,
        requesting_player_id: PlayerId,
        military_building_id: MilitaryBuildingId,
    ) -> Result<(), ()> {
        let military_building = self
            .military_buildings
            .get(&military_building_id)
            .ok_or(())?;

        if military_building.owner_id() == requesting_player_id {
            self.remove_military_building(military_building_id);
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
        self.tracks.remove_track(track_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::building::station_type::StationType;

    #[test]
    fn test_disallow_build_station_over_tracks() {
        let size_x = 4;
        let size_z = 1;
        let mut building_state = BuildingState::new(size_x, size_z);
        let owner_id = PlayerId::random();
        let tile = TileCoordsXZ::new(2, 0);
        let track_type = TrackType::NorthWest;
        let track_info = TrackInfo::new(owner_id, tile, track_type);
        building_state.append_tracks(vec![track_info]);
        let station_info = StationInfo::new(
            owner_id,
            StationId::random(),
            TileCoordsXZ::new(0, 0),
            StationType::WE_1_4,
        );
        let result = building_state.can_build_with_coverage(&station_info);
        assert_eq!(result, Err(BuildError::InvalidOverlap));
    }
}
