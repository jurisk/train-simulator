use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

use fastrand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::building::building_info::{BuildingDynamicInfo, BuildingInfo};
use crate::building::track_info::TrackInfo;
use crate::game_state::GameState;
use crate::game_time::GameTime;
use crate::transport::transport_info::{TransportDynamicInfo, TransportInfo};
use crate::{BuildingId, ClientId, GameId, PlayerId, PlayerName, TrackId, TransportId};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum AuthenticationResponse {
    LoginSucceeded(PlayerId),
    LogoutSucceeded,

    Error(AuthenticationError),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum AuthenticationError {
    LoginFailed,
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
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
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
    GameStateSnapshot(GameState),

    // Later: Actually, many of these should be sending `GameTime` (if it's not already included in other structures such as `GameState`), and it should be handled on the client.
    PlayersUpdated(HashMap<PlayerId, PlayerInfo>),
    BuildingsAdded(Vec<BuildingInfo>),
    TracksAdded(Vec<TrackInfo>),
    TransportsAdded(Vec<TransportInfo>),
    DynamicInfosSync(
        GameTime,
        HashMap<BuildingId, BuildingDynamicInfo>,
        HashMap<TransportId, TransportDynamicInfo>,
    ),
    GameJoined,
    GameLeft,

    Error(GameError),
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum GameError {
    GameNotFound,
    CannotBuildBuildings(Vec<BuildingId>),
    CannotBuildTracks(Vec<TrackId>),
    CannotPurchase(TransportId),
    UnspecifiedError,
}

impl Debug for GameResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameResponse::GameStateSnapshot(game_state) => {
                write!(f, "GameStateSnapshot({:?} time)", game_state.time())
            },
            GameResponse::PlayersUpdated(players) => {
                write!(f, "PlayersUpdated({} players)", players.len())
            },
            GameResponse::BuildingsAdded(buildings) => {
                write!(f, "BuildingsAdded({} buildings)", buildings.len())
            },
            GameResponse::TracksAdded(tracks) => {
                write!(f, "TracksAdded({} tracks)", tracks.len())
            },
            GameResponse::TransportsAdded(transports) => {
                write!(
                    f,
                    "TransportsAdded({})",
                    transports
                        .iter()
                        .map(|t| format!("{:?}", t.transport_id()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
            GameResponse::DynamicInfosSync(game_time, buildings, transports) => {
                write!(
                    f,
                    "DynamicInfosSync({:?} time, {} buildings, {} transports)",
                    game_time,
                    buildings.len(),
                    transports.len()
                )
            },
            GameResponse::GameJoined => write!(f, "GameJoined"),
            GameResponse::GameLeft => write!(f, "GameLeft"),
            GameResponse::Error(error) => {
                write!(f, "Error({error:?})")
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
#[allow(clippy::large_enum_variant)]
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
