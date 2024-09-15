use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct CargoAmount(f32);

impl CargoAmount {
    pub const ZERO: Self = Self(0.0);

    #[must_use]
    pub fn new(amount: f32) -> Self {
        Self(amount)
    }

    #[must_use]
    pub fn as_f32(self) -> f32 {
        self.0
    }
}

impl Debug for CargoAmount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl Sub for CargoAmount {
    type Output = CargoAmount;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Add for CargoAmount {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Mul<f32> for CargoAmount {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Div for CargoAmount {
    type Output = f32;

    fn div(self, rhs: Self) -> Self::Output {
        self.0 / rhs.0
    }
}

impl Neg for CargoAmount {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl AddAssign for CargoAmount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Eq for CargoAmount {}

impl PartialEq<Self> for CargoAmount {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd<Self> for CargoAmount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[expect(clippy::unwrap_used)]
impl Ord for CargoAmount {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl MulAssign<f32> for CargoAmount {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
    }
}
