#![allow(clippy::module_name_repetitions)]

use std::fmt::{Debug, Formatter};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{BuildingInfo, ClientId, GameId, PlayerId, TransportInfo};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AccessToken(pub String);

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
    QueryTransports,
    BuildBuildings(Vec<BuildingInfo>),
    PurchaseTransport(TransportInfo),
}

impl Debug for GameCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GameCommand::QueryBuildings => write!(f, "QueryBuildings"),
            GameCommand::QueryTransports => write!(f, "QueryTransports"),
            GameCommand::BuildBuildings(buildings) => {
                write!(f, "BuildBuildings({} buildings)", buildings.len())
            },
            GameCommand::PurchaseTransport(transport) => {
                write!(f, "PurchaseTransport({transport:?})")
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
    pub client_id: ClientId,
    pub command:   ClientCommand,
}

impl ClientCommandWithClientId {
    #[must_use]
    pub fn new(client_id: ClientId, command: ClientCommand) -> Self {
        Self { client_id, command }
    }
}
