#![allow(clippy::module_name_repetitions)]

use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web_time::Duration;

use crate::building::industry_building_info::IndustryBuildingInfo;
use crate::building::station_info::StationInfo;
use crate::building::track_info::TrackInfo;
use crate::transport::movement_orders::MovementOrders;
use crate::transport::transport_info::TransportInfo;
use crate::{
    ClientId, GameId, IndustryBuildingId, ScenarioId, StationId, TrackId, TransportId, UserId,
};

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
    Login(UserId, AccessToken),
    Logout,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum LobbyCommand {
    ListGames,
    CreateGame(ScenarioId),
    JoinExistingGame(GameId),
    LeaveGame(GameId),
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum DemolishSelector {
    Tracks(Vec<TrackId>),
    Industry(IndustryBuildingId),
    Station(StationId),
}

impl Debug for DemolishSelector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DemolishSelector::Tracks(track_ids) => write!(f, "Track({track_ids:?})"),
            DemolishSelector::Industry(industry_id) => write!(f, "Industry({industry_id:?})"),
            DemolishSelector::Station(station_id) => write!(f, "Station({station_id:?})"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum GameCommand {
    BuildIndustryBuilding(IndustryBuildingInfo),
    BuildStation(StationInfo),
    BuildTracks(Vec<TrackInfo>),
    PurchaseTransport(TransportInfo),
    UpdateTransportMovementOrders(TransportId, MovementOrders),
    Demolish(DemolishSelector),

    // Later: This is only used for testing purposes, perhaps we can refactor to avoid this
    RequestGameStateSnapshot,
}

impl Debug for GameCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GameCommand::BuildIndustryBuilding(building) => {
                write!(f, "BuildIndustryBuilding({})", building.id())
            },
            GameCommand::BuildStation(station) => {
                write!(f, "BuildStation({})", station.id())
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
            GameCommand::Demolish(selector) => {
                write!(f, "Demolish({selector:?})")
            },
            GameCommand::RequestGameStateSnapshot => {
                write!(f, "RequestGameStateSnapshot")
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
