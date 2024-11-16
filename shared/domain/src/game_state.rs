#![allow(clippy::missing_errors_doc, clippy::result_unit_err)]

use std::collections::{HashMap, HashSet};

use log::{debug, trace};
use serde::{Deserialize, Serialize, Serializer};
use shared_util::bool_ops::BoolResultOps;

use crate::building::building_info::{
    BuildingDynamicInfo, WithCostToBuild, WithOwner, WithTileCoverage,
};
use crate::building::building_state::{BuildingState, CanBuildResponse};
use crate::building::industry_building_info::IndustryBuildingInfo;
use crate::building::military_building_info::MilitaryBuildingInfo;
use crate::building::station_info::StationInfo;
use crate::building::track_info::TrackInfo;
use crate::building::{BuildCosts, BuildError};
use crate::game_time::{GameTime, GameTimeDiff, TimeFactor};
use crate::map_level::map_level::{MapLevel, MapLevelFlattened};
use crate::map_level::zoning::ZoningInfo;
use crate::metrics::Metrics;
use crate::players::player_state::PlayerState;
use crate::scenario::{PlayerProfile, Scenario};
use crate::supply_chain::SupplyChain;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::transport::movement_orders::MovementOrders;
use crate::transport::track_type::TrackType;
use crate::transport::transport_info::{TransportDynamicInfo, TransportInfo};
use crate::transport::transport_state::TransportState;
use crate::{
    GameId, IndustryBuildingId, MilitaryBuildingId, PlayerId, ScenarioId, StationId, TrackId,
    TransportId,
};

// Later:   So this is used both on the server (to store authoritative game state), and on the client (to store the game state as known by the client).
//          So the API gets quite busy because of this. There may be better ways, such as splitting the validation-oriented methods into a server-only trait.
#[derive(Debug, PartialEq, Clone)]
pub struct GameState {
    pub game_id: GameId,
    pub scenario_id: ScenarioId,
    map_level: MapLevel,
    buildings: BuildingState,
    transports: TransportState,
    players: PlayerState,
    supply_chain: SupplyChain,
    time: GameTime,
    time_factor: TimeFactor,
    ignore_requesting_player_id: bool,
}

impl Serialize for GameState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let flattened = GameStateFlattened::from(self.clone());
        flattened.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GameState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let flattened = GameStateFlattened::deserialize(deserializer)?;
        let game_state: GameState = flattened.into();
        Ok(game_state)
    }
}

#[expect(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Clone)]
pub struct GameStateFlattened {
    game_id:     GameId,
    scenario_id: ScenarioId,
    map_level:   MapLevelFlattened,
    buildings:   BuildingState,
    transports:  TransportState,
    players:     PlayerState,
    time:        GameTime,
    time_factor: TimeFactor,
}

impl From<GameState> for GameStateFlattened {
    fn from(value: GameState) -> Self {
        Self {
            game_id:     value.game_id,
            scenario_id: value.scenario_id.clone(),
            map_level:   value.map_level.into(),
            buildings:   value.buildings.clone(),
            transports:  value.transports.clone(),
            players:     value.players.clone(),
            time:        value.time,
            time_factor: value.time_factor,
        }
    }
}

impl From<GameStateFlattened> for GameState {
    fn from(value: GameStateFlattened) -> Self {
        Self {
            game_id: value.game_id,
            scenario_id: value.scenario_id,
            map_level: value.map_level.into(),
            buildings: value.buildings,
            transports: value.transports.clone(),
            players: value.players.clone(),
            supply_chain: SupplyChain::new(),
            time: value.time,
            time_factor: value.time_factor,
            ignore_requesting_player_id: false,
        }
    }
}

