use std::collections::BTreeSet;

use log::warn;
use serde::{Deserialize, Serialize};
use shared_util::grid_xz::GridXZ;

use crate::building::track_info::TrackInfo;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::transport::track_type::TrackType;
use crate::{PlayerId, TrackId};

#[derive(Serialize, Deserialize, Clone, PartialEq, Default, Debug)]
pub enum MaybeTracksOnTile {
    #[default]
    Empty,
    SingleOwner {
        owner_id:    PlayerId,
        // TODO HIGH: Move to `BitSet` if it actually is faster
        track_types: BTreeSet<TrackType>,
    },
}

impl MaybeTracksOnTile {
    #[must_use]
    pub fn owner_id(&self) -> Option<PlayerId> {
        match self {
            Self::Empty => None,
            Self::SingleOwner { owner_id, .. } => Some(*owner_id),
        }
    }

    #[must_use]
    pub fn track_types(&self) -> BTreeSet<TrackType> {
        match self {
            Self::Empty => BTreeSet::new(),
            Self::SingleOwner { track_types, .. } => track_types.clone(),
        }
    }

    fn remove_track_type(&mut self, track_type: TrackType) {
        match self {
            Self::Empty => {
                warn!("Tried to remove track from empty tile: {:?}", track_type);
            },
            Self::SingleOwner { track_types, .. } => {
                track_types.remove(&track_type);
                if track_types.is_empty() {
                    *self = Self::Empty;
                }
            },
        }
    }

    fn append_track(&mut self, track: &TrackInfo) {
        match self {
            Self::Empty => {
                *self = Self::SingleOwner {
                    owner_id:    track.owner_id(),
                    track_types: BTreeSet::from([track.track_type]),
                };
            },
            Self::SingleOwner {
                owner_id,
                track_types,
            } => {
                if *owner_id == track.owner_id() {
                    track_types.insert(track.track_type);
                } else {
                    warn!(
                        "Tried to add track to tile with different owner: {:?}",
                        track
                    );
                }
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct TrackState {
    grid: GridXZ<TileCoordsXZ, MaybeTracksOnTile>,
}

impl TrackState {
    #[must_use]
    pub(crate) fn new(size_x: usize, size_z: usize) -> Self {
        Self {
            grid: GridXZ::filled_with(size_x, size_z, MaybeTracksOnTile::default()),
        }
    }

    #[must_use]
    pub(crate) fn tracks_at(&self, tile: TileCoordsXZ) -> MaybeTracksOnTile {
        self.grid.get(tile).cloned().unwrap_or_default()
    }

    pub(crate) fn attempt_to_remove_track(
        &mut self,
        requesting_player_id: PlayerId,
        track_id: TrackId,
    ) -> Result<(), ()> {
        let track = self.tracks_at(track_id.tile);
        if track.owner_id() == Some(requesting_player_id) {
            self.remove_track(track_id);
            Ok(())
        } else {
            Err(())
        }
    }

    pub(crate) fn remove_track(&mut self, track_id: TrackId) {
        match self.grid.get_mut(track_id.tile) {
            None => {
                warn!(
                    "Tried to remove track from non-existing tile: {:?}",
                    track_id
                );
            },
            Some(found) => {
                found.remove_track_type(track_id.track_type);
            },
        }
    }

    pub(crate) fn append_tracks(&mut self, tracks: Vec<TrackInfo>) {
        for track in tracks {
            self.append_track(&track);
        }
    }

    fn append_track(&mut self, track: &TrackInfo) {
        match self.grid.get_mut(track.tile) {
            None => {
                warn!("Tried to add track to non-existing tile: {:?}", track.tile);
            },
            Some(contents) => {
                contents.append_track(track);
            },
        }
    }

    #[must_use]
    pub(crate) fn all_track_infos(&self) -> Vec<TrackInfo> {
        let mut results = vec![];
        for tile in self.grid.coords() {
            if let MaybeTracksOnTile::SingleOwner {
                owner_id,
                track_types,
            } = self.tracks_at(tile)
            {
                let track_infos = track_types
                    .into_iter()
                    .map(|track_type| TrackInfo::new(owner_id, tile, track_type));
                results.extend(track_infos);
            }
        }
        results
    }

    // TODO HIGH: This is frequently called and should be optimised
    #[must_use]
    pub(crate) fn track_types_at(&self, tile: TileCoordsXZ) -> BTreeSet<TrackType> {
        self.tracks_at(tile).track_types()
    }
}