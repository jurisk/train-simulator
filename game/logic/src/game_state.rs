#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::missing_errors_doc)]

use std::collections::HashMap;

use shared_domain::client_command::GameCommand;
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{
    AddressEnvelope, GameInfo, GameResponse, LobbyResponse, ServerResponse,
    ServerResponseWithAddress,
};
use shared_domain::{BuildingInfo, GameId, PlayerId, PlayerName};

#[derive(Debug, Clone)]
pub(crate) struct GameState {
    pub game_id: GameId,
    map_level:   MapLevel,
    buildings:   Vec<BuildingInfo>,
    players:     HashMap<PlayerId, PlayerName>,
}

impl GameState {
    pub(crate) fn new(
        map_level: MapLevel,
        buildings: Vec<BuildingInfo>,
        players: HashMap<PlayerId, PlayerName>,
    ) -> Self {
        let game_id = GameId::random();
        Self {
            game_id,
            map_level,
            buildings,
            players,
        }
    }

    pub(crate) fn from_prototype(prototype: &GameState) -> Self {
        let game_id = GameId::random();
        Self {
            game_id,
            map_level: prototype.map_level.clone(),
            buildings: prototype.buildings.clone(),
            players: prototype.players.clone(),
        }
    }

    pub(crate) fn join_game(
        &mut self,
        requesting_player_id: PlayerId,
        requesting_player_name: PlayerName,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        // Later: Don't allow joining multiple games

        self.players
            .insert(requesting_player_id, requesting_player_name);

        Ok(vec![
            ServerResponseWithAddress::new(
                AddressEnvelope::ToPlayer(requesting_player_id),
                ServerResponse::Lobby(LobbyResponse::GameJoined(self.game_id)),
            ),
            ServerResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(self.game_id),
                ServerResponse::Game(
                    self.game_id,
                    GameResponse::PlayersUpdated(self.players.clone()),
                ),
            ),
            ServerResponseWithAddress::new(
                AddressEnvelope::ToPlayer(requesting_player_id),
                ServerResponse::Game(
                    self.game_id,
                    GameResponse::MapLevelProvided(self.map_level.clone()),
                ),
            ),
        ])
    }

    pub(crate) fn process_command(
        &mut self,
        requesting_player_id: PlayerId,
        game_command: GameCommand,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        match game_command {
            GameCommand::BuildBuilding(building_info) => {
                // TODO: Check that `requesting_player_id` can build there, assign ownership

                self.buildings.push(building_info.clone());

                Ok(vec![ServerResponseWithAddress::new(
                    AddressEnvelope::ToAllPlayersInGame(self.game_id),
                    ServerResponse::Game(
                        self.game_id,
                        GameResponse::BuildingBuilt(building_info.clone()),
                    ),
                )])
            },
            GameCommand::QueryBuildings => {
                Ok(self
                    .buildings
                    .iter()
                    .map(|building_info| {
                        ServerResponseWithAddress::new(
                            AddressEnvelope::ToPlayer(requesting_player_id),
                            ServerResponse::Game(
                                self.game_id,
                                GameResponse::BuildingBuilt(building_info.clone()),
                            ),
                        )
                    })
                    .collect())
            },
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
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        self.players.remove(&player_id);
        Ok(vec![ServerResponseWithAddress::new(
            AddressEnvelope::ToAllPlayersInGame(self.game_id),
            ServerResponse::Lobby(LobbyResponse::GameLeft(self.game_id)),
        )])
    }
}
