use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

use fastrand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::map_level::MapLevel;
use crate::{
    BuildingId, BuildingInfo, ClientId, GameId, PlayerId, PlayerName, TransportId, TransportInfo,
};

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
    pub players: HashMap<PlayerId, PlayerInfo>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum LobbyResponse {
    AvailableGames(Vec<GameInfo>),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Colour {
    #[must_use]
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    #[must_use]
    pub fn random(seed: u64) -> Self {
        let mut rng = Rng::with_seed(seed);
        Self {
            r: rng.u8(..),
            g: rng.u8(..),
            b: rng.u8(..),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct PlayerInfo {
    pub id:     PlayerId,
    pub name:   PlayerName,
    pub colour: Colour,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum GameResponse {
    MapLevelProvided(MapLevel),
    PlayersUpdated(HashMap<PlayerId, PlayerInfo>),
    BuildingsBuilt(Vec<BuildingInfo>),
    TransportsExist(Vec<TransportInfo>),
    GameJoined,
    GameLeft,

    CannotBuild(Vec<BuildingId>),
    CannotPurchase(TransportId),
}

impl Debug for GameResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameResponse::MapLevelProvided(_map_level) => {
                write!(f, "MapLevelProvided")
            },
            GameResponse::PlayersUpdated(players) => {
                write!(f, "PlayersUpdated({} players)", players.len())
            },
            GameResponse::BuildingsBuilt(buildings) => {
                write!(f, "BuildingsBuilt({} buildings)", buildings.len())
            },
            GameResponse::TransportsExist(transports) => {
                write!(f, "TransportsExist({} transports)", transports.len())
            },
            GameResponse::GameJoined => write!(f, "GameJoined"),
            GameResponse::GameLeft => write!(f, "GameLeft"),
            GameResponse::CannotBuild(building_ids) => {
                write!(f, "CannotBuild({} buildings)", building_ids.len())
            },
            GameResponse::CannotPurchase(transport_id) => {
                write!(f, "CannotPurchase({:?})", transport_id)
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ServerError {
    JoinFailedAlreadyInGame,
    LeaveFailedNotInGame,
    GameNotFound,
    NotAuthorized,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum NetworkResponse {
    Pong { id: Uuid, elapsed: Duration },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ServerResponse {
    Network(NetworkResponse),
    Authentication(AuthenticationResponse),
    Lobby(LobbyResponse),
    Game(GameId, GameResponse),
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
