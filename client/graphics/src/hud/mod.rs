#![expect(clippy::module_name_repetitions)]

use bevy::app::{App, Plugin, Update};
use bevy::prelude::{IntoSystemConfigs, ResMut, Resource, default, in_state};
use bevy_egui::EguiPlugin;
use egui::text::LayoutJob;
use egui::{Color32, TextFormat, Ui};
use shared_domain::PlayerId;
use shared_domain::server_response::PlayerInfo;

use crate::hud::domain::SelectedMode;
use crate::hud::labels::draw_labels;
use crate::states::ClientState;

pub mod bottom_panel;
pub mod domain;
mod helpers;
pub mod labels;
pub mod left_panel;
pub mod top_panel;

#[derive(Resource, Default)]
pub struct PointerOverHud {
    previous: bool,
    current:  bool,
}

impl PointerOverHud {
    pub fn next_frame(&mut self) {
        self.previous = self.current;
        self.current = false;
    }

    #[must_use]
    pub fn get(&self) -> bool {
        // This opens doors to some race conditions, but I hope it will be OK
        self.previous
    }

    fn apply(&mut self, ui: &Ui) {
        // Related discussion https://github.com/emilk/egui/discussions/4996
        if ui.rect_contains_pointer(ui.max_rect()) {
            self.current = true;
        }
    }
}

pub(crate) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        app.insert_resource(SelectedMode::Info);
        app.insert_resource(PointerOverHud::default());

        app.add_systems(
            Update,
            reset_pointer_over_hud.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            Update,
            bottom_panel::show_bottom_panel
                .after(reset_pointer_over_hud)
                .run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            Update,
            top_panel::show_top_panel
                .after(bottom_panel::show_bottom_panel)
                .run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            Update,
            left_panel::show_left_panel
                .after(top_panel::show_top_panel)
                .run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            Update,
            draw_labels
                .after(left_panel::show_left_panel)
                .run_if(in_state(ClientState::Playing)),
        );
    }
}

pub fn reset_pointer_over_hud(mut pointer_over_hud: ResMut<PointerOverHud>) {
    pointer_over_hud.next_frame();
}

#[expect(clippy::similar_names)]
#[must_use]
pub fn player_layout_job(own_player_id: PlayerId, player_info: &PlayerInfo) -> LayoutJob {
    let colour = player_info.colour;
    let color = Color32::from_rgb(colour.r, colour.g, colour.b);

    let mut job = LayoutJob::default();
    job.append("⬛", 0.0, TextFormat { color, ..default() });

    job.append(
        format!("{}", player_info.name).as_str(),
        0.0,
        TextFormat::default(),
    );

    if player_info.id == own_player_id {
        job.append(" ⬅", 0.0, TextFormat::default());
    }

    job
}
