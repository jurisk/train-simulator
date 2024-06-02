use serde::{Deserialize, Serialize};
use shared_domain::{GameId, PlayerId};
use shared_util::coords_xz::CoordsXZ;

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
    JoinGame(GameId),
    LeaveGame(GameId),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum TrackType {
    NorthSouth,
    EastWest,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum BuildingType {
    Track(TrackType),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct BuildCommand {
    pub coords_xz:     CoordsXZ,
    pub building_type: BuildingType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum GameCommand {
    Build(BuildCommand),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ClientCommand {
    Authentication(AuthenticationCommand),
    Lobby(LobbyCommand),
    Game(GameCommand),
}
