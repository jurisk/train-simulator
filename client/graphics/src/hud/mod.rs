use bevy::app::{App, Plugin};
use bevy::prelude::{ResMut, Resource, Update};
use bevy_egui::EguiContexts;
use egui;
use egui::{menu, Ui};
use shared_domain::production_type::ProductionType;
use shared_domain::station_type::StationOrientation;

#[derive(Resource, Eq, PartialEq, Debug)]
pub enum SelectedMode {
    Info,
    Tracks,
    Stations(StationOrientation),
    Production(ProductionType),
    Military,
    Trains,
    Demolish,
}

pub(crate) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, show_hud);
        app.insert_resource(SelectedMode::Info);
    }
}

const MIN_X: f32 = 200.0;
const MIN_Y: f32 = 40.0;
fn show_hud(mut contexts: EguiContexts, mut selected_mode: ResMut<SelectedMode>) {
    // Later: We need to better depict the current building mode in the menu, in case it's a sub-menu item that is selected

    let mut style = egui::Style::default();
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(24.0, egui::FontFamily::Proportional),
    );
    contexts.ctx_mut().set_style(style);

    egui::TopBottomPanel::top("hud_top_panel").show(contexts.ctx_mut(), |ui| {
        ui.style_mut().text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(32.0, egui::FontFamily::Proportional),
        );

        // The way we pass `ResMut<SelectedMode>` is on purpose, so that change detection works correctly.
        menu::bar(ui, |ui| {
            info_menu(&mut selected_mode, ui);
            tracks_menu(&mut selected_mode, ui);
            stations_menu(&mut selected_mode, ui);
            production_menu(&mut selected_mode, ui);
            military_menu(&mut selected_mode, ui);
            trains_menu(&mut selected_mode, ui);
            demolish_menu(&mut selected_mode, ui);
        });
    });
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
        println!("Info");
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
        println!("Tracks");
        *selected_mode.as_mut() = SelectedMode::Tracks;
        ui.close_menu();
    }
}

fn stations_menu(selected_mode: &mut ResMut<SelectedMode>, ui: &mut Ui) {
    // Later: We could build stations by just dragging the mouse, but it can wait.
    menu::menu_button(ui, "🚉 Stations", |ui| {
        if ui
            .add(
                egui::Button::new("⬌ East-West")
                    .selected(matches!(
                        *selected_mode.as_ref(),
                        SelectedMode::Stations(StationOrientation::EastToWest),
                    ))
                    .min_size(egui::vec2(MIN_X, MIN_Y)),
            )
            .clicked()
        {
            println!("EW Station");
            *selected_mode.as_mut() = SelectedMode::Stations(StationOrientation::EastToWest);
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new("⬍ North-South")
                    .selected(matches!(
                        *selected_mode.as_ref(),
                        SelectedMode::Stations(StationOrientation::NorthToSouth)
                    ))
                    .min_size(egui::vec2(MIN_X, MIN_Y)),
            )
            .clicked()
        {
            println!("NS Station");
            *selected_mode.as_mut() = SelectedMode::Stations(StationOrientation::NorthToSouth);
            ui.close_menu();
        }
    });
}

fn production_menu(selected_mode: &mut ResMut<SelectedMode>, ui: &mut Ui) {
    menu::menu_button(ui, "⚒ Production", |ui| {
        if ui
            .add(
                egui::Button::new("⛏ Iron Mine")
                    .selected(matches!(
                        *selected_mode.as_ref(),
                        SelectedMode::Production(ProductionType::IronMine)
                    ))
                    .min_size(egui::vec2(MIN_X, MIN_Y)),
            )
            .clicked()
        {
            println!("Iron Mine");
            *selected_mode.as_mut() = SelectedMode::Production(ProductionType::IronMine);
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new("⛏ Coal Mine")
                    .selected(matches!(
                        *selected_mode.as_ref(),
                        SelectedMode::Production(ProductionType::CoalMine)
                    ))
                    .min_size(egui::vec2(MIN_X, MIN_Y)),
            )
            .clicked()
        {
            println!("Coal Mine");
            *selected_mode.as_mut() = SelectedMode::Production(ProductionType::CoalMine);
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new("⚒ Iron Works")
                    .selected(matches!(
                        *selected_mode.as_ref(),
                        SelectedMode::Production(ProductionType::IronWorks)
                    ))
                    .min_size(egui::vec2(MIN_X, MIN_Y)),
            )
            .clicked()
        {
            println!("Iron Works");
            *selected_mode.as_mut() = SelectedMode::Production(ProductionType::IronWorks);
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new("⚓ Cargo Port")
                    .selected(matches!(
                        *selected_mode.as_ref(),
                        SelectedMode::Production(ProductionType::CargoPort)
                    ))
                    .min_size(egui::vec2(MIN_X, MIN_Y)),
            )
            .clicked()
        {
            println!("Cargo Port");
            *selected_mode.as_mut() = SelectedMode::Production(ProductionType::CargoPort);
            ui.close_menu();
        }
    });
}

fn military_menu(selected_mode: &mut ResMut<SelectedMode>, ui: &mut Ui) {
    menu::menu_button(ui, "⚔ Military", |ui| {
        if ui
            .add(
                egui::Button::new("⚔ Fixed Artillery")
                    .selected(matches!(*selected_mode.as_ref(), SelectedMode::Military))
                    .min_size(egui::vec2(MIN_X, MIN_Y)),
            )
            .clicked()
        {
            println!("Fixed Arty");
            *selected_mode.as_mut() = SelectedMode::Military;
            ui.close_menu();
        }
    });
}

fn trains_menu(selected_mode: &mut ResMut<SelectedMode>, ui: &mut Ui) {
    // Later: More types of trains
    menu::menu_button(ui, "🚆 Trains", |ui| {
        if ui
            .add(
                egui::Button::new("🚆 Coal Train")
                    .selected(matches!(*selected_mode.as_ref(), SelectedMode::Trains))
                    .min_size(egui::vec2(MIN_X, MIN_Y)),
            )
            .clicked()
        {
            println!("Coal Train");
            *selected_mode.as_mut() = SelectedMode::Trains;
            ui.close_menu();
        }
    });
}

fn demolish_menu(selected_mode: &mut ResMut<SelectedMode>, ui: &mut Ui) {
    if ui
        .add(
            egui::Button::new("❎ Demolish")
                .selected(matches!(*selected_mode.as_ref(), SelectedMode::Demolish))
                .min_size(egui::vec2(MIN_X, MIN_Y)),
        )
        .clicked()
    {
        println!("Demolish");
        *selected_mode.as_mut() = SelectedMode::Demolish;
        ui.close_menu();
    }
}
