#![allow(clippy::unnecessary_wraps, clippy::missing_errors_doc)]

use std::collections::HashMap;

use shared_domain::building_state::BuildingState;
use shared_domain::client_command::GameCommand;
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{AddressEnvelope, Colour, GameInfo, GameResponse, PlayerInfo};
use shared_domain::{BuildingInfo, GameId, PlayerId, PlayerName, TransportInfo};

#[derive(Debug, Copy, Clone, Default)]
pub struct GameTime(pub f32);

impl GameTime {
    #[must_use]
    pub fn new() -> Self {
        Self(0.0)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GameState {
    pub game_id: GameId,
    map_level:   MapLevel,
    buildings:   BuildingState,
    transports:  Vec<TransportInfo>,
    players:     HashMap<PlayerId, PlayerInfo>,
    time:        GameTime,
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
        }
    }

    pub(crate) fn advance_time(&mut self, time: GameTime) {
        let diff = time.0 - self.time.0;
        for transport in &mut self.transports {
            // Later: If game is paused then no need to advance transports
            transport.advance(diff, &self.buildings);
        }
        self.time = time;
    }

    pub(crate) fn join_game(
        &mut self,
        requesting_player_id: PlayerId,
        requesting_player_name: PlayerName,
    ) -> Result<Vec<GameResponseWithAddress>, GameResponse> {
        // Later: Don't allow joining multiple games

        let colour = Colour::random();
        let requesting_player_info = PlayerInfo {
            id: requesting_player_id,
            name: requesting_player_name,
            colour,
        };

        self.players
            .insert(requesting_player_id, requesting_player_info);

        Ok(vec![
            GameResponseWithAddress::new(
                AddressEnvelope::ToPlayer(requesting_player_id),
                GameResponse::GameJoined,
            ),
            GameResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(self.game_id),
                GameResponse::PlayersUpdated(self.players.clone()),
            ),
            GameResponseWithAddress::new(
                AddressEnvelope::ToPlayer(requesting_player_id),
                GameResponse::MapLevelProvided(self.map_level.clone()),
            ),
        ])
    }

    pub(crate) fn process_command(
        &mut self,
        requesting_player_id: PlayerId,
        game_command: GameCommand,
    ) -> Result<Vec<GameResponseWithAddress>, GameResponse> {
        match game_command {
            GameCommand::PurchaseTransport(transport_info) => {
                self.process_purchase_transport(requesting_player_id, transport_info)
            },
            GameCommand::BuildBuildings(building_infos) => {
                self.process_build_buildings(requesting_player_id, building_infos)
            },
            GameCommand::QueryBuildings => self.process_query_buildings(requesting_player_id),
            GameCommand::QueryTransports => self.process_query_transports(requesting_player_id),
        }
    }

    fn process_query_transports(
        &mut self,
        requesting_player_id: PlayerId,
    ) -> Result<Vec<GameResponseWithAddress>, GameResponse> {
        Ok(vec![GameResponseWithAddress::new(
            AddressEnvelope::ToPlayer(requesting_player_id),
            GameResponse::TransportsExist(self.transports.clone()),
        )])
    }

    fn process_query_buildings(
        &mut self,
        requesting_player_id: PlayerId,
    ) -> Result<Vec<GameResponseWithAddress>, GameResponse> {
        Ok(vec![GameResponseWithAddress::new(
            AddressEnvelope::ToPlayer(requesting_player_id),
            GameResponse::BuildingsBuilt(self.buildings.to_vec()),
        )])
    }

    fn process_build_buildings(
        &mut self,
        requesting_player_id: PlayerId,
        building_infos: Vec<BuildingInfo>,
    ) -> Result<Vec<GameResponseWithAddress>, GameResponse> {
        let valid_player_id = building_infos
            .iter()
            .all(|building_info| building_info.owner_id == requesting_player_id);

        // TODO: Check that this is a valid building and there is enough money to build it, subtract money
        // TODO: Check that terrain matches building requirements - e.g. no building on water, tracks that go out of bounds, tracks that go into water, etc.

        let tiles_are_free = building_infos.iter().all(|building_infos| {
            building_infos
                .covers_tiles
                .to_set()
                .into_iter()
                .all(|tile| {
                    // Later: Actually, we should allow adding a track to tracks if such a track type are not already present!
                    self.buildings.buildings_at(tile).is_empty()
                })
        });

        if valid_player_id && tiles_are_free {
            self.buildings.append(building_infos.clone());

            Ok(vec![GameResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(self.game_id),
                GameResponse::BuildingsBuilt(building_infos),
            )])
        } else {
            Err(GameResponse::CannotBuild(
                building_infos
                    .into_iter()
                    .map(|building_info| building_info.building_id)
                    .collect(),
            ))
        }
    }

    fn process_purchase_transport(
        &mut self,
        requesting_player_id: PlayerId,
        transport_info: TransportInfo,
    ) -> Result<Vec<GameResponseWithAddress>, GameResponse> {
        if requesting_player_id == transport_info.owner_id {
            // TODO: Check if the track / road / etc. is free and owned by the purchaser
            // TODO: Subtract money

            self.transports.push(transport_info.clone());
            Ok(vec![GameResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(self.game_id),
                GameResponse::TransportsExist(vec![transport_info]),
            )])
        } else {
            Err(GameResponse::CannotPurchase(transport_info.transport_id))
        }
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

    pub(crate) fn remove_player(
        &mut self,
        player_id: PlayerId,
    ) -> Result<Vec<GameResponseWithAddress>, GameResponse> {
        self.players.remove(&player_id);
        Ok(vec![GameResponseWithAddress::new(
            AddressEnvelope::ToAllPlayersInGame(self.game_id),
            GameResponse::GameLeft,
        )])
    }
}

#[derive(Clone)]
pub(crate) struct GameResponseWithAddress {
    pub address:  AddressEnvelope,
    pub response: GameResponse,
}

impl GameResponseWithAddress {
    fn new(address: AddressEnvelope, response: GameResponse) -> Self {
        Self { address, response }
    }
}
