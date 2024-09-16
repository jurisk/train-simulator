use game_ai::{ai_commands, ArtificialIntelligenceState};
use game_logic::games_service::GamesService;
use shared_domain::cargo_amount::CargoAmount;
use shared_domain::cargo_map::{CargoMap, WithCargo};
use shared_domain::client_command::GameCommand;
use shared_domain::game_time::{GameTime, GameTimeDiff};
use shared_domain::metrics::NoopMetrics;
use shared_domain::resource_type::ResourceType;
use shared_domain::server_response::{GameResponse, ServerResponse, UserInfo};
use shared_domain::{MapId, UserId, UserName};

#[test]
fn ai_until_final_goods_built() {
    let user_id = UserId::random();
    let user_info = UserInfo {
        id:   user_id,
        name: UserName::new("Test AI".to_string()),
    };
    let mut artificial_intelligence_state = ArtificialIntelligenceState::default();

    let mut games_service = GamesService::new();
    let response = games_service
        .create_and_join_game(&user_info, MapId::all().first().unwrap())
        .unwrap();

    let response = response.first().unwrap();

    let ServerResponse::Game(game_id, GameResponse::GameJoined(player_id, _)) = response.response
    else {
        panic!("Expected response, got {response:?}",);
    };

    let mut time = GameTime::new();

    loop {
        let response = games_service
            .process_command(game_id, user_id, &GameCommand::RequestGameStateSnapshot)
            .unwrap();
        let game_state = response.first().unwrap();
        let ServerResponse::Game(_game_id, GameResponse::GameStateSnapshot(game_state)) =
            &game_state.response
        else {
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
        .any(|resource| cargo.get(*resource) > CargoAmount::ZERO)
        {
            break;
        }

        let commands = ai_commands(
            player_id,
            game_state,
            &mut artificial_intelligence_state,
            &NoopMetrics::default(),
        );
        if let Some(commands) = commands {
            for command in commands {
                let _responses = games_service
                    .process_command(game_id, user_id, &command)
                    .unwrap();
            }
        }

        time = time + GameTimeDiff::from_seconds(0.1);
        games_service.advance_times(time, &NoopMetrics::default());
    }
}
