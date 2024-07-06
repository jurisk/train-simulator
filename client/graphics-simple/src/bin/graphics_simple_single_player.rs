use std::collections::HashMap;

use graphics_simple::render_map::render_map;
use macroquad::prelude::*;
use shared_domain::game_state::GameState;
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{Colour, PlayerInfo};
use shared_domain::{PlayerId, PlayerName};

fn window_conf() -> Conf {
    Conf {
        window_title: "Train Simulator".to_string(),
        window_width: 1920,
        window_height: 1080,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let level_json = include_str!("../../../../assets/map_levels/default.json");
    let map_level = serde_json::from_str::<MapLevel>(level_json)
        .unwrap_or_else(|err| panic!("Failed to deserialise {level_json}: {err}"));
    let player_info = PlayerInfo {
        id:     PlayerId::random(),
        name:   PlayerName::new("Test Player".to_string()),
        colour: Colour { r: 255, g: 0, b: 0 },
    };
    let game_state = GameState::new(
        map_level,
        vec![],
        vec![],
        HashMap::from([(player_info.id, player_info)]),
    );

    loop {
        clear_background(LIGHTGRAY);
        render_map(game_state.map_level());
        next_frame().await;
    }
}
