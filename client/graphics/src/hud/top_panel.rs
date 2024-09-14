use bevy::prelude::ResMut;
use bevy_egui::EguiContexts;
use egui::{menu, Ui};
use shared_domain::building::industry_type::IndustryType;
use shared_domain::building::station_type::{StationOrientation, StationType};
use shared_domain::resource_type::ResourceType;
use shared_domain::transport::transport_type::TransportType;

use crate::ai::ArtificialIntelligenceTimer;
use crate::hud::domain::{DemolishType, SelectedMode};

const MIN_X: f32 = 200.0;
const MIN_Y: f32 = 40.0;

pub(crate) fn show_top_panel(
    mut contexts: EguiContexts,
    mut selected_mode: ResMut<SelectedMode>,
    mut ai_timer: ResMut<ArtificialIntelligenceTimer>,
) {
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
            ai_menu(&mut ai_timer, ui);
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

#[expect(clippy::match_same_arms)]
fn industry_menu(selected_mode: &mut ResMut<SelectedMode>, ui: &mut Ui) {
    menu::menu_button(ui, "⚒ Industry", |ui| {
        set_font_size(ui, 24.0);

        for industry_type in IndustryType::all() {
            let symbol = match industry_type {
                IndustryType::IronMine => "⚒",
                IndustryType::CoalMine => "⛏",
                IndustryType::SteelMill => "🏭",
                IndustryType::MilitaryBase => "💣",
                IndustryType::ConstructionYard => "👷",
                IndustryType::OilWell => "🛢",
                IndustryType::NitrateMine => "💣",
                IndustryType::SulfurMine => "🧪",
                IndustryType::Farm => "🚜",
                IndustryType::Forestry => "🌲",
                IndustryType::ClayPit => "🏺",
                IndustryType::LimestoneMine => "🪨",
                IndustryType::SandAndGravelQuarry => "🪨",
                IndustryType::PowerPlant => "⚡",
                IndustryType::CoalToOilPlant => "🛢",
                IndustryType::ExplosivesPlant => "💣",
                IndustryType::FoodProcessingPlant => "🍖",
                IndustryType::LumberMill => "🪵",
                IndustryType::CementPlant => "🪵",
                IndustryType::OilRefinery => "🛢",
                IndustryType::ConcretePlant => "🪵",
                IndustryType::TrainFactory => "🚆",
                IndustryType::WeaponsFactory => "🔫",
                IndustryType::AmmunitionFactory => "🔫",
            };
            if ui
                .add(
                    egui::Button::new(format!("{symbol} {industry_type:?}"))
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

// TODO HIGH: Fixed artillery, movable artillery, troops, trenches? Need to think carefully about the model and what's needed and what's not.
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
    menu::menu_button(ui, "❎ Demolish", |ui| {
        set_font_size(ui, 24.0);

        for (name, mode) in [
            ("🚉 Stations", SelectedMode::Demolish(DemolishType::Station)),
            ("⚒ Industry", SelectedMode::Demolish(DemolishType::Industry)),
            ("🚆 Tracks", SelectedMode::Demolish(DemolishType::Tracks)),
        ] {
            if ui
                .add(
                    egui::Button::new(name)
                        .selected(*selected_mode.as_ref() == mode)
                        .min_size(egui::vec2(MIN_X, MIN_Y)),
                )
                .clicked()
            {
                *selected_mode.as_mut() = mode;
                ui.close_menu();
            }
        }
    });
}

fn ai_menu(ai_timer: &mut ResMut<ArtificialIntelligenceTimer>, ui: &mut Ui) {
    // Later: Could disable the currently selected AI mode, but that does not matter much
    menu::menu_button(ui, "🖥 AI", |ui| {
        set_font_size(ui, 24.0);

        if ui
            .add(egui::Button::new("❎ Disable").min_size(egui::vec2(MIN_X, MIN_Y)))
            .clicked()
        {
            ai_timer.as_mut().disable();
            ui.close_menu();
        }

        for (name, seconds) in [
            ("☑ Enable 100 milliseconds", 0.1),
            ("☑ Enable 1 second", 1.0),
            ("☑ Enable 10 seconds", 10.0),
        ] {
            if ui
                .add(egui::Button::new(name).min_size(egui::vec2(MIN_X, MIN_Y)))
                .clicked()
            {
                ai_timer.as_mut().enable(seconds);
                ui.close_menu();
            }
        }
    });
}
