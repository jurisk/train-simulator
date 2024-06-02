use serde::{Deserialize, Serialize};
use shared_domain::{BuildingInfo, GameId, PlayerId};

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
