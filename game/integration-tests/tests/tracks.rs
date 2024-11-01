use shared_domain::PlayerId;
use shared_domain::directional_edge::DirectionalEdge;
use shared_domain::game_state::GameState;
use shared_domain::map_level::zoning::ZoningType;
use shared_domain::metrics::NoopMetrics;
use shared_domain::scenario::{Scenario, USA_SCENARIO_BINCODE};
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::transport::tile_track::TileTrack;
use shared_domain::transport::track_length::TrackLength;
use shared_domain::transport::track_pathfinding::find_route_to_tile_tracks;
use shared_domain::transport::track_planner::{DEFAULT_ALREADY_EXISTS_COEF, plan_tracks};
use shared_domain::transport::track_type::TrackType;
use shared_util::direction_xz::DirectionXZ;

#[test]
fn test_plan_tracks() {
    let player_id = PlayerId::random();

    let mut game_state = GameState::from_scenario(
        Scenario::load_from_bytes(USA_SCENARIO_BINCODE).unwrap(),
        false,
    );

    // We spawn construction yards in all free spots because this test is about testing track
    // planning, not availability of resources
    let industrials = game_state
        .all_free_zonings()
        .iter()
        .filter(|zoning| zoning.zoning_type() == ZoningType::Industrial)
        .map(|zoning| zoning.reference_tile())
        .collect::<Vec<_>>();
    for industrial_tile in industrials {
        game_state
            .building_state_mut()
            .gift_initial_construction_yard(player_id, industrial_tile);
    }

    let from_tile = TileCoordsXZ::new(340, 350);
    let to_tile = TileCoordsXZ::new(280, 150);

    let head = DirectionalEdge::new(from_tile, DirectionXZ::West);
    let tail = DirectionalEdge::new(to_tile, DirectionXZ::South);

    let (tracks, length) = plan_tracks(
        player_id,
        head,
        &[tail],
        &game_state,
        DEFAULT_ALREADY_EXISTS_COEF,
        &NoopMetrics::default(),
    )
    .expect("Failed to plan tracks");

    let expected_min_tracks = 250;
    assert!(
        tracks.len() > expected_min_tracks,
        "Expected at least {expected_min_tracks} tracks, got {}",
        tracks.len()
    );
    let expected_min_length = 200f32;
    assert!(
        length > TrackLength::new(expected_min_length),
        "Expected at least {expected_min_length} length, got {length:?}"
    );
    let result = game_state
        .build_tracks(player_id, &tracks)
        .expect("Failed to build tracks");
    assert_eq!(result.len(), tracks.len());

    let first_tile = head.into_tile;
    let last_tile = tail.into_tile + tail.from_direction;

    let from_tile_track = TileTrack {
        tile:        first_tile,
        track_type:  TrackType::NorthWest,
        pointing_in: DirectionXZ::North,
    };

    let to_tile_track = TileTrack {
        tile:        last_tile,
        track_type:  TrackType::NorthSouth,
        pointing_in: DirectionXZ::North,
    };
    let route = find_route_to_tile_tracks(
        from_tile_track,
        &[to_tile_track],
        game_state.building_state(),
        &NoopMetrics::default(),
    )
    .unwrap();
    assert_eq!(route.len(), tracks.len());
}
