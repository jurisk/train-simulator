#![expect(clippy::unwrap_used)]

use std::collections::HashMap;
use std::fs;

use game_ai::ArtificialIntelligenceState;
use game_ai::oct2025::Oct2025ArtificialIntelligenceState;
use game_logic::game_service::{GameResponseWithAddress, GameService};
use game_logic::games_service::GamesService;
use log::{error, info};
use shared_domain::building::industry_type::IndustryType;
use shared_domain::building::military_building_type::MilitaryBuildingType;
use shared_domain::cargo_amount::CargoAmount;
use shared_domain::cargo_map::{CargoMap, WithCargo};
use shared_domain::client_command::GameCommand;
use shared_domain::game_state::{GameState, GameStateFlattened};
use shared_domain::game_time::GameTimeDiff;
use shared_domain::metrics::NoopMetrics;
use shared_domain::resource_type::ResourceType;
use shared_domain::server_response::{AddressEnvelope, GameResponse, ServerResponse, UserInfo};
use shared_domain::{GameId, PlayerId, ScenarioId, UserId, UserName};
use shared_util::compression::save_to_bytes;

fn create_and_join(games_service: &mut GamesService, user_id: UserId) -> (GameId, PlayerId) {
    let user_info = UserInfo {
        id:   user_id,
        name: UserName::new(format!("AI {user_id}")),
    };

    let create_and_join_response = games_service
        .create_and_join_game_by_scenario(&user_info, ScenarioId::all().first().unwrap(), None)
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

    let join_response = games_service.join_game(&user_info, game_id, None).unwrap();

    let response = join_response.first().unwrap();
    let ServerResponse::Game(_game_id, GameResponse::GameJoined(player_id, _)) = response.response
    else {
        panic!("Expected response, got {response:?}",);
    };

    player_id
}

fn cargo_in_buildings(
    game_state: &GameState,
    player_id: PlayerId,
    industry_type: IndustryType,
) -> CargoMap {
    let mut cargo = CargoMap::new();
    for building in game_state
        .building_state()
        .find_industry_buildings_by_owner_and_type(player_id, industry_type)
    {
        cargo += building.cargo();
    }
    cargo
}

fn cargo_in_stations(game_state: &GameState, player_id: PlayerId) -> CargoMap {
    let mut cargo = CargoMap::new();
    for station in game_state
        .building_state()
        .find_stations_by_owner(player_id)
    {
        cargo += station.cargo();
    }
    cargo
}

fn cargo_exceeds_threshold(
    cargo: &CargoMap,
    check_resources: &[ResourceType],
    threshold: CargoAmount,
) -> bool {
    check_resources
        .iter()
        .all(|resource| cargo.get(*resource) > threshold)
}

fn end_condition(game_state: &GameState, player_id: PlayerId) -> bool {
    player_has_enough_cargo(game_state, player_id)
        && player_has_fixed_artillery(game_state, player_id)
        && player_has_projectiles(game_state, player_id)
}

fn player_has_projectiles(game_state: &GameState, player_id: PlayerId) -> bool {
    game_state
        .projectile_state()
        .find_projectiles_by_owner(player_id)
        .into_iter()
        .next()
        .is_some()
}

fn player_has_fixed_artillery(game_state: &GameState, player_id: PlayerId) -> bool {
    let arty: Vec<_> = game_state
        .building_state()
        .find_military_buildings_by_owner_and_type(player_id, MilitaryBuildingType::FixedArtillery)
        .into_iter()
        .collect();
    !arty.is_empty()
}

fn player_has_enough_cargo(game_state: &GameState, player_id: PlayerId) -> bool {
    let cargo = cargo_in_buildings(game_state, player_id, IndustryType::MilitaryBase);
    cargo_exceeds_threshold(
        &cargo,
        &IndustryType::MilitaryBase.input_resource_types(),
        CargoAmount::ZERO,
    )
}

fn run_ai_commands(
    game_service: &mut GameService,
    artificial_intelligence_states: &mut HashMap<PlayerId, Box<dyn ArtificialIntelligenceState>>,
    player_id: PlayerId,
) {
    let artificial_intelligence_state = artificial_intelligence_states.get_mut(&player_id).unwrap();
    let commands = artificial_intelligence_state
        .ai_commands(game_service.game_state(), &NoopMetrics::default());
    if let Some(commands) = commands {
        for command in commands {
            let responses = game_service.process_command(player_id, &command);
            match responses {
                Ok(responses) => {
                    apply_game_responses(artificial_intelligence_states, &Some(command), responses);
                },
                Err(err) => {
                    panic!("Failed to process command: {command:?}: {err:?}",);
                },
            }
        }
    }
}

