use bevy::app::{App, Plugin};
use bevy::prelude::{ResMut, Resource, Update};
use bevy_egui::EguiContexts;
use egui;
use egui::{menu, Ui};
use shared_domain::production_type::ProductionType;

#[derive(Resource, Eq, PartialEq, Debug)]
pub enum SelectedMode {
    Info,
    Tracks,
    Stations,
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

        menu::bar(ui, |ui| {
            if ui
                .add(
                    egui::Button::new("ℹ Info")
                        .selected(matches!(*selected_mode, SelectedMode::Info))
                        .min_size(egui::vec2(MIN_X, MIN_Y)),
                )
                .clicked()
            {
                println!("Info");
                *selected_mode = SelectedMode::Info;
                ui.close_menu();
            }

            if ui
                .add(
                    egui::Button::new("🚆 Tracks")
                        .selected(matches!(*selected_mode, SelectedMode::Tracks))
                        .min_size(egui::vec2(MIN_X, MIN_Y)),
                )
                .clicked()
            {
                println!("Tracks");
                *selected_mode = SelectedMode::Tracks;
                ui.close_menu();
            }

            if ui
                .add(
                    egui::Button::new("🚉 Stations")
                        .selected(matches!(*selected_mode, SelectedMode::Stations))
                        .min_size(egui::vec2(MIN_X, MIN_Y)),
                )
                .clicked()
            {
                println!("Stations");
                *selected_mode = SelectedMode::Stations;
                ui.close_menu();
            }

            production_menu(&mut selected_mode, ui);

            menu::menu_button(ui, "⚔ Military", |ui| {
                if ui
                    .add(
                        egui::Button::new("⚔ Fixed Artillery")
                            .selected(matches!(*selected_mode, SelectedMode::Military))
                            .min_size(egui::vec2(MIN_X, MIN_Y)),
                    )
                    .clicked()
                {
                    println!("Fixed Arty");
                    *selected_mode = SelectedMode::Military;
                    ui.close_menu();
                }
            });
            menu::menu_button(ui, "🚆 Trains", |ui| {
                if ui
                    .add(
                        egui::Button::new("🚆 Coal Train")
                            .selected(matches!(*selected_mode, SelectedMode::Trains))
                            .min_size(egui::vec2(MIN_X, MIN_Y)),
                    )
                    .clicked()
                {
                    println!("Coal Train");
                    *selected_mode = SelectedMode::Trains;
                    ui.close_menu();
                }
            });
            if ui
                .add(
                    egui::Button::new("❎ Demolish")
                        .selected(matches!(*selected_mode, SelectedMode::Demolish))
                        .min_size(egui::vec2(MIN_X, MIN_Y)),
                )
                .clicked()
            {
                println!("Demolish");
                *selected_mode = SelectedMode::Demolish;
                ui.close_menu();
            }
        });
    });
}

fn production_menu(selected_mode: &mut SelectedMode, ui: &mut Ui) {
    menu::menu_button(ui, "⚒ Production", |ui| {
        if ui
            .add(
                egui::Button::new("⛏ Iron Mine")
                    .selected(matches!(
                        *selected_mode,
                        SelectedMode::Production(ProductionType::IronMine)
                    ))
                    .min_size(egui::vec2(MIN_X, MIN_Y)),
            )
            .clicked()
        {
            println!("Iron Mine");
            *selected_mode = SelectedMode::Production(ProductionType::IronMine);
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new("⛏ Coal Mine")
                    .selected(matches!(
                        *selected_mode,
                        SelectedMode::Production(ProductionType::CoalMine)
                    ))
                    .min_size(egui::vec2(MIN_X, MIN_Y)),
            )
            .clicked()
        {
            println!("Coal Mine");
            *selected_mode = SelectedMode::Production(ProductionType::CoalMine);
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new("⚒ Iron Works")
                    .selected(matches!(
                        *selected_mode,
                        SelectedMode::Production(ProductionType::IronWorks)
                    ))
                    .min_size(egui::vec2(MIN_X, MIN_Y)),
            )
            .clicked()
        {
            println!("Iron Works");
            *selected_mode = SelectedMode::Production(ProductionType::IronWorks);
            ui.close_menu();
        }
        if ui
            .add(
                egui::Button::new("⚓ Cargo Port")
                    .selected(matches!(
                        *selected_mode,
                        SelectedMode::Production(ProductionType::CargoPort)
                    ))
                    .min_size(egui::vec2(MIN_X, MIN_Y)),
            )
            .clicked()
        {
            println!("Cargo Port");
            *selected_mode = SelectedMode::Production(ProductionType::CargoPort);
            ui.close_menu();
        }
    });
}
