use bevy::prelude::{Camera, GlobalTransform, Query, Res, Vec3};
use bevy_egui::EguiContexts;
use egui::{Id, Pos2};
use shared_domain::building_type::BuildingType;

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
                if let BuildingType::Production(production_type) = building.building_type() {
                    let label = format!("{production_type:?} {:?}", building.dynamic_info());
                    let building_position_3d = center_vec3(&building, game_state.map_level());
                    let label_position =
                        project_to_screen(building_position_3d, camera, camera_transform, context);

                    egui::Area::new(Id::from(format!("{:?}", building.id())))
                        .fixed_pos(label_position)
                        .show(context, |ui| {
                            ui.label(label);
                        });
                }
            }
        }
    }
}

#[allow(clippy::let_and_return)]
fn project_to_screen(
    position: Vec3,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    context: &egui::Context,
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
