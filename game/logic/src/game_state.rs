#![allow(clippy::missing_errors_doc)]

use std::collections::HashMap;

use shared_domain::client_command::GameCommand;
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{
    AddressEnvelope, GameResponse, LobbyResponse, ServerResponse, ServerResponseWithAddress,
};
use shared_domain::{BuildingInfo, GameId, PlayerId, PlayerName};

#[derive(Debug, Clone)]
pub struct GameState {
    pub map_level: MapLevel,
    pub buildings: Vec<BuildingInfo>,
    pub players:   HashMap<PlayerId, PlayerName>,
}

impl GameState {
    pub fn join_game(
        &mut self,
        game_id: GameId,
        requesting_player_id: PlayerId,
        requesting_player_name: PlayerName,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        // Later: Don't allow joining multiple games

        self.players
            .insert(requesting_player_id, requesting_player_name);

        Ok(vec![
            ServerResponseWithAddress::new(
                AddressEnvelope::ToPlayer(requesting_player_id),
                ServerResponse::Lobby(LobbyResponse::GameJoined(game_id)),
            ),
            ServerResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(game_id),
                ServerResponse::Game(game_id, GameResponse::PlayersUpdated(self.players.clone())),
            ),
            ServerResponseWithAddress::new(
                AddressEnvelope::ToPlayer(requesting_player_id),
                ServerResponse::Game(
                    game_id,
                    GameResponse::MapLevelProvided(self.map_level.clone()),
                ),
            ),
        ])
    }

    pub fn process_game_command(
        &mut self,
        game_id: GameId,
        requesting_player_id: PlayerId,
        game_command: GameCommand,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        match game_command {
            GameCommand::BuildBuilding(building_info) => {
                // TODO: Check that `requesting_player_id` can build there, assign ownership

                self.buildings.push(building_info.clone());

                Ok(vec![ServerResponseWithAddress::new(
                    AddressEnvelope::ToAllPlayersInGame(game_id),
                    ServerResponse::Game(
                        game_id,
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
                                game_id,
                                GameResponse::BuildingBuilt(building_info.clone()),
                            ),
                        )
                    })
                    .collect())
            },
        }
    }
}
