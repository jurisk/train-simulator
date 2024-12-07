use serde::{Deserialize, Serialize};
use shared_physics::constants::METERS_PER_INCH;
use shared_physics::projectile::ProjectileProperties;

use crate::cargo_map::CargoMap;
use crate::resource_type::ResourceType;

pub mod projectile_info;
pub mod projectile_state;

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ProjectileType {
    Standard,
}

impl ProjectileType {
    #[must_use]
    pub fn cost_per_shot(&self) -> CargoMap {
        match self {
            ProjectileType::Standard => CargoMap::single(ResourceType::Ammunition, 0.01),
        }
    }

    #[must_use]
    pub const fn projectile_properties(&self) -> ProjectileProperties {
        match self {
            // TODO HIGH: Think about how to use real values, considering your game scale is in tiles, not meters
            ProjectileType::Standard => {
                ProjectileProperties::create_from_inches(16.0, 1225.0, 1.829, 20.0)
            },
        }
    }
}

// Later: Throw out unused code
#[derive(Debug, Clone, Copy)]
#[expect(dead_code, clippy::enum_variant_names)]
enum ShellType {
    Naval16Inch,
    Naval14Inch,
    Naval5Inch,
}

impl ShellType {
    #[must_use]
    #[expect(dead_code)]
    pub const fn projectile_properties(self: ShellType) -> ProjectileProperties {
        match self {
            ShellType::Naval16Inch => {
                // From https://en.wikipedia.org/wiki/16-inch/50-caliber_Mark_7_gun
                ProjectileProperties::create_from_inches(16.0, 1225.0, 1.829, 762.0)
            },
            ShellType::Naval14Inch => {
                // From http://www.navweaps.com/Weapons/WNJAP_14-45_t41.php
                // Failed to find shell height, so just estimating it
                ProjectileProperties::create_from_inches(
                    14.0,
                    673.5,
                    4.0 * 14.0 * METERS_PER_INCH,
                    772.5,
                )
            },
            ShellType::Naval5Inch => {
                // From http://www.navweaps.com/Weapons/WNGER_5-45_skc34.php
                ProjectileProperties::create_from_inches(5.0, 28.0, 0.68, 830.0)
            },
        }
    }
}
