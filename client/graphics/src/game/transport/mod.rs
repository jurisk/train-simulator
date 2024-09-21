pub mod assets;
pub mod building;
mod train;
pub mod train_layout;
pub mod ui;

use bevy::app::App;
use bevy::asset::Assets;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{
    Children, Commands, Component, Entity, EventReader, FixedUpdate, IntoSystemConfigs, Plugin,
    Query, Res, ResMut, SpatialBundle, Transform, Update, in_state, warn,
};
use shared_domain::TransportId;
use shared_domain::game_state::GameState;
use shared_domain::map_level::map_level::MapLevel;
use shared_domain::players::player_state::PlayerState;
use shared_domain::server_response::{GameResponse, ServerResponse};
use shared_domain::transport::transport_info::TransportInfo;
use shared_domain::transport::transport_type::TransportType;

use crate::assets::GameAssets;
use crate::communication::domain::ServerMessageEvent;
use crate::game::transport::assets::TransportAssets;
use crate::game::transport::building::build_transport_when_mouse_released;
use crate::game::transport::train::{calculate_train_component_transforms, create_train};
use crate::game::transport::ui::{
    TransportsToShow, select_station_to_add_to_movement_orders, show_transport_details,
};
use crate::game::{GameStateResource, player_colour};
use crate::states::ClientState;

#[derive(Component)]
pub struct TransportIdComponent(pub TransportId);

#[derive(Component)]
pub struct TransportIndexComponent(pub usize);

pub struct TransportPlugin;

impl Plugin for TransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, handle_game_state_snapshot);
        app.add_systems(
            FixedUpdate,
            handle_transport_created.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            handle_transports_sync.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            Update,
            move_transports.run_if(in_state(ClientState::Playing)),
        );

        app.add_systems(
            Update,
            build_transport_when_mouse_released.run_if(in_state(ClientState::Playing)),
        );

        app.insert_resource(TransportsToShow::default());

        app.add_systems(
            Update,
            show_transport_details.run_if(in_state(ClientState::Playing)),
        );

        app.add_systems(
            Update,
            select_station_to_add_to_movement_orders.run_if(in_state(ClientState::Playing)),
        );
    }
}

#[expect(clippy::needless_pass_by_value)]
fn move_transports(
    mut query: Query<(&TransportIdComponent, &Children)>,
    mut child_query: Query<(&mut Transform, &TransportIndexComponent)>,
    game_state_resource: Res<GameStateResource>,
) {
    let GameStateResource(game_state) = game_state_resource.as_ref();
    let map_level = game_state.map_level();
    for (transport_id_component, children) in &mut query {
        let TransportIdComponent(transport_id) = transport_id_component;
        if let Some(transport_info) = game_state.get_transport_info(*transport_id) {
            let transforms = match &transport_info.transport_type() {
                TransportType::Train(components) => {
                    calculate_train_component_transforms(
                        components,
                        transport_info.location(),
                        map_level,
                    )
                },
                TransportType::RoadVehicle(_) | TransportType::Ship(_) => todo!(), /* TODO: Also handle others! */
            };

            for &child in children {
                if let Ok((mut child_transform, transport_index_component)) =
                    child_query.get_mut(child)
                {
                    let TransportIndexComponent(transport_index) = transport_index_component;
                    let new_transform: Transform = transforms[*transport_index];
                    child_transform.translation = new_transform.translation;
                    child_transform.rotation = new_transform.rotation;
                }
            }
        } else {
            warn!("Transport {:?} not found", transport_id);
        }
    }
}

#[expect(clippy::collapsible_match)]
fn handle_transports_sync(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut game_state_resource: ResMut<GameStateResource>,
) {
    let GameStateResource(game_state) = game_state_resource.as_mut();
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, game_response) = &message.response {
            if let GameResponse::DynamicInfosSync(
                game_time,
                industry_building_infos,
                station_building_infos,
                transport_infos,
            ) = game_response
            {
                game_state.update_dynamic_infos(
                    *game_time,
                    industry_building_infos,
                    station_building_infos,
                    transport_infos,
                );
            }
        }
    }
}

#[expect(
    clippy::collapsible_match,
    clippy::single_match,
    clippy::needless_pass_by_value
)]
fn handle_game_state_snapshot(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, game_response) = &message.response {
            match game_response {
                GameResponse::GameJoined(_player_id, game_state) => {
                    for transport_info in game_state.transport_infos() {
                        create_transport(
                            transport_info,
                            &mut commands,
                            &game_assets,
                            &mut materials,
                            game_state,
                        );
                    }
                },
                _ => {},
            }
        }
    }
}

#[expect(
    clippy::collapsible_match,
    clippy::needless_pass_by_value,
    clippy::single_match
)]
fn handle_transport_created(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_state_resource: ResMut<GameStateResource>,
) {
    let GameStateResource(game_state) = game_state_resource.as_mut();
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, game_response) = &message.response {
            match game_response {
                GameResponse::TransportsAdded(transport_infos) => {
                    for transport_info in transport_infos {
                        game_state.upsert_transport(transport_info.clone());

                        create_transport(
                            transport_info,
                            &mut commands,
                            &game_assets,
                            &mut materials,
                            game_state,
                        );
                    }
                },
                _ => {},
            }
        }
    }
}

fn create_transport(
    transport_info: &TransportInfo,
    commands: &mut Commands,
    game_assets: &GameAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_state: &GameState,
) {
    let entity = create_transport_internal(
        transport_info,
        commands,
        &game_assets.transport_assets,
        materials,
        game_state.map_level(),
        game_state.players(),
    );

    if let Some(entity) = entity {
        commands
            .entity(entity)
            .insert(SpatialBundle::default()) // For https://bevyengine.org/learn/errors/b0004/
            .insert(TransportIdComponent(transport_info.transport_id()));
    }
}

#[must_use]
fn create_transport_internal(
    transport_info: &TransportInfo,
    commands: &mut Commands,
    transport_assets: &TransportAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
    players: &PlayerState,
) -> Option<Entity> {
    let colour = player_colour(players, transport_info.owner_id());
    match &transport_info.transport_type() {
        TransportType::Train(train_components) => {
            Some(create_train(
                transport_info.transport_id(),
                colour,
                transport_info.location(),
                train_components,
                commands,
                transport_assets,
                materials,
                map_level,
            ))
        },
        TransportType::RoadVehicle(_) | TransportType::Ship(_) => {
            None // TODO: Implement
        },
    }
}
