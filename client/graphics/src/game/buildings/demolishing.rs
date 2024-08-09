use bevy::input::ButtonInput;
use bevy::prelude::{EventWriter, MouseButton, Res};
use bevy_egui::EguiContexts;
use shared_domain::client_command::{ClientCommand, DemolishSelector, GameCommand};

use crate::communication::domain::ClientMessageEvent;
use crate::game::GameStateResource;
use crate::hud::domain::{DemolishType, SelectedMode};
use crate::on_ui;
use crate::selection::HoveredTile;

pub(crate) fn demolish_when_mouse_released(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    selected_mode_resource: Res<SelectedMode>,
    game_state_resource: Res<GameStateResource>,
    hovered_tile: Res<HoveredTile>,
    egui_contexts: EguiContexts,
    mut client_messages: EventWriter<ClientMessageEvent>,
) {
    if let Some(demolish_command) = demolish_command(
        mouse_buttons,
        selected_mode_resource,
        game_state_resource,
        hovered_tile,
        egui_contexts,
    ) {
        client_messages.send(demolish_command);
    }
}

fn demolish_command(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    selected_mode_resource: Res<SelectedMode>,
    game_state_resource: Res<GameStateResource>,
    hovered_tile: Res<HoveredTile>,
    mut egui_contexts: EguiContexts,
) -> Option<ClientMessageEvent> {
    // TODO: Can we do this conversion from `bool` to `Option<()>` in a more elegant way?
    if on_ui(&mut egui_contexts) {
        None
    } else {
        Some(())
    }?;
    if mouse_buttons.just_released(MouseButton::Left) {
        Some(())
    } else {
        None
    }?;

    let selected_mode = selected_mode_resource.as_ref();
    let HoveredTile(hovered_tile) = hovered_tile.as_ref();

    let hovered_tile = hovered_tile.as_ref().map(|hovered_tile| *hovered_tile)?;

    let GameStateResource(game_state) = game_state_resource.as_ref();
    let building_state = game_state.building_state();

    let game_id = game_state.game_id();

    let demolish_type = if let SelectedMode::Demolish(demolish_type) = selected_mode {
        *demolish_type
    } else {
        return None;
    };

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
            let tracks = building_state.tracks_at(hovered_tile);
            let selected_track = tracks.first()?; // Later: Don't just pick the first one
            GameCommand::Demolish(DemolishSelector::Track(selected_track.id()))
        },
    };

    let command = ClientMessageEvent::new(ClientCommand::Game(game_id, command));
    Some(command)
}
