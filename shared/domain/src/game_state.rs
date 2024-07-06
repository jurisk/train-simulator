#![allow(clippy::missing_errors_doc, clippy::result_unit_err)]

use std::collections::HashMap;

use log::trace;
use serde::{Deserialize, Serialize};

use crate::building_info::BuildingInfo;
use crate::building_state::BuildingState;
use crate::game_time::{GameTime, GameTimeDiff};
use crate::map_level::MapLevel;
use crate::server_response::{GameInfo, PlayerInfo};
use crate::transport_info::{TransportDynamicInfo, TransportInfo};
use crate::transport_state::TransportState;
use crate::{GameId, PlayerId, TransportId};

// Later:   So this is used both on the server (to store authoritative game state), and on the client (to store the game state as known by the client).
//          So the API gets quite busy because of this. There may be better ways.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct GameState {
    game_id:    GameId,
    map_level:  MapLevel,
    buildings:  BuildingState,
    transports: TransportState,
    players:    HashMap<PlayerId, PlayerInfo>,
    time:       GameTime,
    time_steps: u64,
}

impl GameState {
    #[must_use]
    pub fn new(
        map_level: MapLevel,
        buildings: Vec<BuildingInfo>,
        transports: Vec<TransportInfo>,
        players: HashMap<PlayerId, PlayerInfo>,
    ) -> Self {
        let game_id = GameId::random();
        Self {
            game_id,
            map_level,
            buildings: BuildingState::from_vec(buildings),
            transports: TransportState::from_vec(transports),
            players,
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
            map_level: prototype.map_level.clone(),
            buildings: prototype.buildings.clone(),
            transports: prototype.transports.clone(),
            players: prototype.players.clone(),
            time: prototype.time,
            time_steps: prototype.time_steps,
        }
    }

    pub fn advance_time_diff(&mut self, diff: GameTimeDiff) {
        self.advance_time_diff_internal(diff);
        self.time = self.time + diff;
    }

    fn advance_time_diff_internal(&mut self, diff: GameTimeDiff) {
        // Later: If game is paused then no need to advance transports
        self.transports.advance_time_diff(diff, &self.buildings);
    }

    pub fn advance_time(&mut self, time: GameTime) {
        let diff = time - self.time;
        self.advance_time_diff_internal(diff);
        self.time = time;
        self.time_steps += 1;
    }

    #[must_use]
    pub fn player_ids(&self) -> Vec<PlayerId> {
        self.players.keys().copied().collect()
    }

    #[must_use]
    pub fn create_game_info(&self) -> GameInfo {
        GameInfo {
            game_id: self.game_id,
            players: self.players.clone(),
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
    pub fn transport_infos(&self) -> Vec<TransportInfo> {
        self.transports.to_vec()
    }

    #[must_use]
    pub fn building_infos(&self) -> Vec<BuildingInfo> {
        self.buildings.to_vec()
    }

    #[must_use]
    pub fn map_level(&self) -> &MapLevel {
        &self.map_level
    }

    #[must_use]
    pub fn players(&self) -> &HashMap<PlayerId, PlayerInfo> {
        &self.players
    }

    pub fn update_player_infos(&mut self, new_player_infos: &HashMap<PlayerId, PlayerInfo>) {
        self.players.clone_from(new_player_infos);
    }

    pub fn insert_player(&mut self, player_info: PlayerInfo) {
        self.players.insert(player_info.id, player_info);
    }

    pub fn remove_player(&mut self, player_id: PlayerId) {
        self.players.remove(&player_id);
    }

    pub fn upsert_transport(&mut self, transport: TransportInfo) {
        self.transports.upsert(transport);
    }

    pub fn append_buildings(&mut self, buildings: Vec<BuildingInfo>) {
        self.buildings.append_all(buildings);
    }

    pub fn build_buildings(
        &mut self,
        requesting_player_id: PlayerId,
        buildings: Vec<BuildingInfo>,
    ) -> Result<(), ()> {
        self.buildings
            .build(requesting_player_id, buildings, &self.map_level)
    }

    #[must_use]
    pub fn building_state(&self) -> &BuildingState {
        &self.buildings
    }

    pub fn update_transport_dynamic_infos(
        &mut self,
        server_time: GameTime,
        dynamic_infos: &HashMap<TransportId, TransportDynamicInfo>,
    ) {
        let diff = server_time - self.time;
        trace!(
            "Updated dynamic infos, diff {:?}, old {:?}, new {:?}, {} transports",
            diff,
            self.time,
            server_time,
            dynamic_infos.len(),
        );
        self.time = server_time;
        for (transport_id, transport_dynamic_info) in dynamic_infos {
            self.update_transport_dynamic_info(*transport_id, transport_dynamic_info);
        }
    }

    fn update_transport_dynamic_info(
        &mut self,
        transport_id: TransportId,
        transport_dynamic_info: &TransportDynamicInfo,
    ) {
        self.transports
            .update_dynamic_info(transport_id, transport_dynamic_info);
    }

    #[must_use]
    pub fn get_transport_info(&self, transport_id: TransportId) -> Option<&TransportInfo> {
        self.transports.info_by_id(transport_id)
    }
}
