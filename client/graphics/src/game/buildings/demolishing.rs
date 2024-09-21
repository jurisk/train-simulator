use bevy::input::ButtonInput;
use bevy::prelude::{EventWriter, MouseButton, Res};
use shared_domain::TrackId;
use shared_domain::client_command::{ClientCommand, DemolishSelector, GameCommand};
use shared_util::bool_ops::BoolOptionOps;

use crate::communication::domain::ClientMessageEvent;
use crate::game::GameStateResource;
use crate::hud::PointerOverHud;
use crate::hud::domain::{DemolishType, SelectedMode};
use crate::selection::HoveredTile;

pub(crate) fn demolish_when_mouse_released(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    selected_mode_resource: Res<SelectedMode>,
    game_state_resource: Res<GameStateResource>,
    hovered_tile: Res<HoveredTile>,
    pointer_over_hud: Res<PointerOverHud>,
    mut client_messages: EventWriter<ClientMessageEvent>,
) {
    if let Some(demolish_command) = demolish_command(
        mouse_buttons,
        selected_mode_resource,
        game_state_resource,
        hovered_tile,
        pointer_over_hud,
    ) {
        client_messages.send(demolish_command);
    }
}

fn demolish_command(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    selected_mode_resource: Res<SelectedMode>,
    game_state_resource: Res<GameStateResource>,
    hovered_tile: Res<HoveredTile>,
    pointer_over_hud: Res<PointerOverHud>,
) -> Option<ClientMessageEvent> {
    pointer_over_hud.get().then_none()?;
    mouse_buttons
        .just_released(MouseButton::Left)
        .then_some_unit()?;

    let selected_mode = selected_mode_resource.as_ref();
    let HoveredTile(hovered_tile) = hovered_tile.as_ref();

    let hovered_tile = hovered_tile.as_ref().map(|hovered_tile| *hovered_tile)?;

    let GameStateResource(game_state) = game_state_resource.as_ref();
    let building_state = game_state.building_state();

    let game_id = game_state.game_id();

    let demolish_type = if let SelectedMode::Demolish(demolish_type) = selected_mode {
        Some(*demolish_type)
    } else {
        None
    }?;

    let command = match demolish_type {
        DemolishType::Industry => {
            let industry_building = building_state.industry_building_at(hovered_tile)?;
            GameCommand::Demolish(DemolishSelector::Industry(industry_building.id()))
        },
        DemolishType::Station => {
            let station = building_state.station_at(hovered_tile)?;
            GameCommand::Demolish(DemolishSelector::Station(station.id()))
        },
        DemolishType::Tracks => {
            // TODO: We should let the user to drag the mouse to select which tracks to demolish
            let tracks = building_state.tracks_at(hovered_tile);
            let track_types = tracks.track_types();
            let track_ids = track_types
                .into_iter()
                .map(|track_type| TrackId::new(hovered_tile, track_type))
                .collect();
            GameCommand::Demolish(DemolishSelector::Tracks(track_ids))
        },
    };

    let command = ClientMessageEvent::new(ClientCommand::Game(game_id, command));
    Some(command)
}
