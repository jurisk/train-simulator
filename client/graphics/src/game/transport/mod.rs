mod train;
pub mod train_layout;

use std::collections::HashMap;

use bevy::app::App;
use bevy::asset::Assets;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{
    error, in_state, warn, Children, Commands, Component, Entity, EventReader, FixedUpdate,
    IntoSystemConfigs, Mesh, Plugin, Query, Res, ResMut, SpatialBundle, Transform, Update,
};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{GameResponse, PlayerInfo, ServerResponse};
use shared_domain::transport_info::TransportInfo;
use shared_domain::transport_type::TransportType;
use shared_domain::{PlayerId, TransportId};

use crate::communication::domain::ServerMessageEvent;
use crate::game::transport::train::{calculate_train_component_transforms, create_train};
use crate::game::GameStateResource;
use crate::states::ClientState;

#[derive(Component)]
pub struct TransportIdComponent(pub TransportId);

#[derive(Component)]
pub struct TransportIndexComponent(pub usize);

pub struct TransportPlugin;

impl Plugin for TransportPlugin {
    fn build(&self, app: &mut App) {
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
        // TODO HIGH: Spawn trains when in Train building mode
        // TODO HIGH: When a train is clicked, allow to adjust MovementOrders
    }
}

#[allow(clippy::needless_pass_by_value)]
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

#[allow(clippy::collapsible_match)]
fn handle_transports_sync(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut game_state_resource: ResMut<GameStateResource>,
) {
    let GameStateResource(game_state) = game_state_resource.as_mut();
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, game_response) = &message.response {
            if let GameResponse::TransportsSync(game_time, transport_infos) = game_response {
                game_state.update_transport_dynamic_infos(*game_time, transport_infos);
            }
        }
    }
}

#[allow(clippy::collapsible_match, clippy::needless_pass_by_value)]
fn handle_transport_created(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_state_resource: ResMut<GameStateResource>,
) {
    let GameStateResource(game_state) = game_state_resource.as_mut();
    let map_level = game_state.map_level().clone();
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, game_response) = &message.response {
            if let GameResponse::TransportsAdded(transport_infos) = game_response {
                for transport_info in transport_infos {
                    game_state.upsert_transport(transport_info.clone());

                    let entity = create_transport(
                        transport_info,
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &map_level,
                        game_state.players(),
                    );

                    if let Some(entity) = entity {
                        commands
                            .entity(entity)
                            .insert(SpatialBundle::default()) // For https://bevyengine.org/learn/errors/b0004/
                            .insert(TransportIdComponent(transport_info.transport_id()));
                    }
                }
            }
        }
    }
}

#[allow(clippy::similar_names)]
#[must_use]
fn create_transport(
    transport_info: &TransportInfo,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
    players_info: &HashMap<PlayerId, PlayerInfo>,
) -> Option<Entity> {
    match players_info.get(&transport_info.owner_id()) {
        None => {
            error!("Player with ID {:?} not found", transport_info.owner_id());
            None
        },
        Some(player_info) => {
            match &transport_info.transport_type() {
                TransportType::Train(train_components) => {
                    Some(create_train(
                        transport_info.transport_id(),
                        player_info,
                        transport_info.location(),
                        train_components,
                        commands,
                        meshes,
                        materials,
                        map_level,
                    ))
                },
                TransportType::RoadVehicle(_) | TransportType::Ship(_) => {
                    None // TODO: Implement
                },
            }
        },
    }
}
