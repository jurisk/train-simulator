use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum StationOrientation {
    NorthToSouth,
    EastToWest,
}

// TODO: Build some test stations in test setup
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct StationType {
    pub orientation:     StationOrientation,
    pub platforms:       usize,
    pub length_in_tiles: usize,
}
