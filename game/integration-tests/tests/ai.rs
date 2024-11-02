#![expect(clippy::unwrap_used)]

use game_ai::ArtificialIntelligenceState;
use game_ai::oct2025::Oct2025ArtificialIntelligenceState;
use game_logic::game_service::GameService;
use game_logic::games_service::GamesService;
use shared_domain::cargo_amount::CargoAmount;
use shared_domain::cargo_map::{CargoMap, WithCargo};
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

fn join_game(games_service: &mut GamesService, game_id: GameId, user_id: UserId) -> PlayerId {
    let user_info = UserInfo {
        id:   user_id,
        name: UserName::new(format!("AI {user_id}")),
    };

    let join_response = games_service.join_game(&user_info, game_id).unwrap();

    let response = join_response.first().unwrap();
    let ServerResponse::Game(_game_id, GameResponse::GameJoined(player_id, _)) = response.response
    else {
        panic!("Expected response, got {response:?}",);
    };

    player_id
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
        ResourceType::ArtilleryWeapons,
        ResourceType::Food,
        ResourceType::Fuel,
        // We are skipping, e.g., Concrete as we are granting it in the initial ConstructionYard
    ]
    .iter()
    // TODO: `all` is flaky... use `any` if `all` fails
    .all(|resource| cargo.get(*resource) > CargoAmount::ZERO)
}

fn player_has_enough_cargo(game_state: &GameState, player_id: PlayerId) -> bool {
    enough_cargo(&cargo_in_stations(game_state, player_id))
}

fn run_ai_commands(
    game_service: &mut GameService,
    artificial_intelligence_state: &mut dyn ArtificialIntelligenceState,
    player_id: PlayerId,
) {
    let commands = artificial_intelligence_state
        .ai_commands(game_service.game_state(), &NoopMetrics::default());
    if let Some(commands) = commands {
        for command in commands {
            let responses = game_service.process_command(player_id, &command);
            match responses {
                Ok(responses) => {
                    for response in responses {
                        if let GameResponse::Error(error) = response.response {
                            panic!("Failed to process command: {command:?}: {error:?}");
                        }
                    }
                },
                Err(err) => {
                    panic!("Failed to process command: {command:?}: {err:?}",);
                },
            }
        }
    }
}

#[test]
fn ai_until_final_goods_built_oct2025() {
    ai_until_final_goods_built(|player_id: PlayerId, game_state: &GameState| {
        Box::new(Oct2025ArtificialIntelligenceState::new(
            player_id, game_state,
        ))
    });
}

const MAX_STEPS: usize = 40_000;

#[expect(clippy::similar_names)]
fn ai_until_final_goods_built<F>(factory: F)
where
    F: Fn(PlayerId, &GameState) -> Box<dyn ArtificialIntelligenceState>,
{
    let mut games_service = GamesService::new(false);

    let user_id_1 = UserId::random();
    let (game_id, player_id_1) = create_and_join(&mut games_service, user_id_1);

    let user_id_2 = UserId::random();
    let player_id_2 = join_game(&mut games_service, game_id, user_id_2);

    let game_service = games_service.get_game_service_mut(game_id).unwrap();

    let mut time = GameTime::new();

    let mut artificial_intelligence_state_1 = factory(player_id_1, game_service.game_state());

    let mut artificial_intelligence_state_2 = factory(player_id_2, game_service.game_state());

    let mut steps = 0;

    while steps < MAX_STEPS {
        let game_state = game_service.game_state();

        // TODO HIGH: Run until military action happens instead of just until enough cargo
        if player_has_enough_cargo(game_state, player_id_1)
            || player_has_enough_cargo(game_state, player_id_2)
        {
            println!("AI finished in {steps} steps");
            return;
        }

        run_ai_commands(
            game_service,
            artificial_intelligence_state_1.as_mut(),
            player_id_1,
        );

        run_ai_commands(
            game_service,
            artificial_intelligence_state_2.as_mut(),
            player_id_2,
        );

        time = time + GameTimeDiff::from_seconds(0.1);
        game_service.advance_time(time, &NoopMetrics::default());

        steps += 1;
    }

    let end_game_state = game_service.game_state();
    let cargo_1 = cargo_in_stations(end_game_state, player_id_1);
    let cargo_2 = cargo_in_stations(end_game_state, player_id_2);

    panic!("AI did not finish in {MAX_STEPS} steps, cargo_1 = {cargo_1:?}, cargo_2 = {cargo_2:?}");
}
