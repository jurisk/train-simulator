use game_ai::{ai_commands, ArtificialIntelligenceState};
use game_logic::games_service::GamesService;
use shared_domain::cargo_amount::CargoAmount;
use shared_domain::cargo_map::{CargoMap, WithCargo};
use shared_domain::client_command::GameCommand;
use shared_domain::game_time::{GameTime, GameTimeDiff};
use shared_domain::resource_type::ResourceType;
use shared_domain::server_response::{Colour, GameResponse, PlayerInfo, ServerResponse};
use shared_domain::{MapId, PlayerId, PlayerName};

#[test]
fn ai_until_final_goods_built() {
    let player_id = PlayerId::random();
    let player_info = PlayerInfo {
        id:     player_id,
        name:   PlayerName::new("Test AI".to_string()),
        colour: Colour::random(0),
    };
    let mut artificial_intelligence_state = ArtificialIntelligenceState::default();

    let mut games_service = GamesService::new();
    let response = games_service
        .create_and_join_game(&player_info, MapId::all().first().unwrap())
        .unwrap();

    let game_state_snapshot = response.first().unwrap();
    let game_id = if let ServerResponse::Game(game_id, _) = &game_state_snapshot.response {
        *game_id
    } else {
        panic!(
            "Expected Game response, got {:?}",
            game_state_snapshot.response
        );
    };

    let mut time = GameTime::new();

    loop {
        let response = games_service
            .process_command(game_id, player_id, &GameCommand::RequestGameStateSnapshot)
            .unwrap();
        let game_state = response.first().unwrap();
        let game_state =
            if let ServerResponse::Game(_game_id, GameResponse::GameStateSnapshot(snapshot)) =
                &game_state.response
            {
                snapshot
            } else {
                panic!("Expected GameStateSnapshot, got {:?}", game_state.response);
            };

        let mut cargo = CargoMap::new();
        for station in game_state.building_state().find_players_stations(player_id) {
            cargo += station.cargo();
        }

        if [
            ResourceType::Ammunition,
            ResourceType::Weapons,
            ResourceType::Food,
            ResourceType::Concrete,
        ]
        .iter()
        .all(|resource| cargo.get(*resource) > CargoAmount::ZERO)
        {
            break;
        }

        let commands = ai_commands(player_id, game_state, &mut artificial_intelligence_state);
        if let Some(commands) = commands {
            for command in commands {
                let _responses = games_service
                    .process_command(game_id, player_id, &command)
                    .unwrap();
            }
        }

        time = time + GameTimeDiff::from_seconds(0.1);
        games_service.advance_times(time);
    }
}
