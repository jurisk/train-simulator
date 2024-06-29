use bevy::prelude::{ButtonInput, EventWriter, MouseButton, Res};
use shared_domain::building_info::BuildingInfo;
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::BuildingId;

use crate::communication::domain::ClientMessageEvent;
use crate::game::{GameStateResource, PlayerIdResource};
use crate::hud::domain::SelectedMode;
use crate::selection::HoveredTile;

pub(crate) fn build_building_when_mouse_released(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    selected_mode_resource: Res<SelectedMode>,
    game_state_resource: Res<GameStateResource>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    player_id_resource: Res<PlayerIdResource>,
    hovered_tile: Res<HoveredTile>,
) {
    if mouse_buttons.just_released(MouseButton::Left) {
        if let Some(building_type) = selected_mode_resource.as_ref().corresponding_building() {
            let HoveredTile(hovered_tile) = hovered_tile.as_ref();
            if let Some(hovered_tile) = hovered_tile {
                // Later: Check we can build this?

                let GameStateResource(game_state) = game_state_resource.as_ref();
                let PlayerIdResource(player_id) = *player_id_resource;

                let game_id = game_state.game_id();

                // Later: Check we can build this? And that check is different for stations, as they can be built on top of fully straight tracks with no branching.
                let building = BuildingInfo {
                    owner_id: player_id,
                    building_id: BuildingId::random(),
                    reference_tile: *hovered_tile,
                    building_type,
                };
                let buildings = vec![building];

                client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                    game_id,
                    GameCommand::BuildBuildings(buildings),
                )));
            }
        }
    }
}
