use bevy::math::Vec3;
use shared_domain::map_level::terrain::Terrain;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::transport::track_type::TrackType;

pub(crate) fn rail_positions(
    tile: TileCoordsXZ,
    track_type: TrackType,
    terrain: &Terrain,
) -> ((Vec3, Vec3), (Vec3, Vec3)) {
    let (a, b) = track_type.connections_clockwise();

    let (a1, a2) = terrain.vertex_coordinates_clockwise(a, tile);
    let (b1, b2) = terrain.vertex_coordinates_clockwise(b, tile);

    let (a1, a2) = pick_rail_positions(a1, a2);
    let (b1, b2) = pick_rail_positions(b1, b2);

    ((a1, a2), (b1, b2))
}

// The usual rail car is 10 feet wide, 10 feet high, and 50 feet long. We want to fit 2 cars per tile, so one tile is 100 feet or 30 meters.
// The standard track gauge is 1435 mm. Thus, 0.1 tiles is a good approximation for the track gauge.
const TRACK_GAUGE: f32 = 0.1;
pub(crate) fn pick_rail_positions(a: Vec3, b: Vec3) -> (Vec3, Vec3) {
    let direction = b - a;
    let midpoint = (a + b) / 2.0;
    let direction = direction.normalize();
    let offset = direction * TRACK_GAUGE / 2.0;
    (midpoint - offset, midpoint + offset)
}
