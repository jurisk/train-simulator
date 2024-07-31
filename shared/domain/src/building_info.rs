use std::collections::HashSet;
use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};
use shared_util::coords_xz::CoordsXZ;
use shared_util::direction_xz::DirectionXZ;

use crate::building_type::BuildingType;
use crate::cargo_map::CargoMap;
use crate::game_time::GameTimeDiff;
use crate::production_type::ProductionType;
use crate::resource_type::ResourceType;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::transport::progress_within_tile::ProgressWithinTile;
use crate::transport::tile_track::TileTrack;
use crate::transport::track_type::TrackType;
use crate::transport::transport_location::TransportLocation;
use crate::{BuildingId, PlayerId};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct BuildingStaticInfo {
    owner_id:       PlayerId,
    building_id:    BuildingId,
    reference_tile: TileCoordsXZ,
    building_type:  BuildingType,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct BuildingDynamicInfo {
    cargo: CargoMap,
}

impl Debug for BuildingDynamicInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.cargo)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct BuildingInfo {
    static_info:  BuildingStaticInfo,
    // TODO: Not all building types have dynamic info, and it can differ between building types... think of a better design.
    dynamic_info: BuildingDynamicInfo,
}

impl Debug for BuildingInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?} {:?} {:?}",
            self.building_id(),
            self.static_info.reference_tile,
            self.static_info.building_type,
            self.dynamic_info
        )
    }
}

impl BuildingInfo {
    #[must_use]
    pub fn new(
        owner_id: PlayerId,
        building_id: BuildingId,
        reference_tile: TileCoordsXZ,
        building_type: BuildingType,
    ) -> Self {
        Self {
            static_info:  BuildingStaticInfo {
                owner_id,
                building_id,
                reference_tile,
                building_type,
            },
            dynamic_info: BuildingDynamicInfo {
                cargo: CargoMap::new(),
            },
        }
    }

    pub fn add_cargo(&mut self, cargo: &CargoMap) {
        self.dynamic_info.cargo += cargo;
    }

    pub fn remove_cargo(&mut self, cargo: &CargoMap) {
        self.dynamic_info.cargo -= cargo;
    }

    pub fn update_dynamic_info(&mut self, dynamic_info: &BuildingDynamicInfo) {
        self.dynamic_info = dynamic_info.clone();
    }

    // TODO: Refactor this as this is really station specific, not building specific
    #[must_use]
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    pub fn transport_location_at_station(
        &self,
        tile: TileCoordsXZ,
        direction: DirectionXZ,
    ) -> Option<TransportLocation> {
        let station_type = match self.building_type() {
            BuildingType::Track(_) | BuildingType::Production(_) => None,
            BuildingType::Station(station_type) => Some(station_type),
        }?;
        let (_, _, exit_track) = station_type
            .exit_tile_tracks(self.reference_tile())
            .into_iter()
            .find(|(_platform, _pointing_in, track)| {
                track.tile_coords_xz == tile && track.pointing_in == direction
            })?;
        let diff: CoordsXZ = exit_track.pointing_in.reverse().into();
        let mut tile_path = vec![];
        for i in 0 .. station_type.length_in_tiles {
            let delta: CoordsXZ = diff * (i as i32);
            let delta_t: TileCoordsXZ = delta.into();
            let tile_coords_xz = exit_track.tile_coords_xz + delta_t;
            let tile_track = TileTrack {
                tile_coords_xz,
                track_type: exit_track.track_type,
                pointing_in: exit_track.pointing_in,
            };
            tile_path.push(tile_track);
        }
        let progress_within_tile = ProgressWithinTile::about_to_exit();
        Some(TransportLocation::new(tile_path, progress_within_tile))
    }

    #[must_use]
    pub fn owner_id(&self) -> PlayerId {
        self.static_info.owner_id
    }

    #[must_use]
    pub fn dynamic_info(&self) -> BuildingDynamicInfo {
        self.dynamic_info.clone()
    }

    #[must_use]
    pub fn building_id(&self) -> BuildingId {
        self.static_info.building_id
    }

    #[must_use]
    pub fn building_type(&self) -> BuildingType {
        self.static_info.building_type
    }

    #[must_use]
    pub fn reference_tile(&self) -> TileCoordsXZ {
        self.static_info.reference_tile
    }

    #[must_use]
    pub fn tile_tracks(&self) -> Vec<TileTrack> {
        let mut results = Vec::new();
        for relative_tile in self.building_type().relative_tiles_used().to_set() {
            for track_type in self.building_type().track_types_at(relative_tile) {
                for pointing_in in track_type.connections() {
                    results.push(TileTrack {
                        tile_coords_xz: self.reference_tile() + relative_tile,
                        track_type,
                        pointing_in,
                    });
                }
            }
        }
        results
    }

    #[must_use]
    pub(crate) fn track_types_at(&self, tile: TileCoordsXZ) -> Vec<TrackType> {
        self.building_type()
            .track_types_at(tile - self.reference_tile())
    }

    #[must_use]
    pub fn covers_tiles(&self) -> TileCoverage {
        self.building_type()
            .relative_tiles_used()
            .offset_by(self.reference_tile())
    }

    pub fn advance(&mut self, diff: GameTimeDiff) {
        let seconds = diff.to_seconds();
        match self.building_type() {
            BuildingType::Track(_) | BuildingType::Station(_) => {},
            BuildingType::Production(production_type) => {
                self.advance_production(seconds, production_type);
            },
        }
    }

    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn accepted_inputs(&self) -> HashSet<ResourceType> {
        match self.building_type() {
            BuildingType::Track(_) => HashSet::new(),
            BuildingType::Station(_) => HashSet::new(),
            BuildingType::Production(production_type) => {
                let mut result = HashSet::new();
                for input in production_type.transform_per_second().inputs {
                    result.insert(input.resource);
                }
                result
            },
        }
    }

    #[must_use]
    pub fn shippable_cargo(&self) -> CargoMap {
        match self.building_type() {
            BuildingType::Track(_) => CargoMap::new(),
            BuildingType::Station(_) => self.dynamic_info().cargo.clone(),
            BuildingType::Production(production_type) => {
                let transform = production_type.transform_per_second();
                let mut result = CargoMap::new();
                for output in transform.outputs {
                    let resource = output.resource;
                    // Later: This is now insta-shipping of everything... consider doing this more gradually?
                    let amount = self.dynamic_info.cargo.get(resource);
                    result.add(resource, amount);
                }
                result
            },
        }
    }

    fn advance_production(&mut self, seconds: f32, production_type: ProductionType) {
        let transform = production_type.transform_per_second();
        let utilisation =
            transform.calculate_utilisation_percentage(&self.dynamic_info.cargo, seconds);
        let effective = seconds * utilisation;

        for item in transform.inputs {
            self.dynamic_info
                .cargo
                .add(item.resource, -item.amount * effective);
        }
        for item in transform.outputs {
            self.dynamic_info
                .cargo
                .add(item.resource, item.amount * effective);
        }
    }

    #[must_use]
    pub fn manhattan_distance_between_closest_tiles(a: &BuildingInfo, b: &BuildingInfo) -> i32 {
        let mut result = i32::MAX;
        for a in a.covers_tiles().to_set() {
            for b in b.covers_tiles().to_set() {
                let distance = a.manhattan_distance(b);
                result = result.min(distance);
            }
        }
        result
    }
}
