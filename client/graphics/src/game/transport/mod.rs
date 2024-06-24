mod train;
pub mod train_layout;

use std::collections::HashMap;

use bevy::app::App;
use bevy::asset::Assets;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{
    error, Children, Commands, Component, Entity, EventReader, FixedUpdate, Mesh, Plugin, Query,
    Res, ResMut, SpatialBundle, Time, Transform, Update,
};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{GameResponse, PlayerInfo, ServerResponse};
use shared_domain::{PlayerId, TransportInfo, TransportType};

use crate::communication::domain::ServerMessageEvent;
use crate::game::transport::train::{calculate_train_component_transforms, create_train};
use crate::game::{GameStateResource, PlayersInfoResource};

// TODO HIGH: Consider keeping this in `GameStateResource` sub-component, and only keep the `TransportId` as a component to associate the Bevy entity with a `TransportId`
#[derive(Component)]
pub struct TransportInfoComponent(pub TransportInfo);

#[derive(Component)]
pub struct TransportIndexComponent(pub usize);

pub struct TransportPlugin;

impl Plugin for TransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, handle_transport_created);
        app.add_systems(FixedUpdate, handle_transports_sync);
        app.add_systems(Update, move_transports);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn move_transports(
    time: Res<Time>,
    mut query: Query<(&mut TransportInfoComponent, &Children)>,
    mut child_query: Query<(&mut Transform, &TransportIndexComponent)>,
    game_state: Option<Res<GameStateResource>>,
) {
    if let Some(game_state) = game_state {
        let GameStateResource(game_state) = game_state.as_ref();
        let map_level = game_state.map_level();
        for (mut transport_info_component, children) in &mut query {
            let TransportInfoComponent(ref mut transport_info) = transport_info_component.as_mut();
            // TODO HIGH:   Instead of advancing each transport individually on the client, we should use the same `GameState` `advance_time` and then
            //              here just update the transforms of the transports.
            transport_info.advance(time.delta_seconds(), game_state.building_state());

            let transforms = match &transport_info.transport_type() {
                TransportType::Train(components) => {
                    calculate_train_component_transforms(
                        components,
                        transport_info.location(),
                        map_level,
                    )
                },
                TransportType::RoadVehicle | TransportType::Ship => todo!(), /* TODO: Also handle others! */
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
        }
    }
}

#[allow(clippy::collapsible_match)]
fn handle_transports_sync(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut query: Query<&mut TransportInfoComponent>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, game_response) = &message.response {
            if let GameResponse::TransportsSync(transports_sync) = game_response {
                for (transport_id, transport_dynamic_info) in transports_sync {
                    for mut transport_info_component in &mut query {
                        let TransportInfoComponent(transport_info) =
                            transport_info_component.as_mut();
                        if transport_info.transport_id() == *transport_id {
                            transport_info.update_dynamic_info(transport_dynamic_info);
                        }
                    }
                }
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
    game_state: Option<Res<GameStateResource>>,
    players_info_resource: Res<PlayersInfoResource>,
) {
    let PlayersInfoResource(players_info) = players_info_resource.as_ref();

    if let Some(game_state) = game_state {
        let GameStateResource(game_state) = game_state.as_ref();
        let map_level = game_state.map_level();
        for message in server_messages.read() {
            if let ServerResponse::Game(_game_id, game_response) = &message.response {
                if let GameResponse::TransportsExist(transport_infos) = game_response {
                    for transport_info in transport_infos {
                        let entity = create_transport(
                            transport_info,
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            map_level,
                            players_info,
                        );

                        if let Some(entity) = entity {
                            commands
                                .entity(entity)
                                .insert(SpatialBundle::default()) // For https://bevyengine.org/learn/errors/b0004/
                                .insert(TransportInfoComponent(transport_info.clone()));
                        }
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
                TransportType::RoadVehicle | TransportType::Ship => {
                    None // TODO: Implement
                },
            }
        },
    }
}
