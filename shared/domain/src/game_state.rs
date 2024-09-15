#![allow(clippy::missing_errors_doc, clippy::result_unit_err)]

use std::collections::HashMap;

use log::trace;
use serde::{Deserialize, Serialize, Serializer};

use crate::building::building_info::{BuildingDynamicInfo, WithCostToBuild, WithTileCoverage};
use crate::building::building_state::{BuildingState, CanBuildResponse};
use crate::building::industry_building_info::IndustryBuildingInfo;
use crate::building::station_info::StationInfo;
use crate::building::track_info::TrackInfo;
use crate::cargo_map::WithCargo;
use crate::game_time::{GameTime, GameTimeDiff};
use crate::map_level::map_level::{MapLevel, MapLevelFlattened};
use crate::map_level::zoning::ZoningInfo;
use crate::metrics::Metrics;
use crate::players::player_state::PlayerState;
use crate::server_response::{GameInfo, PlayerInfo};
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::transport::movement_orders::MovementOrders;
use crate::transport::track_type::TrackType;
use crate::transport::transport_info::{TransportDynamicInfo, TransportInfo};
use crate::transport::transport_state::TransportState;
use crate::{GameId, IndustryBuildingId, MapId, PlayerId, StationId, TrackId, TransportId};

// Later:   So this is used both on the server (to store authoritative game state), and on the client (to store the game state as known by the client).
//          So the API gets quite busy because of this. There may be better ways, such as splitting the validation-oriented methods into a server-only trait.
#[derive(Debug, PartialEq, Clone)]
pub struct GameState {
    game_id:    GameId,
    map_id:     MapId,
    map_level:  MapLevel,
    buildings:  BuildingState,
    transports: TransportState,
    // TODO HIGH: The players are actually 'Nation'-s or 'Polity'-s, and the players just control them. This is needed so we have scenarios / save games without specific players already assigned.
    players:    PlayerState,
    time:       GameTime,
    time_steps: u64,
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

#[derive(Serialize, Deserialize, Clone)]
struct GameStateFlattened {
    game_id:    GameId,
    map_id:     MapId,
    map_level:  MapLevelFlattened,
    buildings:  BuildingState,
    transports: TransportState,
    players:    PlayerState,
    time:       GameTime,
    time_steps: u64,
}

impl From<GameState> for GameStateFlattened {
    fn from(value: GameState) -> Self {
        Self {
            game_id:    value.game_id,
            map_id:     value.map_id,
            map_level:  value.map_level.into(),
            buildings:  value.buildings.clone(),
            transports: value.transports.clone(),
            players:    value.players.clone(),
            time:       value.time,
            time_steps: value.time_steps,
        }
    }
}

impl From<GameStateFlattened> for GameState {
    fn from(value: GameStateFlattened) -> Self {
        Self {
            game_id:    value.game_id,
            map_id:     value.map_id,
            map_level:  value.map_level.into(),
            buildings:  value.buildings,
            transports: value.transports.clone(),
            players:    value.players.clone(),
            time:       value.time,
            time_steps: value.time_steps,
        }
    }
}

impl GameState {
    #[must_use]
    pub fn empty_from_level(map_id: MapId, map_level: MapLevel) -> Self {
        let game_id = GameId::random();
        let terrain = map_level.terrain();
        let size_x = terrain.tile_count_x();
        let size_z = terrain.tile_count_z();
        Self {
            game_id,
            map_id,
            map_level,
            buildings: BuildingState::new(size_x, size_z),
            transports: TransportState::empty(),
            players: PlayerState::empty(),
            time: GameTime::new(),
            time_steps: 0,
        }
    }

    #[must_use]
    pub fn time(&self) -> GameTime {
        self.time
    }

    #[must_use]
    pub fn from_prototype(prototype: &GameState) -> Self {
        let game_id = GameId::random();
        Self {
            game_id,
            map_id: prototype.map_id.clone(),
            map_level: prototype.map_level.clone(),
            buildings: prototype.buildings.clone(),
            transports: prototype.transports.clone(),
            players: prototype.players.clone(),
            time: prototype.time,
            time_steps: prototype.time_steps,
        }
    }

    pub fn advance_time_diff(&mut self, diff: GameTimeDiff, metrics: &impl Metrics) {
        self.advance_time_diff_internal(diff, metrics);
        self.time = self.time + diff;
    }

    fn advance_time_diff_internal(&mut self, diff: GameTimeDiff, metrics: &impl Metrics) {
        // Later: If game is paused then no need to advance anything
        self.buildings.advance_time_diff(diff);
        self.transports
            .advance_time_diff(diff, &mut self.buildings, metrics);
    }

    pub fn advance_time(&mut self, time: GameTime, metrics: &impl Metrics) {
        let diff = time - self.time;
        self.advance_time_diff_internal(diff, metrics);
        self.time = time;
        self.time_steps += 1;
    }

    #[must_use]
    pub fn create_game_info(&self) -> GameInfo {
        GameInfo {
            map_id:  self.map_id.clone(),
            game_id: self.game_id,
            players: self.players.infos_cloned(),
        }
    }

