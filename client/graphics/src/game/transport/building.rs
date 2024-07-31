use bevy::input::ButtonInput;
use bevy::prelude::{info, EventWriter, MouseButton, Res};
use bevy_egui::EguiContexts;
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::edge_xz::EdgeXZ;
use shared_domain::game_state::GameState;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::transport::movement_orders::{MovementOrder, MovementOrders};
use shared_domain::transport::transport_info::TransportInfo;
use shared_domain::transport::transport_type::TransportType;
use shared_domain::{PlayerId, TransportId};

use crate::communication::domain::ClientMessageEvent;
use crate::game::{GameStateResource, PlayerIdResource};
use crate::hud::domain::SelectedMode;
use crate::on_ui;
use crate::selection::{HoveredEdge, HoveredTile};

fn build_transport_command(
    player_id: &PlayerId,
    transport_type: &TransportType,
    game_state: &GameState,
    hovered_edge: &Option<EdgeXZ>,
    hovered_tile: &Option<TileCoordsXZ>,
) -> Result<ClientCommand, String> {
    let game_id = game_state.game_id();
    let edge = hovered_edge.ok_or("No edge hovered")?;
    let tile = hovered_tile.ok_or("No tile hovered")?;
    let options = edge
        .both_tiles_and_directions()
        .into_iter()
        .filter(|(t, _direction)| *t == tile)
        .collect::<Vec<_>>();
    let (_t, direction) = options.first().ok_or("Failed to find direction")?;
    let building = game_state
        .building_state()
        .station_at(tile)
        .ok_or(format!("No station at tile {tile:?}"))?;
    let location = building
        .transport_location_at_station(tile, *direction)
        .ok_or(format!("No transport location at tile {tile:?}"))?;
    let movement_orders =
        MovementOrders::one(MovementOrder::stop_at_station(building.building_id()));
    let transport_info = TransportInfo::new(
        TransportId::random(),
        *player_id,
        transport_type.clone(),
        location,
        movement_orders,
    );
    let result = ClientCommand::Game(game_id, GameCommand::PurchaseTransport(transport_info));
    Ok(result)
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub(crate) fn build_transport_when_mouse_released(
    hovered_edge: Res<HoveredEdge>,
    hovered_tile: Res<HoveredTile>,
    game_state_resource: Res<GameStateResource>,
    player_id_resource: Res<PlayerIdResource>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut egui_contexts: EguiContexts,
    selected_mode_resource: Res<SelectedMode>,
    mut client_messages: EventWriter<ClientMessageEvent>,
) {
    if on_ui(egui_contexts.ctx_mut()) {
        return;
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        let selected_mode = selected_mode_resource.as_ref();
        if let SelectedMode::Transport(transport_type) = selected_mode {
            let GameStateResource(game_state) = game_state_resource.as_ref();
            let PlayerIdResource(player_id) = player_id_resource.as_ref();
            let HoveredEdge(hovered_edge) = hovered_edge.as_ref();
            let HoveredTile(hovered_tile) = hovered_tile.as_ref();

            match build_transport_command(
                player_id,
                transport_type,
                game_state,
                hovered_edge,
                hovered_tile,
            ) {
                Ok(command) => {
                    client_messages.send(ClientMessageEvent::new(command));
                },
                Err(error) => {
                    info!("Failed to build transport: {}", error);
                },
            }
        }
    }
}
