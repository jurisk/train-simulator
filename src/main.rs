use bevy::a11y::AccessibilityPlugin;
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::core::{FrameCountPlugin, TaskPoolPlugin, TypeRegistrationPlugin};
use bevy::core_pipeline::CorePipelinePlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::gizmos::GizmoPlugin;
use bevy::input::InputPlugin;
use bevy::log::LogPlugin;
use bevy::pbr::PbrPlugin;
use bevy::prelude::{App, HierarchyPlugin, ImagePlugin, TransformPlugin};
use bevy::render::RenderPlugin;
use bevy::sprite::SpritePlugin;
use bevy::text::TextPlugin;
use bevy::time::TimePlugin;
use bevy::ui::UiPlugin;
use bevy::utils::default;
use bevy::window::{Window, WindowPlugin, WindowResolution};
use bevy::winit::WinitPlugin;

use crate::cameras::CameraPlugin;
use crate::communication::CommunicationPlugin;
use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::debug::DebugPlugin;
use crate::level::LevelPlugin;
use crate::lights::LightsPlugin;
use crate::states::GameState;

mod cameras;
mod communication;
mod constants;
mod debug;
mod level;
mod lights;
mod states;

fn main() {
    App::new()
        .init_state::<GameState>()
        .add_plugins((
            LogPlugin::default(),
            TaskPoolPlugin::default(),
            TypeRegistrationPlugin,
            FrameCountPlugin,
            TimePlugin,
            TransformPlugin,
            HierarchyPlugin,
            DiagnosticsPlugin,
            InputPlugin,
        ))
        .add_plugins((
            WindowPlugin {
                primary_window: Some(Window {
                    #[allow(clippy::cast_precision_loss)]
                    resolution: WindowResolution::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
                    ..default()
                }),
                ..default()
            },
            AccessibilityPlugin,
            AssetPlugin::default(),
            WinitPlugin::default(),
            RenderPlugin::default(),
            ImagePlugin::default(),
            CorePipelinePlugin,
            SpritePlugin,
            TextPlugin,
            UiPlugin,
            PbrPlugin::default(),
            GizmoPlugin,
        ))
        .add_plugins((
            CommunicationPlugin,
            LightsPlugin,
            LevelPlugin,
            CameraPlugin,
            DebugPlugin,
        ))
        .insert_resource(AssetMetaCheck::Never) // Otherwise we were getting 404-s in WASM for `*.wgsl.meta` files
        .run();
}
