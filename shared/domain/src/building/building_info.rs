use std::collections::HashSet;
use std::fmt::{Debug, Formatter};

use log::error;
use serde::{Deserialize, Serialize};
use shared_util::coords_xz::CoordsXZ;
use shared_util::direction_xz::DirectionXZ;

use crate::building::building_type::BuildingType;
use crate::building::industry_type::IndustryType;
use crate::cargo_map::CargoMap;
use crate::game_time::GameTimeDiff;
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

impl BuildingDynamicInfo {
    pub fn add_cargo(&mut self, cargo: &CargoMap) {
        self.cargo += cargo;
    }

    pub fn remove_cargo(&mut self, cargo: &CargoMap) {
        self.cargo -= cargo;
    }
}

impl Debug for BuildingDynamicInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.cargo)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct BuildingInfo {
    static_info:  BuildingStaticInfo,
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

pub trait WithBuildingDynamicInfo {
    fn dynamic_info(&self) -> &BuildingDynamicInfo;
    fn dynamic_info_mut(&mut self) -> &mut BuildingDynamicInfo;
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
        self.dynamic_info_mut().add_cargo(cargo);
    }

    pub fn remove_cargo(&mut self, cargo: &CargoMap) {
        self.dynamic_info_mut().remove_cargo(cargo);
    }

    pub fn update_dynamic_info(&mut self, dynamic_info: &BuildingDynamicInfo) {
        *self.dynamic_info_mut() = dynamic_info.clone();
    }

    #[must_use]
    pub fn station_exit_tile_tracks(&self) -> Vec<TileTrack> {
        if let BuildingType::Station(station_type) = self.building_type() {
            station_type
                .exit_tile_tracks(self.reference_tile())
                .into_iter()
                .map(|(_, track)| track)
                .collect()
        } else {
            vec![]
        }
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
            BuildingType::Industry(_) => None,
            BuildingType::Station(station_type) => Some(station_type),
        }?;
        let exit_track = self
            .station_exit_tile_tracks()
            .into_iter()
            .find(|track| track.tile_coords_xz == tile && track.pointing_in == direction)?;
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
    pub(crate) fn station_track_types_at(&self, tile: TileCoordsXZ) -> Vec<TrackType> {
        self.building_type()
            .track_types_at(tile - self.reference_tile())
    }

    #[must_use]
    pub fn covers_tiles(&self) -> TileCoverage {
        self.building_type()
            .relative_tiles_used()
            .offset_by(self.reference_tile())
    }

    pub fn advance_industry_building(&mut self, diff: GameTimeDiff) {
        let seconds = diff.to_seconds();
        match self.building_type() {
            BuildingType::Station(station_id) => {
                error!("Tried to advance industry on a station {station_id:?}");
            },
            BuildingType::Industry(industry_type) => {
                self.advance_industry(seconds, industry_type);
            },
        }
    }

    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn industry_transform_inputs(&self) -> HashSet<ResourceType> {
        match self.building_type() {
            BuildingType::Station(_) => HashSet::new(),
            BuildingType::Industry(industry_type) => {
                let mut result = HashSet::new();
                for input in industry_type.transform_per_second().inputs {
                    result.insert(input.resource);
                }
                result
            },
        }
    }

    #[must_use]
    pub fn industry_building_shippable_cargo(&self) -> CargoMap {
        match self.building_type() {
            BuildingType::Station(_) => {
                unreachable!("Should not call this for stations");
            },
            BuildingType::Industry(industry_type) => {
                let transform = industry_type.transform_per_second();
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

    #[must_use]
    pub fn station_shippable_cargo(&self) -> CargoMap {
        match self.building_type() {
            BuildingType::Station(_) => self.dynamic_info().cargo.clone(),
            BuildingType::Industry(_) => {
                unreachable!("Should not call this for industries");
            },
        }
    }

    fn advance_industry(&mut self, seconds: f32, industry_type: IndustryType) {
        let transform = industry_type.transform_per_second();
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
}

impl WithBuildingDynamicInfo for BuildingInfo {
    fn dynamic_info(&self) -> &BuildingDynamicInfo {
        &self.dynamic_info
    }

    fn dynamic_info_mut(&mut self) -> &mut BuildingDynamicInfo {
        &mut self.dynamic_info
    }
}
