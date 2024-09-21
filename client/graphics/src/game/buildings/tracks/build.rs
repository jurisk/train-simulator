use bevy::input::ButtonInput;
use bevy::prelude::{EventWriter, MouseButton, Res, ResMut};
use shared_domain::client_command::{ClientCommand, GameCommand};

use crate::communication::domain::ClientMessageEvent;
use crate::game::GameStateResource;
use crate::game::buildings::tracks::preview::TrackPreviewResource;
use crate::hud::domain::{SelectedMode, TracksBuildingType};
use crate::selection::{SelectedEdges, SelectedTiles};

pub(crate) fn build_tracks_when_mouse_released(
    mut selected_tiles: ResMut<SelectedTiles>,
    mut selected_edges: ResMut<SelectedEdges>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    game_state_resource: Res<GameStateResource>,
    mut selected_mode_resource: ResMut<SelectedMode>,
    mut track_preview_resource: ResMut<TrackPreviewResource>,
) {
    if mouse_buttons.just_released(MouseButton::Left) {
        let selected_mode = selected_mode_resource.as_ref();
        if let SelectedMode::Tracks(TracksBuildingType::SelectEnd { start: _start }) = selected_mode
        {
            // Later: Could this clearing of selections be done more elegantly elsewhere?
            let _ordered_selected_tiles = selected_tiles.take();
            let _ordered_selected_edges = selected_edges.take();
            *selected_mode_resource = SelectedMode::Tracks(TracksBuildingType::SelectStart);

            let GameStateResource(game_state) = game_state_resource.as_ref();
            let tracks = track_preview_resource.take();

            if !tracks.is_empty() {
                client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                    game_state.game_id(),
                    GameCommand::BuildTracks(tracks),
                )));
            }
        }
    }
}
