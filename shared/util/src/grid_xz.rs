#![allow(clippy::cast_sign_loss)]

use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};

use crate::coords_xz::CoordsXZ;

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GridXZ<K, V> {
    pub size_x: usize,
    pub size_z: usize,
    // Note: We could instead use a flat Vec<T>, but serialisation would have to be handled differently then
    data:       Vec<Vec<V>>,

    #[serde(skip)]
    _marker: PhantomData<K>,
}

impl<K, V> GridXZ<K, V> {
    #[must_use]
    pub fn new(data: Vec<Vec<V>>) -> Self {
        let size_x = data.first().map_or(0, Vec::len);
        let size_z = data.len();
        debug_assert!(data.iter().all(|row| row.len() == size_x));
        Self {
            size_x,
            size_z,
            data,
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub fn filled_with(size_x: usize, size_z: usize, value: V) -> Self
    where
        V: Clone,
    {
        Self {
            size_x,
            size_z,
            data: vec![vec![value; size_x]; size_z],
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.data.len() == self.size_z && self.data.iter().all(|row| row.len() == self.size_x)
    }

    pub fn map<F, U>(&self, f: F) -> GridXZ<K, U>
    where
        F: Fn(&V) -> U,
    {
        GridXZ::<K, U> {
            size_x:  self.size_x,
            size_z:  self.size_z,
            data:    self
                .data
                .iter()
                .map(|row| row.iter().map(&f).collect())
                .collect(),
            _marker: PhantomData,
        }
    }
}

impl<K, V> GridXZ<K, V>
where
    K: From<CoordsXZ>,
{
    pub fn coords(&self) -> impl Iterator<Item = K> + '_ {
        (0 .. self.size_z)
            .flat_map(move |z| (0 .. self.size_x).map(move |x| CoordsXZ::from_usizes(x, z).into()))
    }
}

impl<K, T> Index<K> for GridXZ<K, T>
where
    K: Into<CoordsXZ>,
{
    type Output = T;

    fn index(&self, coords: K) -> &Self::Output {
        let coords: CoordsXZ = coords.into();
        &self.data[coords.z as usize][coords.x as usize]
    }
}

impl<K, T> IndexMut<K> for GridXZ<K, T>
where
    K: Into<CoordsXZ>,
{
    fn index_mut(&mut self, coords: K) -> &mut Self::Output {
        let coords: CoordsXZ = coords.into();
        &mut self.data[coords.z as usize][coords.x as usize]
    }
}

impl<K: From<CoordsXZ>, T> Debug for GridXZ<K, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GridXZ")
            .field("size_x", &self.size_x)
            .field("size_z", &self.size_z)
            .finish_non_exhaustive()
    }
}
