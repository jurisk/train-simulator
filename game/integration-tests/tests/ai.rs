#![expect(clippy::unwrap_used)]

use game_ai::{ArtificialIntelligenceState, ai_commands};
use game_logic::games_service::GamesService;
use shared_domain::cargo_amount::CargoAmount;
use shared_domain::cargo_map::{CargoMap, WithCargo};
use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::game_time::{GameTime, GameTimeDiff};
use shared_domain::metrics::NoopMetrics;
use shared_domain::resource_type::ResourceType;
use shared_domain::server_response::{GameResponse, ServerResponse, UserInfo};
use shared_domain::{GameId, PlayerId, ScenarioId, UserId, UserName};

fn create_and_join(games_service: &mut GamesService, user_id: UserId) -> (GameId, PlayerId) {
    let user_info = UserInfo {
        id:   user_id,
        name: UserName::new(format!("AI {user_id}")),
    };

    let create_and_join_response = games_service
        .create_and_join_game(&user_info, ScenarioId::all().first().unwrap())
        .unwrap();

    let response = create_and_join_response.first().unwrap();
    let ServerResponse::Game(game_id, GameResponse::GameJoined(player_id, _)) = response.response
    else {
        panic!("Expected response, got {response:?}",);
    };

    (game_id, player_id)
}

fn get_snapshot(games_service: &mut GamesService, game_id: GameId, user_id: UserId) -> GameState {
    let mut response = games_service
        .process_command(game_id, user_id, &GameCommand::RequestGameStateSnapshot)
        .unwrap();
    assert_eq!(response.len(), 1);
    let game_state = response.remove(0);
    let ServerResponse::Game(_game_id, GameResponse::GameStateSnapshot(game_state)) =
        game_state.response
    else {
        panic!("Expected GameStateSnapshot, got {:?}", game_state.response);
    };

    game_state
}

fn cargo_in_stations(game_state: &GameState, player_id: PlayerId) -> CargoMap {
    let mut cargo = CargoMap::new();
    for station in game_state.building_state().find_players_stations(player_id) {
        cargo += station.cargo();
    }
    cargo
}

fn enough_cargo(cargo: &CargoMap) -> bool {
    [
        ResourceType::Ammunition,
        ResourceType::Weapons,
        ResourceType::Food,
        // We are now skipping Concrete as we are granting it in the initial ConstructionYard
    ]
    .iter()
    .any(|resource| cargo.get(*resource) > CargoAmount::ZERO)
}

fn player_has_enough_cargo(game_state: &GameState, player_id: PlayerId) -> bool {
    enough_cargo(&cargo_in_stations(game_state, player_id))
}

fn run_ai_commands(
    games_service: &mut GamesService,
    player_id: PlayerId,
    game_state: &GameState,
    artificial_intelligence_state: &mut ArtificialIntelligenceState,
    game_id: GameId,
    user_id: UserId,
) {
    let commands = ai_commands(
        player_id,
        game_state,
        artificial_intelligence_state,
        &NoopMetrics::default(),
    );
    if let Some(commands) = commands {
        for command in commands {
            let responses = games_service.process_command(game_id, user_id, &command);
            assert!(responses.is_ok());
        }
    }
}

#[test]
fn ai_until_final_goods_built() {
    let user_id_1 = UserId::random();
    let mut artificial_intelligence_state = ArtificialIntelligenceState::default();

    let mut games_service = GamesService::new();

    let (game_id, player_id_1) = create_and_join(&mut games_service, user_id_1);

    let mut time = GameTime::new();

    loop {
        let game_state = get_snapshot(&mut games_service, game_id, user_id_1);

        if player_has_enough_cargo(&game_state, player_id_1) {
            break;
        }

        run_ai_commands(
            &mut games_service,
            player_id_1,
            &game_state,
            &mut artificial_intelligence_state,
            game_id,
            user_id_1,
        );

        time = time + GameTimeDiff::from_seconds(0.1);
        games_service.advance_times(time, &NoopMetrics::default());
    }
}
