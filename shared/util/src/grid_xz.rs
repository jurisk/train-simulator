#![allow(clippy::cast_sign_loss)]

use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};

use crate::bool_ops::BoolResultOps;
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

    #[expect(clippy::missing_errors_doc)]
    pub fn is_valid(&self) -> Result<(), String> {
        if self.data.len() == self.size_z {
            self.data
                .iter()
                .all(|row| row.len() == self.size_x)
                .then_ok_unit(|| "GridXZ size_x mismatch".to_string())
        } else {
            Err(format!(
                "GridXZ size_z mismatch: expected {}, got {}",
                self.size_z,
                self.data.len()
            ))
        }
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

    pub fn map_with_coords<F, U>(&self, f: F) -> GridXZ<K, U>
    where
        F: Fn(CoordsXZ, &V) -> U,
        K: From<CoordsXZ>,
    {
        GridXZ::<K, U> {
            size_x:  self.size_x,
            size_z:  self.size_z,
            data:    self
                .data
                .iter()
                .enumerate()
                .map(|(z, row)| {
                    row.iter()
                        .enumerate()
                        .map(|(x, v)| f(CoordsXZ::from_usizes(x, z), v))
                        .collect()
                })
                .collect(),
            _marker: PhantomData,
        }
    }

    pub fn values_into_iter(self) -> impl Iterator<Item = V> {
        self.data.into_iter().flat_map(IntoIterator::into_iter)
    }

    pub fn values_iter(&self) -> impl Iterator<Item = &V> {
        self.data.iter().flat_map(|row| row.iter())
    }
}

impl<K, V> GridXZ<K, V>
where
    K: Into<CoordsXZ>,
    V: Copy + Default,
{
    pub fn get_or_default(&self, k: K) -> V {
        self.get(k).copied().unwrap_or_else(|| V::default())
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

impl<K, V> GridXZ<K, V>
where
    K: Into<CoordsXZ>,
{
    #[must_use]
    pub fn get(&self, k: K) -> Option<&V> {
        let coords: CoordsXZ = k.into();
        self.data
            .get(coords.z as usize)
            .and_then(|row| row.get(coords.x as usize))
    }

    pub fn get_mut(&mut self, k: K) -> Option<&mut V> {
        let coords: CoordsXZ = k.into();
        self.data
            .get_mut(coords.z as usize)
            .and_then(|row| row.get_mut(coords.x as usize))
    }

    #[must_use]
    #[expect(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    pub fn in_bounds(&self, k: K) -> bool {
        let coords: CoordsXZ = k.into();
        coords.x < self.size_x as i32
            && coords.z < self.size_z as i32
            && coords.x >= 0
            && coords.z >= 0
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
