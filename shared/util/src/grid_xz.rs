use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};

use crate::coords_xz::CoordsXZ;

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct GridXZ<T> {
    pub size_x: usize,
    pub size_z: usize,
    // Note: We could instead use a flat Vec<T>, but serialisation would have to be handled differently then
    data:       Vec<Vec<T>>,
}

impl<T> GridXZ<T> {
    #[must_use]
    pub fn new(data: Vec<Vec<T>>) -> Self {
        let size_x = data.first().map_or(0, Vec::len);
        let size_z = data.len();
        debug_assert!(data.iter().all(|row| row.len() == size_x));
        Self {
            size_x,
            size_z,
            data,
        }
    }

    #[must_use]
    pub fn filled_with(size_x: usize, size_z: usize, value: T) -> Self
    where
        T: Clone,
    {
        Self {
            size_x,
            size_z,
            data: vec![vec![value; size_x]; size_z],
        }
    }

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

    pub fn coords(&self) -> impl Iterator<Item = CoordsXZ> + '_ {
        (0 .. self.size_z).flat_map(move |z| (0 .. self.size_x).map(move |x| CoordsXZ::new(x, z)))
    }
}

impl<T> Index<&CoordsXZ> for GridXZ<T> {
    type Output = T;

    fn index(&self, coords: &CoordsXZ) -> &Self::Output {
        &self.data[coords.z][coords.x]
    }
}

impl<T> IndexMut<&CoordsXZ> for GridXZ<T> {
    fn index_mut(&mut self, coords: &CoordsXZ) -> &mut Self::Output {
        &mut self.data[coords.z][coords.x]
    }
}
