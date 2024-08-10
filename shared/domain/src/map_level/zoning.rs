#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};

use crate::tile_coords_xz::TileCoordsXZ;
use crate::ResourceId;

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
enum ZoningType {
    CoalDeposit,
    IronDeposit,
    IndustrialBuilding,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ZoningInfo {
    id:             ResourceId,
    zoning_type:    ZoningType,
    reference_tile: TileCoordsXZ,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Zoning(Vec<ZoningInfo>);
