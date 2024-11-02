use itertools::Itertools;
use shared_domain::building::WithRelativeTileCoverage;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::map_level::map_level::MapLevel;
use shared_domain::map_level::zoning::{ZoningInfo, ZoningType};
use shared_domain::resource_type::ResourceType;
use shared_domain::scenario::PlayerProfile;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::tile_coverage::TileCoverage;
use shared_domain::{PlayerId, ZoningId};
use shared_util::random::choose;

use crate::profile::Profile;

const ZONING_COEF: usize = 2;
fn default_zoning_counts(zoning_type: ZoningType) -> usize {
    // Later: These can actually get generated from supply chain graph
    let this_coef = match zoning_type {
        ZoningType::Industrial => 36,
        ZoningType::Source(ResourceType::Iron | ResourceType::Coal) => 4,
        ZoningType::Source(ResourceType::Wood) => 2,
        ZoningType::Source(_) => 1,
    };
    ZONING_COEF * this_coef
}

fn options(map_level: &MapLevel) -> Vec<TileCoordsXZ> {
    let mut result = vec![];

    for tile in map_level.all_tile_coords() {
        // Doing 5x5 instead of 3x3 to leave room for stations
        let coverage = TileCoverage::rectangular_odd(tile, 5, 5);
        if map_level.can_build_for_coverage(&coverage).is_ok() {
            result.push(tile);
        }
    }

    result
}

fn add_zoning(map_level: &mut MapLevel, zoning_type: ZoningType, tile: TileCoordsXZ) {
    let zoning = ZoningInfo::new(ZoningId::random(), zoning_type, tile);

    map_level.zoning_mut().add_zoning(zoning);
}

#[expect(clippy::unwrap_used)]
fn closest_player(tile: TileCoordsXZ, players: &[PlayerProfile]) -> PlayerId {
    players
        .iter()
        .min_by_key(|player| player.initial_construction_yard.manhattan_distance(tile))
        .unwrap()
        .player_id
}

#[expect(clippy::missing_panics_doc, clippy::expect_used, clippy::unwrap_used)]
pub fn augment(map_level: &mut MapLevel, profile: &Profile) {
    let mut options = options(map_level)
        .into_iter()
        .map(|tile| (closest_player(tile, &profile.players), tile))
        .filter(|(player_id, tile)| {
            // Later: If we teach AI to build extension construction yards, this is no longer needed
            let construction_yard_tile = profile.players_construction_yard_at(*player_id);
            let distance = construction_yard_tile.manhattan_distance(*tile);
            let construction_yard = IndustryType::ConstructionYard;
            let threshold = construction_yard.supply_range_in_tiles().unwrap();
            distance <= threshold
        })
        .into_group_map();
    for player in &profile.players {
        let options = options
            .get_mut(&player.player_id)
            .expect("Player should have options");
        for zoning in ZoningType::all() {
            let count = default_zoning_counts(zoning);
            for _ in 0 .. count {
                let chosen = *choose(options).expect("Options should not be empty");
                // We use 5x5 coverage here on purpose to space out the stations
                let a = zoning.relative_tiles_used().extend(1).offset_by(chosen);
                options.retain(|tile| {
                    let b = TileCoverage::rectangular_odd(*tile, 5, 5);
                    !a.intersects(&b)
                });
                add_zoning(map_level, zoning, chosen);
            }
        }
    }
}