impl GameState {
    #[must_use]
    pub fn from_scenario(scenario: Scenario, ignore_requesting_player_id: bool) -> Self {
        let game_id = GameId::random();
        let terrain = scenario.map_level.terrain();
        let size_x = terrain.tile_count_x();
        let size_z = terrain.tile_count_z();

        let player_infos = scenario
            .players
            .iter()
            .map(PlayerProfile::to_player_info)
            .collect();
        let players = PlayerState::from_infos(player_infos);

        let mut result = Self {
            game_id,
            scenario_id: scenario.scenario_id,
            map_level: scenario.map_level,
            buildings: BuildingState::new(size_x, size_z),
            transports: TransportState::empty(),
            players,
            supply_chain: SupplyChain::new(),
            time: GameTime::new(),
            time_factor: TimeFactor::default(),
            ignore_requesting_player_id,
        };

        for player in scenario.players {
            // Later: Fix hack where we do `SupplyChain::new()` twice
            result.building_state_mut().gift_initial_construction_yard(
                player.player_id,
                player.initial_construction_yard,
                &SupplyChain::new(),
            );
        }

        result
    }

    pub fn set_time_factor(&mut self, time_factor: TimeFactor) {
        self.time_factor = time_factor;
    }

    #[must_use]
    pub fn time_factor(&self) -> TimeFactor {
        self.time_factor
    }

    #[must_use]
    pub fn time(&self) -> GameTime {
        self.time
    }

    pub fn advance_time_diff(&mut self, diff: GameTimeDiff, metrics: &impl Metrics) {
        let diff = diff * self.time_factor;
        if diff != GameTimeDiff::ZERO {
            self.buildings.advance_time_diff(diff);
            self.transports
                .advance_time_diff(diff, &mut self.buildings, metrics);
            self.time = self.time + diff;
        }
    }

    #[must_use]
    pub fn game_id(&self) -> GameId {
        self.game_id
    }

    #[must_use]
    pub fn scenario_id(&self) -> ScenarioId {
        self.scenario_id.clone()
    }

    #[must_use]
    pub fn transport_infos(&self) -> &Vec<TransportInfo> {
        self.transports.all_transports()
    }

    #[must_use]
    pub fn track_infos(&self) -> Vec<TrackInfo> {
        self.buildings.all_track_infos()
    }

    #[must_use]
    pub fn map_level(&self) -> &MapLevel {
        &self.map_level
    }

    #[must_use]
    pub fn players(&self) -> &PlayerState {
        &self.players
    }

    pub fn upsert_transport(&mut self, transport: TransportInfo) {
        self.transports.upsert(transport);
    }

    pub fn update_transport_movement_orders(
        &mut self,
        transport_id: TransportId,
        movement_orders: &MovementOrders,
    ) -> Result<(), ()> {
        self.transports
            .update_movement_orders(transport_id, movement_orders)
    }

    pub fn build_tracks(
        &mut self,
        requesting_player_id: PlayerId,
        tracks: &[TrackInfo],
    ) -> Result<Vec<TrackInfo>, BuildError> {
        let (filtered, costs) = self.can_build_tracks(requesting_player_id, tracks)?;
        self.buildings.build_tracks(filtered.clone(), costs);
        Ok(filtered)
    }

    pub fn can_purchase_transport(
        &self,
        requesting_player_id: PlayerId,
        station_id: StationId,
        transport_info: &TransportInfo,
    ) -> Result<BuildCosts, BuildError> {
        // TODO: Check if the track / road / etc. is free and owned by the purchaser
        // TODO: Check if the transport is on a station

        self.valid_owner(requesting_player_id, transport_info.owner_id())?;

        let (source_industry, cargo_map) = transport_info.cost_to_build();
        let station = self
            .building_state()
            .find_station(station_id)
            .ok_or(BuildError::UnknownError)?;
        let costs = self.building_state().can_pay_known_cost(
            transport_info.owner_id(),
            station,
            source_industry,
            cargo_map,
        )?;

        Ok(costs)
    }

    pub fn purchase_transport(
        &mut self,
        requesting_player_id: PlayerId,
        station_id: StationId,
        transport_info: &TransportInfo,
    ) -> Result<(), BuildError> {
        let costs =
            self.can_purchase_transport(requesting_player_id, station_id, transport_info)?;

        self.buildings.pay_costs(costs);
        self.upsert_transport(transport_info.clone());

        Ok(())
    }

