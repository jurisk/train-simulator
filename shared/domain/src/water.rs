use serde::{Deserialize, Serialize};

use crate::map_level::Height;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Water {
    pub between: (Height, Height),
}

impl Water {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let (below, above) = &self.between;
        below.0 + 1 == above.0
    }

    #[must_use]
    pub fn under_water(&self, height: Height) -> bool {
        let (below, _above) = &self.between;
        height <= *below
    }
}
