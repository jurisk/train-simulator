use crate::MilitaryUnitId;
use crate::building::military_unit_type::MilitaryUnitType;
use crate::tile_coords_xz::TileCoordsXZ;

pub enum MilitaryUnitLocation {
    Fixed(TileCoordsXZ),
    Movable(TileCoordsXZ),
}

pub struct MilitaryUnitInfo {
    id:        MilitaryUnitId,
    unit_type: MilitaryUnitType,
    location:  MilitaryUnitLocation,
}