    #[must_use]
    pub fn game_id(&self) -> GameId {
        self.game_id
    }

    #[must_use]
    pub fn time_steps(&self) -> u64 {
        self.time_steps
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

    pub fn update_player_infos(&mut self, new_player_infos: &Vec<PlayerInfo>) {
        self.players.update_many(new_player_infos);
    }

    pub fn insert_player(&mut self, player_info: PlayerInfo) {
        self.players.insert(player_info);
    }

    pub fn remove_player(&mut self, player_id: PlayerId) {
        self.players.remove(player_id);
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
    ) -> Result<Vec<TrackInfo>, ()> {
        // TODO HIGH: Subtract resource cost
        match self.can_build_tracks(requesting_player_id, tracks) {
            None => Err(()),
            Some(filtered) => {
                self.buildings.append_tracks(filtered.clone());
                Ok(filtered)
            },
        }
    }

    pub fn can_build_tracks(
        &mut self,
        requesting_player_id: PlayerId,
        track_infos: &[TrackInfo],
    ) -> Option<Vec<TrackInfo>> {
        // TODO HIGH: Check you have resources to build
        let mut results = vec![];
        for track_info in track_infos {
            match self.can_build_track(requesting_player_id, track_info) {
                CanBuildResponse::Ok => {
                    results.push(track_info.clone());
                },
                CanBuildResponse::AlreadyExists => {},
                CanBuildResponse::Invalid => {
                    return None;
                },
            }
        }
        Some(results)
    }

    pub(crate) fn can_build_track(
        &self,
        player_id: PlayerId,
        track: &TrackInfo,
    ) -> CanBuildResponse {
        if track.owner_id() == player_id {
            self.can_build_track_internal(player_id, track.tile, track.track_type)
        } else {
            CanBuildResponse::Invalid
        }
    }

    pub(crate) fn can_build_track_internal(
        &self,
        player_id: PlayerId,
        tile: TileCoordsXZ,
        track_type: TrackType,
    ) -> CanBuildResponse {
        if self.map_level.can_build_track(tile, track_type) {
            self.buildings.can_build_track(player_id, tile, track_type)
        } else {
            CanBuildResponse::Invalid
        }
    }

    #[must_use]
    pub fn can_build_industry_building(
        &self,
        requesting_player_id: PlayerId,
        building: &IndustryBuildingInfo,
    ) -> bool {
        self.can_pay_cost(requesting_player_id, building).is_some()
            && self.map_level.can_build_industry_building(building)
            && self
                .buildings
                .can_build_building(requesting_player_id, building)
    }

    pub fn build_industry_building(
        &mut self,
        requesting_player_id: PlayerId,
        building: &IndustryBuildingInfo,
    ) -> Result<(), ()> {
        // TODO HIGH: Subtract resource cost
        if self.can_build_industry_building(requesting_player_id, building) {
            self.buildings
                .build_industry_building(requesting_player_id, building)
        } else {
            Err(())
        }
    }

    #[must_use]
    pub fn can_build_station(&self, requesting_player_id: PlayerId, station: &StationInfo) -> bool {
        self.can_pay_cost(requesting_player_id, station).is_some()
            && self.map_level.can_build_station(station)
            && self
                .buildings
                .can_build_building(requesting_player_id, station)
    }

    #[expect(clippy::collapsible_if)]
    fn can_pay_cost<T: WithCostToBuild + WithTileCoverage>(
        &self,
        player_id: PlayerId,
        something: &T,
    ) -> Option<IndustryBuildingId> {
        let (industry_type, cost) = something.cost_to_build();
        let coverage = something.covers_tiles();
        if let Some(supply_range) = industry_type.supply_range_in_tiles() {
            for building in self
                .buildings
                .find_industry_building_by_owner_and_type(player_id, industry_type)
            {
                let distance = TileCoverage::manhattan_distance_between_closest_tiles(
                    &coverage,
                    &building.covers_tiles(),
                );
                if distance <= supply_range {
                    if building.cargo().is_superset_of(&cost) {
                        // Later. We currently return the first one that satisfies the conditions - we could instead return the closest one, or the one with most resources.
                        return Some(building.id());
                    }
                }
            }

            None
        } else {
            None
        }
    }

    pub fn build_station(
        &mut self,
        requesting_player_id: PlayerId,
        station: &StationInfo,
    ) -> Result<(), ()> {
        // TODO HIGH: Subtract resource cost
        if self.can_build_station(requesting_player_id, station) {
            self.buildings.build_station(requesting_player_id, station)
        } else {
            Err(())
        }
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

    #[must_use]
    pub fn all_free_zonings(&self) -> Vec<&ZoningInfo> {
        self.map_level
            .zoning()
            .all_zonings()
            .into_iter()
            .filter(|zoning| {
                self.building_state()
                    .industry_building_at(zoning.reference_tile())
                    .is_none()
            })
            .collect()
    }
}
