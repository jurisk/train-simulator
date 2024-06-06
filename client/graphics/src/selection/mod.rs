use bevy::app::App;
use bevy::prelude::{Color, Gizmos, Plugin, Query, TypePath, Update};
use bevy_mod_raycast::deferred::RaycastSource;
use bevy_mod_raycast::prelude::{DeferredRaycastingPlugin, RaycastPluginState};

pub(crate) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DeferredRaycastingPlugin::<()>::default());
        app.insert_resource(RaycastPluginState::<()>::default()); // Add .with_debug_cursor() for debug cursor
        app.add_systems(Update, update_debug_cursor::<()>);
    }
}

#[allow(
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::match_bool
)]
pub fn update_debug_cursor<T: TypePath + Send + Sync>(
    sources: Query<&RaycastSource<T>>,
    mut gizmos: Gizmos,
) {
    for (is_first, intersection) in sources.iter().flat_map(|m| {
        m.intersections()
            .iter()
            .map(|i| i.1.clone())
            .enumerate()
            .map(|(i, hit)| (i == 0, hit))
    }) {
        let color = match is_first {
            true => Color::GREEN,
            false => Color::PINK,
        };
        gizmos.ray(intersection.position(), intersection.normal(), color);
    }
}
