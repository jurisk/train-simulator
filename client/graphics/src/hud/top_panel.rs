use bevy::prelude::{AppExit, EventWriter, Res, ResMut};
use bevy_egui::EguiContexts;
use egui::{Ui, menu};
use shared_domain::building::industry_type::IndustryType;
use shared_domain::building::military_building_type::MilitaryBuildingType;
use shared_domain::building::station_type::{StationOrientation, StationType};
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::game_state::GameState;
use shared_domain::game_time::TimeFactor;
use shared_domain::resource_type::ResourceType;
use shared_domain::transport::transport_type::TransportType;

use crate::ai::ArtificialIntelligenceResource;
use crate::communication::domain::ClientMessageEvent;
use crate::game::GameStateResource;
use crate::hud::PointerOverHud;
use crate::hud::domain::{DemolishType, SelectedMode, TracksBuildingType};

const MIN_X: f32 = 200.0;
const MIN_Y: f32 = 40.0;

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn show_top_panel(
    mut contexts: EguiContexts,
    game_state: Res<GameStateResource>,
    mut selected_mode: ResMut<SelectedMode>,
    mut ai_resource: ResMut<ArtificialIntelligenceResource>,
    mut pointer_over_hud: ResMut<PointerOverHud>,
    mut exit: EventWriter<AppExit>,
    mut client_messages: EventWriter<ClientMessageEvent>,
) {
    // Later: We need to better depict the current building mode in the main menu, in case it's a sub-menu item that is selected
    let GameStateResource(game_state) = game_state.as_ref();

    egui::TopBottomPanel::top("hud_top_panel").show(contexts.ctx_mut(), |ui| {
        // TODO HIGH: This is insufficient, as when the menu is open, it is not considered as being part of HUD. Need to pass these down to the submenus.
        pointer_over_hud.apply(ui);
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
            ai_menu(&mut ai_resource, game_state, ui);
            actions_menu(&mut exit, game_state, &mut client_messages, ui);
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
                .selected(matches!(*selected_mode.as_ref(), SelectedMode::Tracks(_)))
                .min_size(egui::vec2(MIN_X, MIN_Y)),
        )
        .clicked()
    {
        *selected_mode.as_mut() = SelectedMode::Tracks(TracksBuildingType::SelectStart);
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
                StationOrientation::WestToEast => "⬌ EW",
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
                IndustryType::CellulosePlant => "🪵",
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

fn military_menu(selected_mode: &mut ResMut<SelectedMode>, ui: &mut Ui) {
    menu::menu_button(ui, "⚔ Military", |ui| {
        set_font_size(ui, 24.0);

        for military_building_type in MilitaryBuildingType::all() {
            if ui
                .add(
                    egui::Button::new(format!("⚔ {military_building_type:?}"))
                        .selected(
                            *selected_mode.as_ref()
                                == SelectedMode::MilitaryBuilding(military_building_type),
                        )
                        .min_size(egui::vec2(MIN_X, MIN_Y)),
                )
                .clicked()
            {
                *selected_mode.as_mut() = SelectedMode::MilitaryBuilding(military_building_type);
                ui.close_menu();
            }
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
            (
                "⚔ Military Building",
                SelectedMode::Demolish(DemolishType::MilitaryBuilding),
            ),
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

// TODO HIGH: Have an option where AI is enabled on startup already - could be useful for profiling and testing. Consider adding auto-exit on some end conditions as well, as a launch option. That would give you an integration test with UI enabled.
fn ai_menu(
    ai_resource: &mut ResMut<ArtificialIntelligenceResource>,
    game_state: &GameState,
    ui: &mut Ui,
) {
    // Later: Could disable the currently selected AI mode, but that does not matter much
    menu::menu_button(ui, "🖥 AI", |ui| {
        set_font_size(ui, 24.0);

        for player in game_state.players().infos() {
            let player_id = player.id;
            let player_name = format!("{}", player.name);

            if ui
                .add(
                    egui::Button::new(format!("❎ Disable for {player_name}"))
                        .min_size(egui::vec2(MIN_X, MIN_Y)),
                )
                .clicked()
            {
                ai_resource.as_mut().disable(player_id);
                ui.close_menu();
            }

            for (name, seconds) in [
                ("☑ Enable 100 milliseconds", 0.1),
                ("☑ Enable 1 second", 1.0),
                ("☑ Enable 10 seconds", 10.0),
            ] {
                if ui
                    .add(
                        egui::Button::new(format!("{name} for {player_name}"))
                            .min_size(egui::vec2(MIN_X, MIN_Y)),
                    )
                    .clicked()
                {
                    ai_resource.as_mut().enable(player_id, seconds, game_state);
                    ui.close_menu();
                }
            }
        }
    });
}

fn actions_menu(
    exit: &mut EventWriter<AppExit>,
    game_state: &GameState,
    client_messages: &mut EventWriter<ClientMessageEvent>,
    ui: &mut Ui,
) {
    menu::menu_button(ui, "Actions", |ui| {
        set_font_size(ui, 24.0);
        ui.menu_button("Game Speed", |ui| {
            for (name, speed) in [
                ("Pause", 0.0),
                ("¼×", 0.25),
                ("½×", 0.5),
                ("Normal", 1.0),
                ("2×", 2.0),
                ("4×", 4.0),
                ("8×", 8.0),
                ("16×", 16.0),
            ] {
                let time_factor = TimeFactor::new(speed);
                if ui
                    .add(
                        egui::Button::new(name)
                            .selected(game_state.time_factor() == time_factor)
                            .min_size(egui::vec2(MIN_X, MIN_Y)),
                    )
                    .clicked()
                {
                    client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                        game_state.game_id,
                        GameCommand::SetTimeFactor(time_factor),
                    )));
                    ui.close_menu();
                }
            }
        });

        if ui
            .add(egui::Button::new("❎ Exit").min_size(egui::vec2(MIN_X, MIN_Y)))
            .clicked()
        {
            exit.send(AppExit::Success);
        }
    });
}
