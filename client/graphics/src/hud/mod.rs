use bevy::app::{App, Plugin};
use bevy::prelude::{Resource, Update};
use bevy_egui::EguiContexts;
use egui;
use egui::menu;
use shared_domain::production_type::ProductionType;
use shared_domain::station_type::StationOrientation;

#[derive(Resource)]
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

fn show_hud(mut contexts: EguiContexts) {
    egui::TopBottomPanel::top("Current Mode").show(contexts.ctx_mut(), |ui| {
        menu::bar(ui, |ui| {
            if ui.add(egui::Button::new("ℹ Info").selected(true)).clicked() {
                println!("Info");
                ui.close_menu();
            }
            if ui.button("🚆 Tracks").clicked() {
                println!("Tracks");
                ui.close_menu();
            }
            menu::menu_button(ui, "🚉 Stations", |ui| {
                if ui.button("↕ North to South").clicked() {
                    println!("North to South");
                    ui.close_menu();
                }
                if ui.button("↔ East to West").clicked() {
                    println!("East to West");
                    ui.close_menu();
                }
            });
            menu::menu_button(ui, "⚒ Production", |ui| {
                if ui.button("⛏ Iron Mine").clicked() {
                    println!("Iron Mine");
                    ui.close_menu();
                }
                if ui.button("⛏ Coal Mine").clicked() {
                    println!("Coal Mine");
                    ui.close_menu();
                }
                if ui.button("⚒ Iron Works").clicked() {
                    println!("Iron Works");
                    ui.close_menu();
                }
                if ui.button("⚓ Cargo Port").clicked() {
                    println!("Cargo Port");
                    ui.close_menu();
                }
            });
            menu::menu_button(ui, "⚔ Military", |ui| {
                if ui.button("⚔ Fixed Artillery").clicked() {
                    println!("Fixed Artillery");
                    ui.close_menu();
                }
                if ui.button("⚔ Mobile Artillery").clicked() {
                    println!("Mobile Artillery");
                    ui.close_menu();
                }
            });
            menu::menu_button(ui, "🚆 Trains", |ui| {
                if ui.button("🚆 Coal Train length 4").clicked() {
                    println!("Coal Train");
                    ui.close_menu();
                }
            });
            if ui.button("❎ Demolish").clicked() {
                println!("Demolish");
                ui.close_menu();
            }
        });
    });
}