fn apply_game_responses(
    artificial_intelligence_states: &mut HashMap<PlayerId, Box<dyn ArtificialIntelligenceState>>,
    command: &Option<GameCommand>,
    responses: Vec<GameResponseWithAddress>,
) {
    for response in responses {
        if let GameResponse::Error(error) = &response.response {
            error!("Failed to process command: {command:?}: {error:?}");
        }

        match &response.address {
            AddressEnvelope::ToClient(_) | AddressEnvelope::ToUser(_) => {
                error!("Unexpected response: {response:?}");
            },
            AddressEnvelope::ToPlayer(_, player_id) => {
                artificial_intelligence_states
                    .get_mut(player_id)
                    .unwrap()
                    .as_mut()
                    .notify_of_response(&response.response);
            },
            AddressEnvelope::ToAllPlayersInGame(_) => {
                for ai_state in artificial_intelligence_states.values_mut() {
                    ai_state.notify_of_response(&response.response);
                }
            },
        }
    }
}

#[test_log::test]
fn ai_test_oct2025() {
    info!("Starting AI test Oct 2025...");
    ai_test(|player_id: PlayerId, game_state: &GameState| {
        Box::new(Oct2025ArtificialIntelligenceState::new(
            player_id, game_state,
        ))
    });
}

fn print_end_state(
    player_ais: &HashMap<PlayerId, Box<dyn ArtificialIntelligenceState>>,
    game_state: &GameState,
) {
    for player_id in player_ais.keys() {
        println!("Player {player_id}");
        println!();

        println!("AI state:");
        let ai_state = player_ais.get(player_id).unwrap().as_ref();
        println!("{ai_state:?}");
        println!();

        println!("Transports:");
        for transport in game_state
            .transport_state()
            .find_players_transports(*player_id)
        {
            println!("  {transport:?}");
        }
        println!();

        println!("Industries:");
        for industry_type in IndustryType::all() {
            if industry_type != IndustryType::MilitaryBase {
                let cargo = cargo_in_buildings(game_state, *player_id, industry_type);
                println!("  {industry_type:?}: {cargo:?}");
            }
        }

        println!("Cargo in stations:");
        let cargo = cargo_in_stations(game_state, *player_id);
        for resource in ResourceType::all() {
            println!("  {resource:?}: {:?}", cargo.get(resource));
        }
        println!();

        println!("Cargo in military bases:");
        let cargo = cargo_in_buildings(game_state, *player_id, IndustryType::MilitaryBase);
        for resource in IndustryType::MilitaryBase.input_resource_types() {
            println!("  {resource:?}: {:?}", cargo.get(resource));
        }
        println!();

        let buildings = game_state
            .building_state()
            .find_military_buildings_by_owner_and_type(
                *player_id,
                MilitaryBuildingType::FixedArtillery,
            )
            .into_iter()
            .collect::<Vec<_>>();
        println!("Fixed artillery: {buildings:?}");
        println!();
        println!();
    }
}

// TODO HIGH: The test is flaky as sometimes the supply chains fail: tracks did not get built (not sure why), or stations don't get built as there is no space for them due to tracks crowding industries...
const MAX_STEPS: usize = 10_000;

#[expect(clippy::similar_names)]
fn ai_test<F>(factory: F)
where
    F: Fn(PlayerId, &GameState) -> Box<dyn ArtificialIntelligenceState>,
{
    let mut games_service = GamesService::new(false);

    let (game_id, player_id_1) = create_and_join(&mut games_service, UserId::random());
    let player_id_2 = join_game(&mut games_service, game_id, UserId::random());

    let game_service = games_service.get_game_service_mut(game_id).unwrap();

    let game_state = game_service.game_state();
    let mut player_ais: HashMap<_, _> = vec![player_id_1, player_id_2]
        .into_iter()
        .map(|player_id| (player_id, factory(player_id, game_state)))
        .collect();

    let mut steps = 0;

    while steps < MAX_STEPS {
        let game_state = game_service.game_state();

        // TODO HIGH: Run until military action happens instead of just until enough cargo + has military buildings
        if player_ais
            .keys()
            .all(|player_id| end_condition(game_state, *player_id))
        {
            print_end_state(&player_ais, game_service.game_state());
            println!("AI finished in {steps} steps");
            return;
        }

        let player_ids: Vec<_> = player_ais.keys().copied().collect();
        for player_id in player_ids {
            run_ai_commands(game_service, &mut player_ais, player_id);
        }

        let diff = GameTimeDiff::from_seconds(0.1);
        let responses = game_service.advance_time_diff(diff, &NoopMetrics::default());
        apply_game_responses(&mut player_ais, &None, responses);

        steps += 1;
    }

    let final_game_state = game_service.game_state();
    print_end_state(&player_ais, final_game_state);

    let flattened: GameStateFlattened = final_game_state.clone().into();
    let serialized = save_to_bytes(&flattened).unwrap();

    let output_path = "../../ai_until_final_goods_built.game_state.bincode.gz";
    fs::write(output_path, serialized).unwrap();

    panic!("AI did not finish in {MAX_STEPS} steps, game state dumped to {output_path}");

    // TODO: We should also dump - and later read - AI state for better debugging
}
