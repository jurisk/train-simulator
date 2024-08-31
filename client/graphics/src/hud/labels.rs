use std::collections::BTreeMap;

use bevy::prelude::{Camera, EventWriter, GlobalTransform, Query, Res, Vec3};
use bevy_egui::EguiContexts;
use egui::{Align2, Context, Id, Pos2};
use shared_domain::building::building_info::WithTileCoverage;
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::cargo_map::WithCargo;
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::game_state::GameState;
use shared_domain::{GameId, IndustryBuildingId, PlayerId};

use crate::communication::domain::ClientMessageEvent;
use crate::game::{center_vec3, GameStateResource, PlayerIdResource};

fn with_tile_coverage_label(
    id: String,
    label: String,
    with_tile_coverage: &dyn WithTileCoverage,
    game_state: &GameState,
    context: &mut Context,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    let position_3d = center_vec3(with_tile_coverage, game_state.map_level());
    draw_label(label, position_3d, id, context, camera, camera_transform);
}

#[allow(clippy::needless_pass_by_value, clippy::module_name_repetitions)]
pub fn draw_labels(
    mut contexts: EguiContexts,
    game_state_resource: Option<Res<GameStateResource>>,
    player_id_resource: Res<PlayerIdResource>,
    camera_query: Query<(&GlobalTransform, &Camera)>,
    mut client_messages: EventWriter<ClientMessageEvent>,
) {
    if let Some((camera_transform, camera)) =
        camera_query.iter().find(|(_, camera)| camera.is_active)
    {
        if let Some(game_state_resource) = game_state_resource {
            let GameStateResource(game_state) = game_state_resource.as_ref();
            let PlayerIdResource(player_id) = player_id_resource.as_ref();

            let context = contexts.ctx_mut();

            draw_zoning_buttons(
                game_state,
                *player_id,
                context,
                camera,
                camera_transform,
                &mut client_messages,
            );
            draw_industry_labels(game_state, context, camera, camera_transform);
            draw_station_labels(game_state, context, camera, camera_transform);
            draw_transport_labels(game_state, context, camera, camera_transform);
        }
    }
}

fn draw_zoning_buttons(
    game_state: &GameState,
    player_id: PlayerId,
    context: &mut Context,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    client_messages: &mut EventWriter<ClientMessageEvent>,
) {
    for zoning_info in game_state.all_free_zonings() {
        let id = format!("{:?}", zoning_info.id());
        let game_id = game_state.game_id();
        let label = format!("{:?}", zoning_info.zoning_type());
        let position_3d = center_vec3(zoning_info, game_state.map_level());
        let mut sub_buttons: BTreeMap<String, Box<dyn FnOnce() -> GameCommand>> = BTreeMap::new();
        for industry_type in IndustryType::all() {
            let hypothetical_building = IndustryBuildingInfo::new(
                player_id,
                IndustryBuildingId::random(),
                zoning_info.reference_tile(),
                industry_type,
            );
            if game_state.can_build_industry_building(player_id, &hypothetical_building) {
                sub_buttons.insert(
                    format!("Build {industry_type:?}"),
                    Box::new(|| GameCommand::BuildIndustryBuilding(hypothetical_building)),
                );
            }
        }
        draw_menu(
            label,
            position_3d,
            id,
            context,
            camera,
            camera_transform,
            game_id,
            sub_buttons,
            client_messages,
        );
    }
}

fn draw_industry_labels(
    game_state: &GameState,
    context: &mut Context,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    for industry_building in game_state.building_state().all_industry_buildings() {
        let id = format!("{:?}", industry_building.id());

        let label = format!(
            "{:?} {:?}",
            industry_building.industry_type(),
            industry_building.cargo(),
        );

        with_tile_coverage_label(
            id,
            label,
            industry_building,
            game_state,
            context,
            camera,
            camera_transform,
        );
    }
}

fn draw_station_labels(
    game_state: &GameState,
    context: &mut Context,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    for station in game_state.building_state().all_stations() {
        let id = format!("{:?}", station.id());
        let label = format!("{:?} {:?}", station.reference_tile(), station.cargo());

        with_tile_coverage_label(
            id,
            label,
            station,
            game_state,
            context,
            camera,
            camera_transform,
        );
    }
}

fn draw_transport_labels(
    game_state: &GameState,
    context: &mut Context,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    for transport in game_state.transport_infos() {
        let label = transport.cargo_as_string();
        let id = format!("{:?}", transport.transport_id());
        let transport_location = transport.location();
        let transport_position_3d = transport_location.tile_path[0].progress_coordinates(
            transport_location.progress_within_tile,
            game_state.map_level().terrain(),
        );
        // TODO HIGH: This should be a button which opens/closes the transport menu
        draw_label(
            label,
            transport_position_3d,
            id,
            context,
            camera,
            camera_transform,
        );
    }
}

fn draw_in_position<F>(
    position: Vec3,
    id: String,
    context: &mut Context,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    f: F,
) where
    F: FnOnce(&mut egui::Ui),
{
    let position = project_to_screen(position, camera, camera_transform, context);

    egui::Area::new(Id::from(id))
        .fixed_pos(position)
        .pivot(Align2::CENTER_CENTER)
        .constrain(false)
        .show(context, f);
}

// TODO HIGH: This looks ugly and often breaks. Consider using https://docs.rs/egui/latest/egui/struct.Painter.html instead? Or https://bevyengine.org/examples/2d-rendering/text2d/ or https://github.com/kulkalkul/bevy_mod_billboard?
fn draw_label(
    label: String,
    position: Vec3,
    id: String,
    context: &mut Context,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    draw_in_position(position, id, context, camera, camera_transform, |ui| {
        ui.colored_label(egui::Color32::WHITE, label);
    });
}

#[allow(clippy::too_many_arguments)]
fn draw_menu(
    label: String,
    position: Vec3,
    id: String,
    context: &mut Context,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    game_id: GameId,
    sub_buttons: BTreeMap<String, Box<dyn FnOnce() -> GameCommand>>,
    client_messages: &mut EventWriter<ClientMessageEvent>,
) {
    draw_in_position(position, id, context, camera, camera_transform, |ui| {
        ui.menu_button(label, |ui| {
            for (sub_label, f) in sub_buttons {
                if ui.button(sub_label).clicked() {
                    let game_command = f();
                    let client_command = ClientCommand::Game(game_id, game_command);
                    client_messages.send(ClientMessageEvent::new(client_command));
                }
            }
        });
    });
}

#[allow(clippy::let_and_return)]
fn project_to_screen(
    position: Vec3,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    context: &Context,
) -> Pos2 {
    if let Some(ndc_space_coords) = camera.world_to_ndc(camera_transform, position) {
        let screen_size = context.screen_rect().size();
        let screen_position = Pos2::new(
            (ndc_space_coords.x + 1.0) * 0.5 * screen_size.x,
            (1.0 - ndc_space_coords.y) * 0.5 * screen_size.y,
        );

        screen_position
    } else {
        Pos2::ZERO
    }
}
