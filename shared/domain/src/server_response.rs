use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::game_state::GameState;
use crate::{BuildingInfo, ClientId, GameId, PlayerId, PlayerName};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum AuthenticationResponse {
    LoginSucceeded(PlayerId),
    LoginFailed,
    LogoutSucceeded,
    LogoutFailed,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct GameInfo {
    pub game_id: GameId,
    pub players: HashMap<PlayerId, PlayerName>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum LobbyResponse {
    AvailableGames(Vec<GameInfo>),
    GameJoined(GameInfo),
    GameLeft(GameId),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum GameResponse {
    State(GameState),
    BuildingBuilt(BuildingInfo),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ServerError {
    JoinFailedAlreadyInGame,
    LeaveFailedNotInGame,
    GameNotFound,
    NotAuthorized,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ServerResponse {
    Authentication(AuthenticationResponse),
    Lobby(LobbyResponse),
    Game(GameResponse),
    Error(ServerError),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum AddressEnvelope {
    ToClient(ClientId),
    ToPlayer(PlayerId),
    ToAllPlayersInGame(GameId),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct ServerResponseWithAddress {
    pub address:  AddressEnvelope,
    pub response: ServerResponse,
}

impl ServerResponseWithAddress {
    #[must_use]
    pub fn new(address: AddressEnvelope, response: ServerResponse) -> Self {
        Self { address, response }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct ServerResponseWithClientIds {
    pub client_ids: Vec<ClientId>,
    pub response:   ServerResponse,
}

impl ServerResponseWithClientIds {
    #[must_use]
    pub fn new(client_ids: Vec<ClientId>, response: ServerResponse) -> Self {
        Self {
            client_ids,
            response,
        }
    }
}