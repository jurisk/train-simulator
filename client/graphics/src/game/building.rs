use bevy::prelude::{ButtonInput, EventWriter, MouseButton, Res};
use shared_domain::client_command::ClientCommand;

use crate::communication::domain::ClientMessageEvent;
use crate::game::{GameStateResource, PlayerIdResource};
use crate::hud::PointerOverHud;
use crate::hud::domain::SelectedMode;
use crate::selection::HoveredTile;

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn build_something_when_mouse_released(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    selected_mode_resource: Res<SelectedMode>,
    game_state_resource: Res<GameStateResource>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    player_id_resource: Res<PlayerIdResource>,
    hovered_tile: Res<HoveredTile>,
    pointer_over_hud: Res<PointerOverHud>,
) {
    if pointer_over_hud.get() {
        return;
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        let selected_mode = selected_mode_resource.as_ref();
        let HoveredTile(hovered_tile) = hovered_tile.as_ref();
        if let Some(hovered_tile) = hovered_tile {
            let GameStateResource(game_state) = game_state_resource.as_ref();
            let PlayerIdResource(player_id) = *player_id_resource;

            let game_id = game_state.game_id();

            let command = selected_mode.build_something_command(player_id, *hovered_tile);

            // Later: Check we can build this "something"? And do it invoking the real 'GameState'.

            if let Some(command) = command {
                client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                    game_id, command,
                )));
            }
        }
    }
}