    #[expect(clippy::missing_panics_doc, clippy::unwrap_used)]
    pub fn can_build_tracks(
        &self,
        requesting_player_id: PlayerId,
        track_infos: &[TrackInfo],
    ) -> Result<(Vec<TrackInfo>, BuildCosts), BuildError> {
        let mut results = vec![];
        let mut costs = BuildCosts::none();
        let mut player_ids = HashSet::new();
        for track_info in track_infos {
            match self.can_build_track(requesting_player_id, track_info) {
                CanBuildResponse::Ok => {
                    let player_id = track_info.owner_id();
                    player_ids.insert(player_id);
                    results.push(track_info.clone());
                    match self.can_pay_cost(player_id, track_info) {
                        Ok(cost) => {
                            costs += cost;
                        },
                        Err(err) => return Err(err),
                    }
                },
                CanBuildResponse::AlreadyExists => {},
                CanBuildResponse::Invalid(error) => {
                    return Err(error);
                },
            }
        }

        (player_ids.len() == 1).then_ok_unit(|| BuildError::InvalidOwner)?;
        let player_id = player_ids.iter().next().unwrap();

        debug!("Aggregated track costs: {:?}", costs);
        self.can_pay_costs(*player_id, &costs)?;
        Ok((results, costs))
    }

    pub(crate) fn can_build_track(
        &self,
        requesting_player_id: PlayerId,
        track: &TrackInfo,
    ) -> CanBuildResponse {
        match self.valid_owner(requesting_player_id, track.owner_id()) {
            Ok(()) => self.can_build_track_internal(track.owner_id(), track.tile, track.track_type),
            Err(err) => CanBuildResponse::Invalid(err),
        }
    }

    pub(crate) fn can_build_track_internal(
        &self,
        owner_id: PlayerId,
        tile: TileCoordsXZ,
        track_type: TrackType,
    ) -> CanBuildResponse {
        match self.map_level.can_build_track(tile, track_type) {
            Ok(()) => self.buildings.can_build_track(owner_id, tile, track_type),
            Err(err) => CanBuildResponse::Invalid(err),
        }
    }

    #[expect(clippy::missing_errors_doc)]
    pub fn can_build_industry_building(
        &self,
        requesting_player_id: PlayerId,
        building: &IndustryBuildingInfo,
    ) -> Result<BuildCosts, BuildError> {
        self.valid_owner(requesting_player_id, building.owner_id())?;
        self.map_level.can_build_industry_building(building)?;
        self.buildings.can_build_with_coverage(building)?;

        self.can_pay_cost(building.owner_id(), building)
    }

    #[expect(clippy::missing_errors_doc)]
    pub fn can_build_military_building(
        &self,
        requesting_player_id: PlayerId,
        building: &MilitaryBuildingInfo,
    ) -> Result<BuildCosts, BuildError> {
        self.valid_owner(requesting_player_id, building.owner_id())?;
        self.map_level.can_build_military_building(building)?;
        self.buildings.can_build_with_coverage(building)?;

        self.can_pay_cost(building.owner_id(), building)
    }

    pub fn build_industry_building(
        &mut self,
        requesting_player_id: PlayerId,
        building: &IndustryBuildingInfo,
    ) -> Result<(), BuildError> {
        let costs = self.can_build_industry_building(requesting_player_id, building)?;
        self.buildings.build_industry_building(building, costs)
    }

    pub fn build_military_building(
        &mut self,
        requesting_player_id: PlayerId,
        building: &MilitaryBuildingInfo,
    ) -> Result<(), BuildError> {
        let costs = self.can_build_military_building(requesting_player_id, building)?;
        self.buildings.build_military_building(building, costs)
    }

    #[expect(clippy::missing_errors_doc)]
    pub fn can_build_station(
        &self,
        requesting_player_id: PlayerId,
        station: &StationInfo,
    ) -> Result<BuildCosts, BuildError> {
        self.valid_owner(requesting_player_id, station.owner_id())?;
        self.map_level.can_build_station(station)?;
        self.buildings.can_build_with_coverage(station)?;
        self.can_pay_cost(station.owner_id(), station)
    }

    fn can_pay_cost<T: WithCostToBuild + WithTileCoverage>(
        &self,
        player_id: PlayerId,
        something: &T,
    ) -> Result<BuildCosts, BuildError> {
        self.buildings.can_pay_cost(player_id, something)
    }

