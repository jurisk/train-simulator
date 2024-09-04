use bevy::input::ButtonInput;
use bevy::prelude::{EventWriter, MouseButton, Res, ResMut};
use bevy_egui::EguiContexts;
use shared_domain::client_command::{ClientCommand, GameCommand};

use crate::communication::domain::ClientMessageEvent;
use crate::game::buildings::tracks::plan::try_plan_tracks;
use crate::game::{GameStateResource, PlayerIdResource};
use crate::hud::domain::SelectedMode;
use crate::selection::{SelectedEdges, SelectedTiles};

// TODO HIGH: Store the track preview in a resource so we can reuse it when building tracks
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_tracks_when_mouse_released(
    mut selected_tiles: ResMut<SelectedTiles>,
    mut selected_edges: ResMut<SelectedEdges>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    player_id_resource: Res<PlayerIdResource>,
    game_state_resource: Res<GameStateResource>,
    selected_mode_resource: Res<SelectedMode>,
    egui_contexts: EguiContexts,
) {
    if selected_mode_resource.as_ref() != &SelectedMode::Tracks {
        return;
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        // Later: Could this clearing of selections be done more elegantly elsewhere?
        let _ordered_selected_tiles = selected_tiles.take();
        let ordered_selected_edges = selected_edges.take();

        let GameStateResource(game_state) = game_state_resource.as_ref();

        if let Some(tracks) = try_plan_tracks(
            player_id_resource,
            game_state,
            &ordered_selected_edges,
            egui_contexts,
        ) {
            client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                game_state.game_id(),
                GameCommand::BuildTracks(tracks),
            )));
        }
    }
}
