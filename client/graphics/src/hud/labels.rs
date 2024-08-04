use bevy::prelude::{Camera, GlobalTransform, Query, Res, Vec3};
use bevy_egui::EguiContexts;
use egui::{Align2, Context, Id, Pos2};
use shared_domain::building::building_type::BuildingType;

use crate::game::buildings::building::center_vec3;
use crate::game::GameStateResource;

#[allow(clippy::needless_pass_by_value, clippy::module_name_repetitions)]
pub fn draw_labels(
    mut contexts: EguiContexts,
    game_state_resource: Option<Res<GameStateResource>>,
    camera_query: Query<(&GlobalTransform, &Camera)>,
) {
    if let Some((camera_transform, camera)) =
        camera_query.iter().find(|(_, camera)| camera.is_active)
    {
        if let Some(game_state_resource) = game_state_resource {
            let GameStateResource(game_state) = game_state_resource.as_ref();

            let context = contexts.ctx_mut();
            let buildings = game_state.building_state();
            for building in buildings.to_vec() {
                match building.building_type() {
                    BuildingType::Track(_) => {},
                    BuildingType::Station(_) => {
                        let label = format!("{:?}", building.dynamic_info());
                        let id = format!("{:?}", building.building_id());
                        let building_position_3d = center_vec3(&building, game_state.map_level());
                        draw_label(
                            building_position_3d,
                            label,
                            id,
                            context,
                            camera,
                            camera_transform,
                        );
                    },
                    BuildingType::Industry(industry_type) => {
                        let label = format!("{industry_type:?} {:?}", building.dynamic_info());
                        let id = format!("{:?}", building.building_id());
                        let building_position_3d = center_vec3(&building, game_state.map_level());
                        draw_label(
                            building_position_3d,
                            label,
                            id,
                            context,
                            camera,
                            camera_transform,
                        );
                    },
                }
            }

            let transports = game_state.transport_infos();
            for transport in transports {
                // TODO: The cargo label could actually be "I 1.0 of 2.0" or similar
                let label = transport.cargo_as_string();
                let id = format!("{:?}", transport.transport_id());
                let transport_location = transport.location();
                let transport_position_3d = transport_location.tile_path[0].progress_coordinates(
                    transport_location.progress_within_tile,
                    game_state.map_level().terrain(),
                );
                draw_label(
                    transport_position_3d,
                    label,
                    id,
                    context,
                    camera,
                    camera_transform,
                );
            }
        }
    }
}

// TODO HIGH: This looks ugly and often breaks. Consider using https://docs.rs/egui/latest/egui/struct.Painter.html instead? Or https://bevyengine.org/examples/2d-rendering/text2d/ or https://github.com/kulkalkul/bevy_mod_billboard?
fn draw_label(
    position: Vec3,
    label: String,
    id: String,
    context: &mut Context,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    let label_position = project_to_screen(position, camera, camera_transform, context);

    egui::Area::new(Id::from(id))
        .fixed_pos(label_position)
        .pivot(Align2::CENTER_CENTER)
        .constrain(false)
        .show(context, |ui| {
            ui.colored_label(egui::Color32::WHITE, label);
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
