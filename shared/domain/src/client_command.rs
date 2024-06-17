#![allow(clippy::module_name_repetitions)]

use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{BuildingInfo, ClientId, GameId, PlayerId, PlayerName, VehicleInfo};

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
    CreateGame(PlayerName),
    JoinExistingGame(GameId, PlayerName),
    LeaveGame(GameId),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum GameCommand {
    QueryBuildings,
    BuildBuildings(Vec<BuildingInfo>),
    PurchaseVehicle(VehicleInfo), // TODO: Rename, as we can purchase a whole train at once!
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
