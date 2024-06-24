use std::collections::HashMap;

use shared_domain::building_state::BuildingState;
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{GameInfo, PlayerInfo};
use shared_domain::{BuildingInfo, GameId, PlayerId, TransportInfo};

#[derive(Debug, Copy, Clone, Default)]
pub struct GameTime(pub f32);

impl GameTime {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

// TODO HIGH: Move to domain
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
    pub(crate) fn new(
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

    pub(crate) fn from_prototype(prototype: &GameState) -> Self {
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

    pub(crate) fn advance_time(&mut self, time: GameTime) {
        let diff = time.0 - self.time.0;
        for transport in &mut self.transports {
            // Later: If game is paused then no need to advance transports
            transport.advance(diff, &self.buildings);
        }
        self.time = time;
        self.time_steps += 1;
    }

    pub(crate) fn player_ids(&self) -> Vec<PlayerId> {
        self.players.keys().copied().collect()
    }

    pub(crate) fn create_game_info(&self) -> GameInfo {
        GameInfo {
            game_id: self.game_id,
            players: self.players.clone(),
        }
    }

    pub(crate) fn game_id(&self) -> GameId {
        self.game_id
    }

    pub(crate) fn time_steps(&self) -> u64 {
        self.time_steps
    }

    pub(crate) fn transport_infos(&self) -> Vec<TransportInfo> {
        self.transports.clone()
    }

    pub(crate) fn building_infos(&self) -> Vec<BuildingInfo> {
        self.buildings.to_vec()
    }

    pub(crate) fn map_level(&self) -> MapLevel {
        self.map_level.clone()
    }

    pub(crate) fn players(&self) -> HashMap<PlayerId, PlayerInfo> {
        self.players.clone()
    }

    pub(crate) fn insert_player(&mut self, player_info: PlayerInfo) {
        self.players.insert(player_info.id, player_info);
    }

    pub(crate) fn remove_player(&mut self, player_id: PlayerId) {
        self.players.remove(&player_id);
    }

    pub(crate) fn insert_transport(&mut self, transport: TransportInfo) {
        self.transports.push(transport);
    }

    pub(crate) fn build_buildings(
        &mut self,
        requesting_player_id: PlayerId,
        buildings: Vec<BuildingInfo>,
    ) -> Result<(), ()> {
        self.buildings.build(requesting_player_id, buildings)
    }
}
