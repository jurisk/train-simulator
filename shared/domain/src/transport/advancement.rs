use log::{debug, info, warn};

use crate::building_state::BuildingState;
use crate::game_time::GameTimeDiff;
use crate::transport::movement_orders::MovementOrderLocation;
use crate::transport::progress_within_tile::ProgressWithinTile;
use crate::transport::tile_track::TileTrack;
use crate::transport::track_pathfinding::{find_location_tile_tracks, find_route_to};
use crate::transport::transport_info::{CargoLoading, TransportDynamicInfo, TransportInfo};
use crate::transport::transport_location::TransportLocation;
use crate::transport::transport_type::TransportType;

fn jump_tile(transport_info: &mut TransportInfo, building_state: &BuildingState) {
    debug!("Jumping tile: {:?}", transport_info);

    let transport_type = transport_info.transport_type().clone();
    let current_order = transport_info.dynamic_info.movement_orders.current_order();
    let route = find_route_to(
        transport_info.dynamic_info.location.next_tile_in_path(),
        current_order.go_to,
        building_state,
    );

    // The first one is the current tile, so we take the second one
    match route.unwrap_or_default().get(1) {
        None => {
            transport_info.dynamic_info.movement_orders.force_stop();
            warn!(
                "No route found for orders {current_order:?} for transport {:?}, stopping: {transport_info:?}",
                transport_info.transport_id()
            );
        },
        Some(next_tile_track) => {
            perform_jump(
                &mut transport_info.dynamic_info.location,
                &transport_type,
                *next_tile_track,
            );
            debug!("Finished jump: {:?}", transport_info);
        },
    };
}

fn at_location(
    transport_dynamic_info: &TransportDynamicInfo,
    target: MovementOrderLocation,
    building_state: &BuildingState,
) -> bool {
    let current_tile_path = transport_dynamic_info.location.next_tile_in_path();
    find_location_tile_tracks(target, building_state).is_some_and(|targets| {
        targets
            .into_iter()
            .any(|target| target == current_tile_path)
    })
}

fn advance_internal(
    transport_info: &mut TransportInfo,
    building_state: &mut BuildingState,
    diff: GameTimeDiff,
) -> GameTimeDiff {
    if transport_info
        .dynamic_info
        .movement_orders
        .is_force_stopped()
    {
        return GameTimeDiff::ZERO;
    }

    let progress_within_tile = transport_info.dynamic_info.location.progress_within_tile();
    if progress_within_tile == ProgressWithinTile::about_to_exit() {
        let current_orders = transport_info.dynamic_info.movement_orders.current_order();
        let at_location = at_location(
            &transport_info.dynamic_info,
            current_orders.go_to,
            building_state,
        );

        if at_location {
            if transport_info.dynamic_info.cargo_loading == CargoLoading::Finished {
                info!("Finished loading/unloading, advancing to next orders: {transport_info:?}");
                transport_info
                    .dynamic_info
                    .movement_orders
                    .advance_to_next_order();
                jump_tile(transport_info, building_state);
                transport_info.dynamic_info.cargo_loading = CargoLoading::NotStarted;
                diff
            } else {
                transport_info
                    .dynamic_info
                    .cargo_loading
                    .advance(building_state, diff)
            }
        } else {
            jump_tile(transport_info, building_state);
            diff
        }
    } else {
        advance_within_tile(transport_info, diff)
    }
}

pub fn advance(
    transport_info: &mut TransportInfo,
    building_state: &mut BuildingState,
    diff: GameTimeDiff,
) {
    loop {
        let remaining = advance_internal(transport_info, building_state, diff);
        if remaining == GameTimeDiff::ZERO {
            break;
        }
    }
}

fn advance_within_tile(transport_info: &mut TransportInfo, diff: GameTimeDiff) -> GameTimeDiff {
    let track_length = transport_info.dynamic_info.location.tile_path[0]
        .track_type
        .length();
    let distance_covered_this_tick = transport_info.dynamic_info.velocity * diff;
    let location = &mut transport_info.dynamic_info.location;
    let distance_remaining_in_tile = track_length
        * (ProgressWithinTile::about_to_exit() - location.progress_within_tile).as_f32();

    if distance_covered_this_tick >= distance_remaining_in_tile {
        // We jump tiles and use the remainder of our time for more actions (recursively)
        location.progress_within_tile = ProgressWithinTile::about_to_exit();
        let time_used = distance_remaining_in_tile / transport_info.dynamic_info.velocity;
        diff - time_used
    } else {
        location.progress_within_tile += distance_covered_this_tick / track_length;
        GameTimeDiff::ZERO
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::items_after_statements
)]
fn perform_jump(
    location: &mut TransportLocation,
    transport_type: &TransportType,
    next_tile_track: TileTrack,
) {
    location.tile_path.insert(0, next_tile_track);

    // Later: We are rather crudely sometimes removing the last element when we are inserting an
    // element.
    // This means - depending on `HEURISTIC_COEF` - that sometimes we will be carrying around
    // "too many tiles", or it could lead to running out of tiles if it is too short.
    // The alternative is to use `calculate_train_component_head_tails_and_final_tail_position`
    // to calculate the tail position, and then remove the last tiles if they are not needed,
    // but that introduces more complexity.
    const HEURISTIC_COEF: f32 = 2.0;
    if location.tile_path.len() > (HEURISTIC_COEF * transport_type.length_in_tiles()) as usize {
        let _ = location.tile_path.pop();
    }

    location.progress_within_tile = ProgressWithinTile::just_entering();
}