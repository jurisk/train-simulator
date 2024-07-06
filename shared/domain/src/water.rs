use serde::{Deserialize, Serialize};

use crate::map_level::Height;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Water {
    between: (Height, Height),
}

impl Water {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let (below, above) = &self.between;
        below.as_u8() + 1 == above.as_u8()
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
