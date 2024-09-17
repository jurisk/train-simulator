use serde::{Deserialize, Serialize};
use shared_util::bool_ops::BoolResultOps;

use crate::map_level::map_level::Height;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Water {
    between: (Height, Height),
}

impl Water {
    #[must_use]
    pub fn from_below(below: Height) -> Self {
        Self {
            between: (below, below + Height::from_u8(1)),
        }
    }

    #[expect(clippy::missing_errors_doc)]
    pub fn new(below: Height, above: Height) -> Result<Self, String> {
        let result = Self {
            between: (below, above),
        };
        result.is_valid().map(|()| result)
    }

    #[expect(clippy::missing_errors_doc)]
    pub fn is_valid(&self) -> Result<(), String> {
        let (below, above) = &self.between;
        (below.as_u8() + 1 == above.as_u8())
            .then_ok_unit(|| format!("Water height range is invalid: {below:?} - {above:?}",))
    }

    #[must_use]
    pub fn between(&self) -> (Height, Height) {
        self.between
    }

    #[must_use]
    pub fn under_water(&self, height: Height) -> bool {
        let (below, _above) = &self.between;
        height <= *below
    }
}
