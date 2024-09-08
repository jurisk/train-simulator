use serde::{Deserialize, Serialize};

use crate::map_level::map_level::Height;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Water {
    between: (Height, Height),
}

impl Water {
    pub fn new(below: Height, above: Height) -> Result<Self, String> {
        let result = Self {
            between: (below, above),
        };
        result.is_valid().map(|_| result)
    }

    #[expect(clippy::missing_errors_doc)]
    pub fn is_valid(&self) -> Result<(), String> {
        let (below, above) = &self.between;
        if below.as_u8() + 1 == above.as_u8() {
            Ok(())
        } else {
            Err(format!(
                "Water height range is invalid: {below:?} - {above:?}",
            ))
        }
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
