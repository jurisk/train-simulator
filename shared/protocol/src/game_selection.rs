use shared_domain::game_state::GameState;

pub enum ClientMessage {
    JoinGame,
}

pub enum ServerMessage {
    GameJoined { game_state: GameState },
}
