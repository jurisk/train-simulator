use bevy::prelude::{ButtonInput, EventWriter, MouseButton, Res};
use bevy_egui::EguiContexts;
use shared_domain::client_command::ClientCommand;

use crate::communication::domain::ClientMessageEvent;
use crate::game::{GameStateResource, PlayerIdResource};
use crate::hud::domain::SelectedMode;
use crate::on_ui;
use crate::selection::HoveredTile;

pub(crate) fn build_building_when_mouse_released(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    selected_mode_resource: Res<SelectedMode>,
    game_state_resource: Res<GameStateResource>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    player_id_resource: Res<PlayerIdResource>,
    hovered_tile: Res<HoveredTile>,
    mut egui_contexts: EguiContexts,
) {
    if on_ui(&mut egui_contexts) {
        return;
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        let selected_mode = selected_mode_resource.as_ref();
        let HoveredTile(hovered_tile) = hovered_tile.as_ref();
        if let Some(hovered_tile) = hovered_tile {
            // Later: Check we can build this?

            let GameStateResource(game_state) = game_state_resource.as_ref();
            let PlayerIdResource(player_id) = *player_id_resource;

            let game_id = game_state.game_id();

            let command = selected_mode.build_building_command(player_id, *hovered_tile);

            // Later: Check we can build this? And that check is different for stations, as they can be built on top of fully straight tracks with no branching.

            if let Some(command) = command {
                client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                    game_id, command,
                )));
            }
        }
    }
}
