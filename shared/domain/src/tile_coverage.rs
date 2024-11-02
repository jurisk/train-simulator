use serde::{Deserialize, Serialize};

use crate::tile_coords_xz::{TileCoordsXZ, TileDistance, closest_tile_distance};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub enum TileCoverage {
    Single(TileCoordsXZ),
    Rectangular {
        north_west_inclusive: TileCoordsXZ,
        south_east_inclusive: TileCoordsXZ,
    },
}

impl TileCoverage {
    #[must_use]
    pub fn single_at_zero() -> Self {
        TileCoverage::Single(TileCoordsXZ::ZERO)
    }

    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    pub fn rectangular_odd(
        center: TileCoordsXZ,
        size_x: TileDistance,
        size_z: TileDistance,
    ) -> Self {
        assert_eq!(size_x % 2, 1, "size_x must be odd");
        assert_eq!(size_z % 2, 1, "size_z must be odd");
        TileCoverage::Rectangular {
            north_west_inclusive: TileCoordsXZ::new(center.x - size_x / 2, center.z - size_z / 2),
            south_east_inclusive: TileCoordsXZ::new(center.x + size_x / 2, center.z + size_z / 2),
        }
    }

    #[must_use]
    pub fn extend(&self, diff: TileDistance) -> Self {
        match self {
            TileCoverage::Single(tile) => {
                TileCoverage::Rectangular {
                    north_west_inclusive: TileCoordsXZ::new(tile.x - diff, tile.z - diff),
                    south_east_inclusive: TileCoordsXZ::new(tile.x + diff, tile.z + diff),
                }
            },
            TileCoverage::Rectangular {
                north_west_inclusive,
                south_east_inclusive,
            } => {
                TileCoverage::Rectangular {
                    north_west_inclusive: TileCoordsXZ::new(
                        north_west_inclusive.x - diff,
                        north_west_inclusive.z - diff,
                    ),
                    south_east_inclusive: TileCoordsXZ::new(
                        south_east_inclusive.x + diff,
                        south_east_inclusive.z + diff,
                    ),
                }
            },
        }
    }

    #[must_use]
    pub fn contains(&self, tile: TileCoordsXZ) -> bool {
        match self {
            TileCoverage::Single(single_tile) => *single_tile == tile,
            TileCoverage::Rectangular {
                north_west_inclusive,
                south_east_inclusive,
            } => {
                tile.x >= north_west_inclusive.x
                    && tile.x <= south_east_inclusive.x
                    && tile.z >= north_west_inclusive.z
                    && tile.z <= south_east_inclusive.z
            },
        }
    }

    #[must_use]
    pub fn offset_by(self, tile: TileCoordsXZ) -> Self {
        match self {
            TileCoverage::Single(single_tile) => TileCoverage::Single(single_tile + tile),
            TileCoverage::Rectangular {
                north_west_inclusive,
                south_east_inclusive,
            } => {
                TileCoverage::Rectangular {
                    north_west_inclusive: north_west_inclusive + tile,
                    south_east_inclusive: south_east_inclusive + tile,
                }
            },
        }
    }

    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        match (self, other) {
            (TileCoverage::Single(a), TileCoverage::Single(b)) => a == b,
            (TileCoverage::Single(a), b @ TileCoverage::Rectangular { .. }) => b.contains(*a),
            (a @ TileCoverage::Rectangular { .. }, TileCoverage::Single(b)) => a.contains(*b),
            (
                TileCoverage::Rectangular {
                    north_west_inclusive: a_north_west,
                    south_east_inclusive: a_south_east,
                },
                TileCoverage::Rectangular {
                    north_west_inclusive: b_north_west,
                    south_east_inclusive: b_south_east,
                },
            ) => {
                let one_left_of_other =
                    a_north_west.x > b_south_east.x || b_north_west.x > a_south_east.x;
                let one_above_another =
                    a_north_west.z > b_south_east.z || b_north_west.z > a_south_east.z;
                !(one_left_of_other || one_above_another)
            },
        }
    }

    const fn min_x(&self) -> TileDistance {
        match self {
            TileCoverage::Single(tile) => tile.x,
            TileCoverage::Rectangular {
                north_west_inclusive,
                south_east_inclusive: _,
            } => north_west_inclusive.x,
        }
    }

    const fn max_x(&self) -> TileDistance {
        match self {
            TileCoverage::Single(tile) => tile.x,
            TileCoverage::Rectangular {
                north_west_inclusive: _,
                south_east_inclusive,
            } => south_east_inclusive.x,
        }
    }

    const fn min_z(&self) -> TileDistance {
        match self {
            TileCoverage::Single(tile) => tile.z,
            TileCoverage::Rectangular {
                north_west_inclusive,
                south_east_inclusive: _,
            } => north_west_inclusive.z,
        }
    }

    const fn max_z(&self) -> TileDistance {
        match self {
            TileCoverage::Single(tile) => tile.z,
            TileCoverage::Rectangular {
                north_west_inclusive: _,
                south_east_inclusive,
            } => south_east_inclusive.z,
        }
    }

    #[must_use]
    pub const fn manhattan_distance_between_closest_tiles(
        a: &TileCoverage,
        b: &TileCoverage,
    ) -> TileDistance {
        let closest_x = closest_tile_distance(a.min_x(), a.max_x(), b.min_x(), b.max_x());
        let closest_z = closest_tile_distance(a.min_z(), a.max_z(), b.min_z(), b.max_z());
        closest_x + closest_z
    }
}

#[expect(clippy::module_name_repetitions)]
pub struct TileCoverageIterator {
    next:          Option<TileCoordsXZ>,
    tile_coverage: TileCoverage,
}

impl IntoIterator for TileCoverage {
    type IntoIter = TileCoverageIterator;
    type Item = TileCoordsXZ;

    fn into_iter(self) -> Self::IntoIter {
        let next = match self {
            TileCoverage::Single(tile) => tile,
            TileCoverage::Rectangular {
                north_west_inclusive,
                south_east_inclusive: _south_east_inclusive,
            } => north_west_inclusive,
        };
        TileCoverageIterator {
            next:          Some(next),
            tile_coverage: self,
        }
    }
}

impl Iterator for TileCoverageIterator {
    type Item = TileCoordsXZ;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next {
            None => None,
            Some(result) => {
                match self.tile_coverage {
                    TileCoverage::Single(_) => {
                        self.next = None;
                        Some(result)
                    },
                    TileCoverage::Rectangular {
                        north_west_inclusive,
                        south_east_inclusive,
                    } => {
                        let mut next = result;
                        next.x += 1;
                        if next.x > south_east_inclusive.x {
                            next.x = north_west_inclusive.x;
                            next.z += 1;
                        }
                        if next.z > south_east_inclusive.z {
                            self.next = None;
                        } else {
                            self.next = Some(next);
                        }
                        Some(result)
                    },
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tile_coords_xz::TileCoordsXZ;
    use crate::tile_coverage::TileCoverage;

    #[test]
    fn test_overlaps() {
        let a = TileCoverage::rectangular_odd(TileCoordsXZ::new(73, 31), 3, 3);
        let b = TileCoverage::rectangular_odd(TileCoordsXZ::new(74, 32), 3, 3);
        assert!(a.intersects(&b));
    }
}
