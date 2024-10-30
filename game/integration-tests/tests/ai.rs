#![expect(clippy::unwrap_used)]

use game_ai::ArtificialIntelligenceState;
use game_ai::oct2025::Oct2025ArtificialIntelligenceState;
use game_ai::sep2025::Sep2025ArtificialIntelligenceState;
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
        ResourceType::ArtilleryWeapons,
        ResourceType::Food,
        ResourceType::Fuel,
        // We are skipping, e.g., Concrete as we are granting it in the initial ConstructionYard
    ]
    .iter()
    // TODO: Switch to `all` eventually
    .any(|resource| cargo.get(*resource) > CargoAmount::ZERO)
}

fn player_has_enough_cargo(game_state: &GameState, player_id: PlayerId) -> bool {
    enough_cargo(&cargo_in_stations(game_state, player_id))
}

#[expect(clippy::match_same_arms)]
fn run_ai_commands(
    games_service: &mut GamesService,
    game_state: &GameState,
    artificial_intelligence_state: &mut dyn ArtificialIntelligenceState,
    game_id: GameId,
    user_id: UserId,
) {
    let commands = artificial_intelligence_state.ai_commands(game_state, &NoopMetrics::default());
    if let Some(commands) = commands {
        for command in commands {
            let responses = games_service.process_command(game_id, user_id, &command);
            match responses {
                Ok(responses) => {
                    for response in responses {
                        match response.response {
                            ServerResponse::Network(_) => {},
                            ServerResponse::Authentication(_) => {},
                            ServerResponse::Lobby(_) => {},
                            ServerResponse::Game(_, GameResponse::Error(error)) => {
                                panic!("Failed to process command: {command:?}: {error:?}");
                            },
                            ServerResponse::Game(..) => {},
                            ServerResponse::Error(error) => {
                                panic!("Failed to process command: {command:?}: {error:?}");
                            },
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
fn ai_until_final_goods_built_sep2025() {
    ai_until_final_goods_built(|player_id: PlayerId, game_state: &GameState| {
        Box::new(Sep2025ArtificialIntelligenceState::new(
            player_id, game_state,
        ))
    });
}

#[test]
fn ai_until_final_goods_built_oct2025() {
    ai_until_final_goods_built(|player_id: PlayerId, game_state: &GameState| {
        Box::new(Oct2025ArtificialIntelligenceState::new(
            player_id, game_state,
        ))
    });
}

const MAX_STEPS: usize = 10_000;
fn ai_until_final_goods_built<F>(factory: F)
where
    F: Fn(PlayerId, &GameState) -> Box<dyn ArtificialIntelligenceState>,
{
    let mut games_service = GamesService::new(false);

    let user_id_1 = UserId::random();
    let (game_id, player_id_1) = create_and_join(&mut games_service, user_id_1);

    let user_id_2 = UserId::random();
    let player_id_2 = join_game(&mut games_service, game_id, user_id_2);

    let mut time = GameTime::new();

    let game_state_1 = get_snapshot(&mut games_service, game_id, user_id_1);
    let mut artificial_intelligence_state_1 = factory(player_id_1, &game_state_1);

    let game_state_2 = get_snapshot(&mut games_service, game_id, user_id_2);
    let mut artificial_intelligence_state_2 = factory(player_id_2, &game_state_2);

    let mut steps = 0;

    while steps < MAX_STEPS {
        let game_state = get_snapshot(&mut games_service, game_id, user_id_1);

        // Later: Optimise so you can do `&&` instead of `||` here
        // TODO HIGH: Run until military action happens instead of just until enough cargo
        if player_has_enough_cargo(&game_state, player_id_1)
            || player_has_enough_cargo(&game_state, player_id_2)
        {
            println!("AI finished in {steps} steps");
            return;
        }

        run_ai_commands(
            &mut games_service,
            &game_state,
            artificial_intelligence_state_1.as_mut(),
            game_id,
            user_id_1,
        );

        // Refreshing game state to avoid clashing commands
        let game_state = get_snapshot(&mut games_service, game_id, user_id_2);

        run_ai_commands(
            &mut games_service,
            &game_state,
            artificial_intelligence_state_2.as_mut(),
            game_id,
            user_id_2,
        );

        time = time + GameTimeDiff::from_seconds(0.1);
        games_service.advance_times(time, &NoopMetrics::default());

        steps += 1;
    }

    let end_game_state = get_snapshot(&mut games_service, game_id, user_id_1);
    let cargo_1 = cargo_in_stations(&end_game_state, player_id_1);
    let cargo_2 = cargo_in_stations(&end_game_state, player_id_2);

    panic!("AI did not finish in {MAX_STEPS} steps, cargo_1 = {cargo_1:?}, cargo_2 = {cargo_2:?}");
}
