use shared_domain::directional_edge::DirectionalEdge;
use shared_domain::game_state::GameState;
use shared_domain::map_level::map_level::MapLevel;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::transport::tile_track::TileTrack;
use shared_domain::transport::track_length::TrackLength;
use shared_domain::transport::track_pathfinding::find_route_to_tile_tracks;
use shared_domain::transport::track_planner::{plan_tracks, DEFAULT_ALREADY_EXISTS_COEF};
use shared_domain::transport::track_type::TrackType;
use shared_domain::{MapId, PlayerId};
use shared_util::direction_xz::DirectionXZ;

#[test]
fn test_plan_tracks() {
    let player_id = PlayerId::random();

    let mut game_state = GameState::empty_from_level(
        MapId("usa_east".to_string()),
        MapLevel::load(include_str!("../../../assets/map_levels/usa_east.json")),
    );

    let from_tile = TileCoordsXZ::new(1, 190);
    let to_tile = TileCoordsXZ::new(255, 0);

    let head = DirectionalEdge::new(from_tile, DirectionXZ::West);
    let tail = DirectionalEdge::new(to_tile, DirectionXZ::South);

    let (tracks, length) = plan_tracks(
        player_id,
        head,
        &[tail],
        &game_state,
        DEFAULT_ALREADY_EXISTS_COEF,
    )
    .expect("Failed to plan tracks");

    assert!(tracks.len() > 450);
    assert!(length > TrackLength::new(300f32));
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
        track_type:  TrackType::NorthWest,
        pointing_in: DirectionXZ::North,
    };
    let route = find_route_to_tile_tracks(
        from_tile_track,
        &[to_tile_track],
        game_state.building_state(),
    )
    .unwrap();
    assert_eq!(route.len(), tracks.len());
}