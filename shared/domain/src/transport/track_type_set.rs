use std::collections::HashSet;
use std::fmt;
use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::transport::track_type::TrackType;

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
pub struct TrackTypeSet {
    byte: u8,
}

impl Default for TrackTypeSet {
    fn default() -> Self {
        Self::empty()
    }
}

impl Debug for TrackTypeSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut set = HashSet::new();
        for track_type in TrackType::all() {
            if self.contains(track_type) {
                set.insert(track_type);
            }
        }
        write!(f, "{set:?}")
    }
}

const fn mask(track_type: TrackType) -> u8 {
    1 << track_type as u8
}

impl TrackTypeSet {
    #[must_use]
    pub const fn empty() -> Self {
        Self { byte: 0 }
    }

    #[must_use]
    pub const fn single(track_type: TrackType) -> Self {
        Self {
            byte: mask(track_type),
        }
    }

    #[must_use]
    pub const fn contains(&self, track_type: TrackType) -> bool {
        self.byte & mask(track_type) != 0
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.byte == 0
    }

    pub fn insert(&mut self, track_type: TrackType) {
        self.byte |= mask(track_type);
    }

    pub fn remove(&mut self, track_type: TrackType) {
        self.byte &= !mask(track_type);
    }

    pub fn clear(&mut self) {
        self.byte = 0;
    }
}

impl IntoIterator for TrackTypeSet {
    type IntoIter = std::vec::IntoIter<TrackType>;
    type Item = TrackType;

    fn into_iter(self) -> Self::IntoIter {
        // Later: We could optimise it later to avoid the allocation, but this should not be used in the critical path anyway.
        let mut results = Vec::new();

        for track_type in TrackType::all() {
            if self.contains(track_type) {
                results.push(track_type);
            }
        }

        results.into_iter()
    }
}
