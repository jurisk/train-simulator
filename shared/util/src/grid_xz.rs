use std::ops::Index;

use serde::{Deserialize, Serialize};

use crate::coords_xz::CoordsXZ;

#[derive(Clone, Serialize, Deserialize)]
pub struct GridXZ<T> {
    pub size_x: usize,
    pub size_z: usize,
    // Note: We could instead use a flat Vec<T>, but serialisation would have to be handled differently then
    data:       Vec<Vec<T>>,
}

impl<T> GridXZ<T> {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.data.len() == self.size_z && self.data.iter().all(|row| row.len() == self.size_x)
    }

    pub fn map<F, U>(&self, f: F) -> GridXZ<U>
    where
        F: Fn(&T) -> U,
    {
        GridXZ::<U> {
            size_x: self.size_x,
            size_z: self.size_z,
            data:   self
                .data
                .iter()
                .map(|row| row.iter().map(&f).collect())
                .collect(),
        }
    }
}

impl<T> Index<&CoordsXZ> for GridXZ<T> {
    type Output = T;

    fn index(&self, coords: &CoordsXZ) -> &Self::Output {
        &self.data[coords.z][coords.x]
    }
}
