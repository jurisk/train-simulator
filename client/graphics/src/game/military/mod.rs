pub(crate) mod assets;

use bevy::app::App;
use bevy::input::ButtonInput;
use bevy::log::debug;
use bevy::math::{Quat, Vec3};
use bevy::prelude::{
    Commands, IntoSystemConfigs, KeyCode, PbrBundle, Plugin, Res, Transform, Update, default,
    in_state,
};
use shared_domain::game_state::GameState;
use shared_domain::military::ShellType;
use shared_domain::tile_coords_xz::TileCoordsXZ;

use crate::assets::GameAssets;
use crate::game::GameStateResource;
use crate::game::military::assets::MilitaryAssets;
use crate::states::ClientState;

pub struct MilitaryPlugin;

impl Plugin for MilitaryPlugin {
    fn build(&self, app: &mut App) {
        // TODO HIGH: Actually add events for spawning shells, and spawn them, and animate them. But how to handle impacts & explosions?
        app.add_systems(
            Update,
            test_shell_when_keyboard_pressed.run_if(in_state(ClientState::Playing)),
        );
    }
}

#[expect(clippy::needless_pass_by_value)]
fn test_shell_when_keyboard_pressed(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    game_state: Res<GameStateResource>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.pressed(KeyCode::Space) {
        let GameStateResource(game_state) = game_state.as_ref();
        create_shell_entity(
            &mut commands,
            game_assets.as_ref(),
            game_state,
            TileCoordsXZ::new(100, 100),
        );
    }
}

fn create_shell_entity(
    commands: &mut Commands,
    game_assets: &GameAssets,
    game_state: &GameState,
    tile: TileCoordsXZ,
) {
    // TODO HIGH: Do something better here

    let mut position = game_state
        .map_level()
        .terrain()
        .tile_center_coordinate(tile);
    position.y += 2.0;

    let velocity = Vec3::new(1.0, 1.0, 1.0);
    let rotation = Quat::from_rotation_arc(Vec3::Y, velocity);

    let pbr_bundle = create_shell_pbr_bundle(
        ShellType::Standard,
        position,
        rotation,
        &game_assets.military_assets,
    );
    commands.spawn(pbr_bundle);
}

fn create_shell_pbr_bundle(
    shell_type: ShellType,
    position: Vec3,
    rotation: Quat,
    military_assets: &MilitaryAssets,
) -> PbrBundle {
    debug!("Spawning a shell at {position} with rotation {rotation}...");

    let shell = military_assets
        .shells
        .mesh_and_material_for_shell_type(shell_type);

    PbrBundle {
        mesh: shell.mesh.clone(),
        material: shell.material.clone(),
        transform: Transform::from_xyz(position.x, position.y, position.z).with_rotation(rotation),
        ..default()
    }
}
