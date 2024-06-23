mod train;

use std::collections::HashMap;

use bevy::app::App;
use bevy::asset::Assets;
use bevy::log::error;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{
    Children, Commands, Component, Entity, EventReader, FixedUpdate, Mesh, Plugin, Query, Res,
    ResMut, SpatialBundle, Time, Transform, Update,
};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{GameResponse, PlayerInfo, ServerResponse};
use shared_domain::{PlayerId, TransportInfo, TransportType};

use crate::communication::domain::ServerMessageEvent;
use crate::game::buildings::BuildingStateResource;
use crate::game::map_level::MapLevelResource;
use crate::game::transport::train::{calculate_train_component_transforms, create_train};
use crate::game::PlayersInfoResource;

#[derive(Component)]
pub struct TransportInfoComponent(pub TransportInfo);

#[derive(Component)]
pub struct TransportIndexComponent(pub usize);

pub struct TransportPlugin;

impl Plugin for TransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, handle_transport_created);
        app.add_systems(Update, move_transports);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn move_transports(
    time: Res<Time>,
    mut query: Query<(&mut TransportInfoComponent, &Children)>,
    mut child_query: Query<(&mut Transform, &TransportIndexComponent)>,
    map_level: Option<Res<MapLevelResource>>,
    building_state_resource: Res<BuildingStateResource>,
) {
    let BuildingStateResource(building_state) = building_state_resource.as_ref();
    if let Some(map_level) = map_level {
        for (mut transport_info_component, children) in &mut query {
            let TransportInfoComponent(ref mut transport_info) = transport_info_component.as_mut();
            transport_info.advance(time.delta_seconds(), building_state);

            let transforms = match &transport_info.transport_type {
                TransportType::Train(components) => {
                    calculate_train_component_transforms(
                        components,
                        &transport_info.location,
                        &map_level.map_level,
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

#[allow(clippy::collapsible_match, clippy::needless_pass_by_value)]
fn handle_transport_created(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_level: Option<Res<MapLevelResource>>,
    players_info_resource: Res<PlayersInfoResource>,
) {
    let PlayersInfoResource(players_info) = players_info_resource.as_ref();

    if let Some(map_level) = map_level {
        for message in server_messages.read() {
            if let ServerResponse::Game(_game_id, game_response) = &message.response {
                if let GameResponse::TransportCreated(transport_info) = game_response {
                    let entity = create_transport(
                        transport_info,
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &map_level.map_level,
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
    match players_info.get(&transport_info.owner_id) {
        None => {
            error!("Player with ID {:?} not found", transport_info.owner_id);
            None
        },
        Some(player_info) => {
            match &transport_info.transport_type {
                TransportType::Train(train_components) => {
                    Some(create_train(
                        transport_info.transport_id,
                        player_info,
                        &transport_info.location,
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
