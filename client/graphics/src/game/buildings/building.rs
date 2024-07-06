use bevy::prelude::{
    Assets, ButtonInput, Color, Commands, EventWriter, Handle, Mesh, MouseButton, Name, PbrBundle,
    Res, ResMut, StandardMaterial, Transform,
};
use bevy_egui::EguiContexts;
use shared_domain::building_info::BuildingInfo;
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::Colour;
use shared_domain::tile_coverage::TileCoverage;
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
    mut egui_contexts: EguiContexts,
) {
    if egui_contexts.ctx_mut().is_pointer_over_area() {
        return;
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        if let Some(building_type) = selected_mode_resource.as_ref().corresponding_building() {
            let HoveredTile(hovered_tile) = hovered_tile.as_ref();
            if let Some(hovered_tile) = hovered_tile {
                // Later: Check we can build this?

                let GameStateResource(game_state) = game_state_resource.as_ref();
                let PlayerIdResource(player_id) = *player_id_resource;

                let game_id = game_state.game_id();

                // Later: Check we can build this? And that check is different for stations, as they can be built on top of fully straight tracks with no branching.
                let building = BuildingInfo::new(
                    player_id,
                    BuildingId::random(),
                    *hovered_tile,
                    building_type,
                );
                let buildings = vec![building];

                client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                    game_id,
                    GameCommand::BuildBuildings(buildings),
                )));
            }
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::similar_names)]
pub(crate) fn create_building_entity(
    tile_coverage: TileCoverage,
    label: String,
    colour: Colour,
    mesh: Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    commands: &mut Commands,
    map_level: &MapLevel,
) {
    let terrain = &map_level.terrain();
    let (nw, se) = match tile_coverage {
        TileCoverage::Empty => panic!("Cannot create a building with no tiles"),
        TileCoverage::Single(tile) => (tile, tile),
        TileCoverage::Rectangular {
            north_west_inclusive,
            south_east_inclusive,
        } => (north_west_inclusive, south_east_inclusive),
    };
    let nw = nw.vertex_coords_nw();
    let se = se.vertex_coords_se();
    let nw = terrain.logical_to_world(nw);
    let se = terrain.logical_to_world(se);

    let color = Color::rgb_u8(colour.r, colour.g, colour.b);
    let material = materials.add(color);
    let center = (se + nw) / 2.0;

    // TODO HIGH: Use `label` to also draw text on the sides / roof of the building

    commands.spawn((
        PbrBundle {
            transform: Transform {
                translation: center,
                ..Default::default()
            },
            material,
            mesh,
            ..Default::default()
        },
        Name::new(label),
    ));
}
