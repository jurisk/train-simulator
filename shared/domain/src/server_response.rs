use std::collections::HashMap;
use std::fmt::Debug;

use fastrand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web_time::Duration;

use crate::building::BuildError;
use crate::building::building_info::BuildingDynamicInfo;
use crate::building::industry_building_info::IndustryBuildingInfo;
use crate::building::military_building_info::MilitaryBuildingInfo;
use crate::building::station_info::StationInfo;
use crate::building::track_info::TrackInfo;
use crate::client_command::DemolishSelector;
use crate::game_state::GameState;
use crate::game_time::GameTime;
use crate::transport::transport_info::{TransportDynamicInfo, TransportInfo};
use crate::{
    ClientId, GameId, IndustryBuildingId, MilitaryBuildingId, PlayerId, PlayerName, ScenarioId,
    StationId, TrackId, TransportId, UserId, UserName,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum AuthenticationResponse {
    LoginSucceeded(UserId),
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
    pub scenario_id:  ScenarioId,
    pub game_id:      GameId,
    pub players:      Vec<PlayerInfo>,
    pub user_players: Vec<(UserId, PlayerId)>,
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct UserInfo {
    pub id:   UserId,
    pub name: UserName,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum GameResponse {
    // Later: Remove as this is for testing purposes only
    GameStateSnapshot(GameState),

    // Later: Actually, many of these should be sending `GameTime` (if it's not already included in other structures such as `GameState`), and it should be handled on the client.
    PlayersUpdated(Vec<(UserId, PlayerId)>),
    IndustryBuildingAdded(IndustryBuildingInfo),
    IndustryBuildingRemoved(IndustryBuildingId),
    MilitaryBuildingAdded(MilitaryBuildingInfo),
    MilitaryBuildingRemoved(MilitaryBuildingId),
    StationAdded(StationInfo),
    StationRemoved(StationId),
    TracksAdded(Vec<TrackInfo>),
    TracksRemoved(Vec<TrackId>),
    TransportsAdded(Vec<TransportInfo>),
    DynamicInfosSync(
        GameTime,
        HashMap<IndustryBuildingId, BuildingDynamicInfo>,
        HashMap<StationId, BuildingDynamicInfo>,
        HashMap<TransportId, TransportDynamicInfo>,
    ),
    GameJoined(PlayerId, GameState),
    GameLeft,

    Error(GameError),
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum GameError {
    GameNotFound,
    CannotBuildStation(StationId, BuildError),
    CannotBuildIndustryBuilding(IndustryBuildingId, BuildError),
    CannotBuildMilitaryBuilding(MilitaryBuildingId, BuildError),
    CannotBuildTracks(Vec<TrackId>, BuildError),
    CannotPurchase(TransportId, BuildError),
    CannotDemolish(DemolishSelector),
    UnspecifiedError,
}

impl Debug for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::GameNotFound => write!(f, "GameNotFound"),
            GameError::CannotBuildStation(station_id, error) => {
                write!(f, "CannotBuildStation({station_id:?}: {error:?})")
            },
            GameError::CannotBuildIndustryBuilding(industry_building_id, error) => {
                write!(
                    f,
                    "CannotBuildIndustryBuilding({industry_building_id:?}: {error:?})"
                )
            },
            GameError::CannotBuildMilitaryBuilding(military_building_id, error) => {
                write!(
                    f,
                    "CannotBuildMilitaryBuilding({military_building_id:?}: {error:?})"
                )
            },
            GameError::CannotBuildTracks(track_ids, error) => {
                write!(
                    f,
                    "CannotBuildTracks({} tracks: {error:?})",
                    track_ids.len()
                )
            },
            GameError::CannotPurchase(transport_id, error) => {
                write!(f, "CannotPurchase({transport_id:?}, {error:?})")
            },
            GameError::CannotDemolish(demolish_selector) => {
                write!(f, "CannotDemolish({demolish_selector:?})")
            },
            GameError::UnspecifiedError => write!(f, "UnspecifiedError"),
        }
    }
}

impl Debug for GameResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameResponse::GameStateSnapshot(game_state) => {
                write!(
                    f,
                    "GameStateSnapshot({:?}, {:?}, {:?})",
                    game_state.time(),
                    game_state.building_state(),
                    game_state.transport_state(),
                )
            },
            GameResponse::PlayersUpdated(players) => {
                write!(f, "PlayersUpdated({} players)", players.len())
            },
            GameResponse::IndustryBuildingAdded(building) => {
                write!(f, "IndustryBuildingAdded({})", building.id())
            },
            GameResponse::IndustryBuildingRemoved(industry_building_id) => {
                write!(f, "IndustryBuildingRemoved({industry_building_id:?})")
            },
            GameResponse::StationAdded(station) => {
                write!(f, "StationAdded({})", station.id())
            },
            GameResponse::MilitaryBuildingAdded(building) => {
                write!(f, "MilitaryBuildingAdded({})", building.id())
            },
            GameResponse::MilitaryBuildingRemoved(building_id) => {
                write!(f, "MilitaryBuildingRemoved({building_id})")
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
            GameResponse::DynamicInfosSync(game_time, buildings, stations, transports) => {
                write!(
                    f,
                    "DynamicInfosSync({:?} time, {} industry, {} stations, {} transports)",
                    game_time,
                    buildings.len(),
                    stations.len(),
                    transports.len()
                )
            },
            GameResponse::GameJoined(player_id, _game_state) => {
                write!(f, "GameJoined({player_id:?})")
            },
            GameResponse::GameLeft => write!(f, "GameLeft"),
            GameResponse::Error(error) => {
                write!(f, "Error({error:?})")
            },
            GameResponse::StationRemoved(station_id) => {
                write!(f, "StationRemoved({station_id:?})")
            },
            GameResponse::TracksRemoved(track_ids) => {
                write!(f, "TracksRemoved({:?} tracks)", track_ids.len())
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
    ScenarioNotFound(ScenarioId),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum NetworkResponse {
    Pong { id: Uuid, elapsed: Duration },
}

// Later: We are shipping too much in `GameState`, it has too much denormalisation.
#[expect(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    ToUser(UserId),
    ToPlayer(GameId, PlayerId),
    ToAllPlayersInGame(GameId),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[expect(clippy::module_name_repetitions)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[expect(clippy::module_name_repetitions)]
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
