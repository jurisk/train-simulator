#![allow(clippy::missing_errors_doc, clippy::result_unit_err)]

use std::collections::HashMap;

use crate::building_state::BuildingState;
use crate::map_level::MapLevel;
use crate::server_response::{GameInfo, PlayerInfo};
use crate::{BuildingInfo, GameId, PlayerId, TransportInfo};

#[derive(Debug, Copy, Clone, Default)]
pub struct GameTime(pub f32);

impl GameTime {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub struct GameState {
    game_id:    GameId,
    map_level:  MapLevel,
    buildings:  BuildingState,
    transports: Vec<TransportInfo>,
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
            transports,
            players,
            time: GameTime::new(),
            time_steps: 0,
        }
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

    pub fn advance_time(&mut self, time: GameTime) {
        let diff = time.0 - self.time.0;
        for transport in &mut self.transports {
            // Later: If game is paused then no need to advance transports
            transport.advance(diff, &self.buildings);
        }
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
        self.transports.clone()
    }

    #[must_use]
    pub fn building_infos(&self) -> Vec<BuildingInfo> {
        self.buildings.to_vec()
    }

    #[must_use]
    pub fn map_level(&self) -> MapLevel {
        self.map_level.clone()
    }

    #[must_use]
    pub fn players(&self) -> HashMap<PlayerId, PlayerInfo> {
        self.players.clone()
    }

    pub fn insert_player(&mut self, player_info: PlayerInfo) {
        self.players.insert(player_info.id, player_info);
    }

    pub fn remove_player(&mut self, player_id: PlayerId) {
        self.players.remove(&player_id);
    }

    pub fn insert_transport(&mut self, transport: TransportInfo) {
        self.transports.push(transport);
    }

    pub fn build_buildings(
        &mut self,
        requesting_player_id: PlayerId,
        buildings: Vec<BuildingInfo>,
    ) -> Result<(), ()> {
        self.buildings.build(requesting_player_id, buildings)
    }
}