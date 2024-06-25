use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum StationOrientation {
    NorthSouth,
    EastWest,
}

// TODO: Build some test stations in test setup
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct StationType {
    orientation:     StationOrientation,
    platforms:       usize,
    length_in_tiles: usize,
}
