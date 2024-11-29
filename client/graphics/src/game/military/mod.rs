pub(crate) mod assets;

use bevy::app::Update;
use bevy::log::debug;
use bevy::prelude::{
    App, Commands, Component, Entity, EventReader, FixedUpdate, IntoSystemConfigs, PbrBundle,
    Plugin, Query, Res, ResMut, Transform, default, in_state,
};
use shared_domain::ProjectileId;
use shared_domain::military::ProjectileType;
use shared_domain::military::projectile_info::ProjectileInfo;
use shared_domain::server_response::{GameResponse, ServerResponse};

use crate::assets::GameAssets;
use crate::communication::domain::ServerMessageEvent;
use crate::game::GameStateResource;
use crate::game::military::assets::MilitaryAssets;
use crate::states::ClientState;
use crate::util::transform_from_midpoint_and_direction_yz;

#[derive(Component)]
pub struct ProjectileIdComponent(ProjectileId);

pub struct MilitaryPlugin;

impl Plugin for MilitaryPlugin {
    fn build(&self, app: &mut App) {
        // TODO HIGH: Handle shell impacts & explosions.

        app.add_systems(
            Update,
            move_projectiles.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            handle_projectile_added_or_removed.run_if(in_state(ClientState::Playing)),
        );
    }
}

#[expect(clippy::needless_pass_by_value)]
fn move_projectiles(
    game_state_resource: Res<GameStateResource>,
    mut query: Query<(&mut Transform, &ProjectileIdComponent)>,
) {
    let GameStateResource(game_state) = game_state_resource.as_ref();
    for (mut transform, projectile_id_component) in &mut query {
        let ProjectileIdComponent(projectile_id) = projectile_id_component;
        if let Some(projectile) = game_state
            .projectile_state()
            .find_projectile(*projectile_id)
        {
            *transform = calculate_transform(projectile);
        }
    }
}

fn calculate_transform(projectile: &ProjectileInfo) -> Transform {
    transform_from_midpoint_and_direction_yz(
        projectile.location().into(),
        projectile.velocity().into(),
    )
}

#[expect(clippy::match_same_arms, clippy::needless_pass_by_value)]
fn handle_projectile_added_or_removed(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut game_state_resource: ResMut<GameStateResource>,
    projectile_id_query: Query<(Entity, &ProjectileIdComponent)>,
) {
    let GameStateResource(game_state) = game_state_resource.as_mut();
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, game_response) = &message.response {
            match game_response {
                GameResponse::GameStateSnapshot(_) => {},
                GameResponse::PlayersUpdated(_) => {},
                GameResponse::IndustryBuildingAdded(_) => {},
                GameResponse::IndustryBuildingRemoved(_) => {},
                GameResponse::MilitaryBuildingAdded(_) => {},
                GameResponse::MilitaryBuildingRemoved(_) => {},
                GameResponse::StationAdded(_) => {},
                GameResponse::StationRemoved(_) => {},
                GameResponse::TracksAdded(_) => {},
                GameResponse::TracksRemoved(_) => {},
                GameResponse::TransportsAdded(_) => {},
                GameResponse::ProjectilesAdded(projectiles) => {
                    for projectile in projectiles {
                        game_state.upsert_projectile(projectile.clone());

                        create_projectile_entity(&mut commands, game_assets.as_ref(), projectile);
                    }
                },
                GameResponse::ProjectilesRemoved(projectile_ids) => {
                    for projectile_id in projectile_ids {
                        game_state.remove_projectile(*projectile_id);
                    }

                    remove_projectile_entities(projectile_ids, &mut commands, &projectile_id_query);
                },
                GameResponse::DynamicInfosSync(..) => {},
                GameResponse::GameJoined(..) => {},
                GameResponse::GameLeft => {},
                GameResponse::Error(_) => {},
            }
        }
    }
}

fn remove_projectile_entities(
    projectile_ids: &[ProjectileId],
    commands: &mut Commands,
    query: &Query<(Entity, &ProjectileIdComponent)>,
) {
    for (entity, projectile_id_component) in query {
        let ProjectileIdComponent(found_projectile_id) = projectile_id_component;
        if projectile_ids.contains(found_projectile_id) {
            commands.entity(entity).despawn();
        }
    }
}

fn create_projectile_entity(
    commands: &mut Commands,
    game_assets: &GameAssets,
    projectile: &ProjectileInfo,
) {
    let transform = calculate_transform(projectile);
    let pbr_bundle = create_shell_pbr_bundle(
        ProjectileType::Standard,
        transform,
        &game_assets.military_assets,
    );

    commands
        .spawn(pbr_bundle)
        .insert(ProjectileIdComponent(projectile.projectile_id()));
}

fn create_shell_pbr_bundle(
    shell_type: ProjectileType,
    transform: Transform,
    military_assets: &MilitaryAssets,
) -> PbrBundle {
    debug!("Spawning a shell at {transform:?}...");

    let shell = military_assets
        .shells
        .mesh_and_material_for_shell_type(shell_type);

    PbrBundle {
        mesh: shell.mesh.clone(),
        material: shell.material.clone(),
        transform,
        ..default()
    }
}
