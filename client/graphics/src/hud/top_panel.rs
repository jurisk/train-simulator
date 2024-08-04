use bevy::prelude::ResMut;
use bevy_egui::EguiContexts;
use egui::{menu, Ui};
use shared_domain::industry_type::IndustryType;
use shared_domain::resource_type::ResourceType;
use shared_domain::station_type::{StationOrientation, StationType};
use shared_domain::transport::transport_type::TransportType;

use crate::hud::domain::SelectedMode;

const MIN_X: f32 = 200.0;
const MIN_Y: f32 = 40.0;

pub(crate) fn show_top_panel(mut contexts: EguiContexts, mut selected_mode: ResMut<SelectedMode>) {
    // Later: We need to better depict the current building mode in the main menu, in case it's a sub-menu item that is selected

    egui::TopBottomPanel::top("hud_top_panel").show(contexts.ctx_mut(), |ui| {
        set_font_size(ui, 32.0);

        // The way we pass `ResMut<SelectedMode>` is on purpose, so that change detection works correctly.
        menu::bar(ui, |ui| {
            // Later: Landscaping for terrain modification
            info_menu(&mut selected_mode, ui);
            tracks_menu(&mut selected_mode, ui);
            stations_menu(&mut selected_mode, ui);
            industry_menu(&mut selected_mode, ui);
            military_menu(&mut selected_mode, ui);
            trains_menu(&mut selected_mode, ui);
            demolish_menu(&mut selected_mode, ui);
        });
    });
}

fn set_font_size(ui: &mut Ui, size: f32) {
    ui.style_mut().text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(size, egui::FontFamily::Proportional),
    );
}

fn info_menu(selected_mode: &mut ResMut<SelectedMode>, ui: &mut Ui) {
    if ui
        .add(
            egui::Button::new("ℹ Info")
                .selected(matches!(*selected_mode.as_ref(), SelectedMode::Info))
                .min_size(egui::vec2(MIN_X, MIN_Y)),
        )
        .clicked()
    {
        *selected_mode.as_mut() = SelectedMode::Info;
        ui.close_menu();
    }
}

fn tracks_menu(selected_mode: &mut ResMut<SelectedMode>, ui: &mut Ui) {
    if ui
        .add(
            egui::Button::new("🚆 Tracks")
                .selected(matches!(*selected_mode.as_ref(), SelectedMode::Tracks))
                .min_size(egui::vec2(MIN_X, MIN_Y)),
        )
        .clicked()
    {
        *selected_mode.as_mut() = SelectedMode::Tracks;
        ui.close_menu();
    }
}

fn stations_menu(selected_mode: &mut ResMut<SelectedMode>, ui: &mut Ui) {
    // Later: We could build stations by just dragging the mouse, but it can wait.
    menu::menu_button(ui, "🚉 Stations", |ui| {
        set_font_size(ui, 24.0);

        for station_type in StationType::all() {
            let symbol = match station_type.orientation {
                StationOrientation::NorthToSouth => "⬍ NS",
                StationOrientation::EastToWest => "⬌ EW",
            };
            if ui
                .add(
                    egui::Button::new(format!(
                        "{symbol} {} × {}",
                        station_type.length_in_tiles, station_type.platforms
                    ))
                    .selected(*selected_mode.as_ref() == SelectedMode::Stations(station_type))
                    .min_size(egui::vec2(MIN_X, MIN_Y)),
                )
                .clicked()
            {
                *selected_mode.as_mut() = SelectedMode::Stations(station_type);
                ui.close_menu();
            }
        }
    });
}

#[allow(clippy::match_same_arms)]
fn industry_menu(selected_mode: &mut ResMut<SelectedMode>, ui: &mut Ui) {
    menu::menu_button(ui, "⚒ Industry", |ui| {
        set_font_size(ui, 24.0);

        for industry_type in IndustryType::all() {
            let symbol = match industry_type {
                IndustryType::IronMine => "⚒",
                IndustryType::CoalMine => "⛏",
                IndustryType::IronWorks => "🏭",
                IndustryType::Warehouse => "📦",
            };
            if ui
                .add(
                    egui::Button::new(format!("{symbol} {industry_type:#?}"))
                        .selected(*selected_mode.as_ref() == SelectedMode::Industry(industry_type))
                        .min_size(egui::vec2(MIN_X, MIN_Y)),
                )
                .clicked()
            {
                *selected_mode.as_mut() = SelectedMode::Industry(industry_type);
                ui.close_menu();
            }
        }
    });
}

// Later: Fixed artillery, movable artillery, troops, trenches? Need to think carefully about the model and what's needed and what's not.
fn military_menu(selected_mode: &mut ResMut<SelectedMode>, ui: &mut Ui) {
    menu::menu_button(ui, "⚔ Military", |ui| {
        set_font_size(ui, 24.0);
        if ui
            .add(
                egui::Button::new("⚔ Fixed Artillery")
                    .selected(matches!(*selected_mode.as_ref(), SelectedMode::Military))
                    .min_size(egui::vec2(MIN_X, MIN_Y)),
            )
            .clicked()
        {
            *selected_mode.as_mut() = SelectedMode::Military;
            ui.close_menu();
        }
    });
}

fn trains_menu(selected_mode_res: &mut ResMut<SelectedMode>, ui: &mut Ui) {
    // Later: More types of trains
    menu::menu_button(ui, "🚆 Trains", |ui| {
        set_font_size(ui, 24.0);

        for resource_type in ResourceType::all() {
            let transport_type = TransportType::cargo_train(resource_type);
            let selected_mode = selected_mode_res.as_ref();
            if ui
                .add(
                    egui::Button::new(format!("🚆 {resource_type:?} Train"))
                        .selected(*selected_mode == SelectedMode::Transport(transport_type.clone()))
                        .min_size(egui::vec2(MIN_X, MIN_Y)),
                )
                .clicked()
            {
                *selected_mode_res.as_mut() = SelectedMode::Transport(transport_type);
                ui.close_menu();
            }
        }
    });
}

fn demolish_menu(selected_mode: &mut ResMut<SelectedMode>, ui: &mut Ui) {
    if ui
        .add(
            // TODO HIGH: Implement - possibly separate demolish for tracks vs buildings
            egui::Button::new("❎ Demolish")
                .selected(matches!(*selected_mode.as_ref(), SelectedMode::Demolish))
                .min_size(egui::vec2(MIN_X, MIN_Y)),
        )
        .clicked()
    {
        *selected_mode.as_mut() = SelectedMode::Demolish;
        ui.close_menu();
    }
}
