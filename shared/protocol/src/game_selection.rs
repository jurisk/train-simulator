use shared_domain::level::Level;

pub enum ClientMessage {
    JoinGame,
}

pub enum ServerMessage {
    GameJoined { level: Level },
}
