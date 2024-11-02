use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};
use shared_util::coords_xz::CoordsXZ;
use shared_util::direction_xz::DirectionXZ;

use crate::building::WithRelativeTileCoverage;
use crate::building::building_info::{
    BuildingDynamicInfo, BuildingInfo, BuildingStaticInfo, WithBuildingDynamicInfo,
    WithBuildingDynamicInfoMut, WithCostToBuild, WithOwner, WithTileCoverage,
};
use crate::building::industry_type::IndustryType;
use crate::building::station_type::StationType;
use crate::cargo_map::{CargoMap, WithCargo};
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::transport::progress_within_tile::ProgressWithinTile;
use crate::transport::tile_track::TileTrack;
use crate::transport::track_type_set::TrackTypeSet;
use crate::transport::transport_location::TransportLocation;
use crate::{PlayerId, StationId};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct StationInfo {
    id:           StationId,
    station_type: StationType,
    static_info:  BuildingStaticInfo,
    dynamic_info: BuildingDynamicInfo,
}

impl Debug for StationInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?} {:?} {:?}",
            self.id(),
            self.static_info.reference_tile(),
            self.station_type,
            self.dynamic_info
        )
    }
}

impl StationInfo {
    #[must_use]
    pub fn new(
        owner_id: PlayerId,
        id: StationId,
        reference_tile: TileCoordsXZ,
        station_type: StationType,
    ) -> Self {
        Self {
            id,
            station_type,
            static_info: BuildingStaticInfo::new(owner_id, reference_tile),
            dynamic_info: BuildingDynamicInfo::new(CargoMap::new()),
        }
    }

    pub fn update_dynamic_info(&mut self, dynamic_info: &BuildingDynamicInfo) {
        *self.dynamic_info_mut() = dynamic_info.clone();
    }

    #[must_use]
    pub fn station_exit_tile_tracks(&self) -> Vec<TileTrack> {
        self.station_type
            .exit_tile_tracks(self.reference_tile())
            .into_iter()
            .map(|(_, track)| track)
            .collect()
    }

    #[must_use]
    #[expect(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    pub fn transport_location_at_station(
        &self,
        tile: TileCoordsXZ,
        direction: DirectionXZ,
    ) -> Option<TransportLocation> {
        let exit_track = self
            .station_exit_tile_tracks()
            .into_iter()
            .find(|track| track.tile == tile && track.pointing_in == direction)?;
        let diff: CoordsXZ = exit_track.pointing_in.reverse().into();
        let mut tile_path = vec![];
        for i in 0 .. self.station_type.length_in_tiles {
            let delta: CoordsXZ = diff * (i as i32);
            let delta_t: TileCoordsXZ = delta.into();
            let tile_coords_xz = exit_track.tile + delta_t;
            let tile_track = TileTrack {
                tile:        tile_coords_xz,
                track_type:  exit_track.track_type,
                pointing_in: exit_track.pointing_in,
            };
            tile_path.push(tile_track);
        }
        let progress_within_tile = ProgressWithinTile::about_to_exit();
        Some(TransportLocation::new(tile_path, progress_within_tile))
    }

    #[must_use]
    pub fn id(&self) -> StationId {
        self.id
    }

    #[must_use]
    pub fn station_type(&self) -> StationType {
        self.station_type
    }

    #[must_use]
    pub fn reference_tile(&self) -> TileCoordsXZ {
        self.static_info.reference_tile()
    }

    #[must_use]
    pub fn tile_tracks(&self) -> Vec<TileTrack> {
        let mut results = Vec::new();
        for relative_tile in self.station_type().relative_tiles_used() {
            for track_type in self.station_type().track_types_at(relative_tile) {
                for pointing_in in track_type.connections() {
                    results.push(TileTrack {
                        tile: self.reference_tile() + relative_tile,
                        track_type,
                        pointing_in,
                    });
                }
            }
        }
        results
    }

    #[must_use]
    pub(crate) fn station_track_types_at(&self, tile: TileCoordsXZ) -> TrackTypeSet {
        self.station_type()
            .track_types_at(tile - self.reference_tile())
    }

    #[must_use]
    pub fn station_shippable_cargo(&self) -> CargoMap {
        self.dynamic_info().cargo().clone()
    }
}

impl WithBuildingDynamicInfo for StationInfo {
    fn dynamic_info(&self) -> &BuildingDynamicInfo {
        &self.dynamic_info
    }
}

impl WithBuildingDynamicInfoMut for StationInfo {
    fn dynamic_info_mut(&mut self) -> &mut BuildingDynamicInfo {
        &mut self.dynamic_info
    }
}

impl WithRelativeTileCoverage for StationInfo {
    fn relative_tiles_used(&self) -> TileCoverage {
        self.station_type().relative_tiles_used()
    }
}

impl BuildingInfo for StationInfo {}

impl WithOwner for StationInfo {
    fn owner_id(&self) -> PlayerId {
        self.static_info.owner_id()
    }
}

impl WithTileCoverage for StationInfo {
    fn covers_tiles(&self) -> TileCoverage {
        self.relative_tiles_used().offset_by(self.reference_tile())
    }
}

impl WithCostToBuild for StationInfo {
    fn cost_to_build(&self) -> (IndustryType, CargoMap) {
        self.station_type.cost_to_build()
    }
}
