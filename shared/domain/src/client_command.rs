#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};

use crate::{BuildingInfo, ClientId, GameId, PlayerId, PlayerName};

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
    BuildBuilding(BuildingInfo),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ClientCommand {
    Authentication(AuthenticationCommand),
    Lobby(LobbyCommand),
    Game(GameCommand),
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
