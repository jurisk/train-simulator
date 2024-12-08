pub(crate) mod assets;

use bevy::app::Update;
use bevy::log::debug;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::{
    App, Commands, Component, Entity, EventReader, FixedUpdate, IntoSystemConfigs, Mesh3d, Plugin,
    Query, Res, ResMut, Transform, in_state,
};
use shared_domain::ProjectileId;
use shared_domain::military::ProjectileType;
use shared_domain::military::projectile_info::ProjectileInfo;
use shared_domain::server_response::{GameResponse, ServerResponse};

use crate::assets::GameAssets;
use crate::communication::domain::ServerMessageEvent;
use crate::game::GameStateResource;
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
            Update,
            sync_projectiles_with_state.run_if(in_state(ClientState::Playing)),
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

#[expect(clippy::match_same_arms)]
fn handle_projectile_added_or_removed(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut game_state_resource: ResMut<GameStateResource>,
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
                    // The tricky part is that we can receive the same projectile multiple times - once from the client side game state, once from the server side game state...
                    // Is there a better way? Not sure.

                    for projectile in projectiles {
                        game_state.upsert_projectile(projectile.clone());
                    }
                },
                GameResponse::ProjectilesRemoved(projectile_ids) => {
                    for projectile_id in projectile_ids {
                        game_state.remove_projectile(*projectile_id);
                    }
                },
                GameResponse::DynamicInfosSync(..) => {},
                GameResponse::GameJoined(_player_id, _game_state) => {},
                GameResponse::GameLeft => {},
                GameResponse::Error(_) => {},
            }
        }
    }
}

fn ensure_projectile_entity_exists(
    commands: &mut Commands,
    game_assets: &GameAssets,
    projectile: &ProjectileInfo,
    query: &Query<(Entity, &ProjectileIdComponent)>,
) {
    if query
        .iter()
        .all(|(_, ProjectileIdComponent(projectile_id))| {
            *projectile_id != projectile.projectile_id()
        })
    {
        create_projectile_entity(commands, game_assets, projectile);
    }
}

#[expect(clippy::needless_pass_by_value)]
fn sync_projectiles_with_state(
    game_assets: Res<GameAssets>,
    game_state_resource: Res<GameStateResource>,
    mut commands: Commands,
    projectile_id_query: Query<(Entity, &ProjectileIdComponent)>,
) {
    // Later: Would using set subtraction be more efficient? Anyway, it can wait.

    let GameStateResource(game_state) = game_state_resource.as_ref();

    for (entity, projectile_id_component) in &projectile_id_query {
        let ProjectileIdComponent(found_projectile_id) = projectile_id_component;
        if !game_state
            .projectile_state()
            .has_projectile(*found_projectile_id)
        {
            commands.entity(entity).despawn();
        }
    }

    for projectile in game_state.projectile_infos() {
        ensure_projectile_entity_exists(
            &mut commands,
            game_assets.as_ref(),
            projectile,
            &projectile_id_query,
        );
    }
}

fn create_projectile_entity(
    commands: &mut Commands,
    game_assets: &GameAssets,
    projectile: &ProjectileInfo,
) {
    let transform = calculate_transform(projectile);

    debug!("Spawning a shell at {transform:?}...");

    let shell = game_assets
        .military_assets
        .shells
        .mesh_and_material_for_shell_type(ProjectileType::Standard);

    commands.spawn((
        Mesh3d(shell.mesh.clone()),
        MeshMaterial3d(shell.material.clone()),
        transform,
        ProjectileIdComponent(projectile.projectile_id()),
    ));
}