    fn can_pay_costs(&self, player_id: PlayerId, costs: &BuildCosts) -> Result<(), BuildError> {
        self.buildings.can_pay_costs(player_id, costs)
    }

    pub fn build_station(
        &mut self,
        requesting_player_id: PlayerId,
        station: &StationInfo,
    ) -> Result<(), BuildError> {
        let costs = self.can_build_station(requesting_player_id, station)?;
        self.buildings.build_station(station, costs)
    }

    pub fn remove_tracks(
        &mut self,
        requesting_player_id: PlayerId,
        track_ids: &[TrackId],
    ) -> Result<(), ()> {
        // TODO: Check there are no trains on (or near?) these tracks
        self.buildings
            .attempt_to_remove_tracks(requesting_player_id, track_ids)
    }

    pub fn remove_industry_building(
        &mut self,
        requesting_player_id: PlayerId,
        industry_building_id: IndustryBuildingId,
    ) -> Result<(), ()> {
        self.buildings
            .attempt_to_remove_industry_building(requesting_player_id, industry_building_id)
    }

    pub fn remove_station(
        &mut self,
        requesting_player_id: PlayerId,
        station_id: StationId,
    ) -> Result<(), ()> {
        self.buildings
            .attempt_to_remove_station(requesting_player_id, station_id)
    }

    pub fn remove_military_building(
        &mut self,
        requesting_player_id: PlayerId,
        military_building_id: MilitaryBuildingId,
    ) -> Result<(), ()> {
        self.buildings
            .attempt_to_remove_military_building(requesting_player_id, military_building_id)
    }

    #[must_use]
    pub fn building_state(&self) -> &BuildingState {
        &self.buildings
    }

    #[must_use]
    pub fn building_state_mut(&mut self) -> &mut BuildingState {
        &mut self.buildings
    }

    pub fn update_dynamic_infos(
        &mut self,
        server_time: GameTime,
        industry_building_dynamic_infos: &HashMap<IndustryBuildingId, BuildingDynamicInfo>,
        station_dynamic_infos: &HashMap<StationId, BuildingDynamicInfo>,
        transport_dynamic_infos: &HashMap<TransportId, TransportDynamicInfo>,
    ) {
        let diff = server_time - self.time;
        trace!(
            "Updated dynamic infos, diff {:?}, old {:?}, new {:?}, {} buildings, {} stations, {} transports",
            diff,
            self.time,
            server_time,
            industry_building_dynamic_infos.len(),
            station_dynamic_infos.len(),
            transport_dynamic_infos.len(),
        );
        self.time = server_time;
        for (transport_id, transport_dynamic_info) in transport_dynamic_infos {
            self.transports
                .update_dynamic_info(*transport_id, transport_dynamic_info);
        }
        self.buildings
            .update_dynamic_infos(industry_building_dynamic_infos, station_dynamic_infos);
    }

    #[must_use]
    pub fn get_transport_info(&self, transport_id: TransportId) -> Option<&TransportInfo> {
        self.transports.info_by_id(transport_id)
    }

    #[must_use]
    pub fn transport_state(&self) -> &TransportState {
        &self.transports
    }

    pub fn all_free_zonings(&self) -> impl Iterator<Item = &ZoningInfo> {
        self.map_level
            .zoning()
            .all_zonings()
            .into_iter()
            .filter(|zoning| {
                self.building_state()
                    .industry_building_at(zoning.reference_tile())
                    .is_none()
            })
    }

    fn valid_owner(
        &self,
        requesting_player_id: PlayerId,
        owner_id: PlayerId,
    ) -> Result<(), BuildError> {
        // Hack used to let single-player AI build on behalf of all players... Didn't think of a more elegant way.
        if self.ignore_requesting_player_id {
            Ok(())
        } else {
            (owner_id == requesting_player_id).then_ok_unit(|| BuildError::InvalidOwner)
        }
    }

    #[must_use]
    pub fn supply_chain(&self) -> &SupplyChain {
        &self.supply_chain
    }
}
