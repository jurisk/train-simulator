#![allow(clippy::module_name_repetitions)]

use std::fmt::{Debug, Formatter};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::building::building_info::{IndustryBuildingInfo, StationInfo};
use crate::building::track_info::TrackInfo;
use crate::transport::movement_orders::MovementOrders;
use crate::transport::transport_info::TransportInfo;
use crate::{ClientId, GameId, PlayerId, TransportId};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AccessToken(String);

impl AccessToken {
    #[must_use]
    pub fn new(token: String) -> Self {
        Self(token)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum AuthenticationCommand {
    Login(PlayerId, AccessToken),
    Logout,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum LobbyCommand {
    ListGames,
    CreateGame,
    JoinExistingGame(GameId),
    LeaveGame(GameId),
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum GameCommand {
    // These queries are separate due to some race conditions on the client, where the map level
    // was not available yet, so received buildings / transports got ignored.
    QueryBuildings,
    QueryTracks,
    QueryTransports,
    BuildIndustryBuildings(Vec<IndustryBuildingInfo>),
    BuildStations(Vec<StationInfo>),
    BuildTracks(Vec<TrackInfo>),
    PurchaseTransport(TransportInfo),
    UpdateTransportMovementOrders(TransportId, MovementOrders),
}

impl Debug for GameCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GameCommand::QueryBuildings => write!(f, "QueryBuildings"),
            GameCommand::QueryTracks => write!(f, "QueryTracks"),
            GameCommand::QueryTransports => write!(f, "QueryTransports"),
            GameCommand::BuildIndustryBuildings(buildings) => {
                write!(f, "BuildIndustryBuildings({} buildings)", buildings.len())
            },
            GameCommand::BuildStations(stations) => {
                write!(f, "BuildStations({} stations)", stations.len())
            },
            GameCommand::BuildTracks(tracks) => {
                write!(f, "BuildTracks({} tracks)", tracks.len())
            },
            GameCommand::PurchaseTransport(transport) => {
                write!(f, "PurchaseTransport({transport:?})")
            },
            GameCommand::UpdateTransportMovementOrders(transport_id, _) => {
                write!(f, "UpdateTransportMovementOrders({transport_id:?})",)
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum NetworkCommand {
    Ping { id: Uuid, elapsed: Duration },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ClientCommand {
    Network(NetworkCommand),
    Authentication(AuthenticationCommand),
    Lobby(LobbyCommand),
    Game(GameId, GameCommand),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ClientCommandWithClientId {
    client_id: ClientId,
    command:   ClientCommand,
}

impl ClientCommandWithClientId {
    #[must_use]
    pub fn new(client_id: ClientId, command: ClientCommand) -> Self {
        Self { client_id, command }
    }

    #[must_use]
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    #[must_use]
    pub fn command(&self) -> &ClientCommand {
        &self.command
    }
}
