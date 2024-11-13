use serde::{Deserialize, Serialize};

use crate::map_level::map_level::MapLevel;
use crate::server_response::{Colour, PlayerInfo};
use crate::tile_coords_xz::TileCoordsXZ;
use crate::{PlayerId, PlayerName, ScenarioId};

pub const EUROPE_SCENARIO_BINCODE: &[u8] =
    include_bytes!("../../../assets/scenarios/europe.bincode.gz");
// TODO: Have a full USA map and use that
pub const USA_SCENARIO_BINCODE: &[u8] =
    include_bytes!("../../../assets/scenarios/usa_east.bincode.gz");

#[derive(Serialize, Deserialize, Clone)]
pub struct Scenario {
    pub scenario_id: ScenarioId,
    pub players:     Vec<PlayerProfile>,
    pub map_level:   MapLevel,
}

impl Scenario {
    #[expect(clippy::missing_errors_doc)]
    pub fn is_valid(&self) -> Result<(), String> {
        self.map_level.is_valid()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use shared_util::compression;

    use super::*;

    #[test]
    fn test_scenarios_can_be_deserialised() {
        let scenarios = [USA_SCENARIO_BINCODE, EUROPE_SCENARIO_BINCODE];
        for scenario in scenarios {
            let read: Scenario = compression::load_from_bytes(scenario).unwrap();
            assert!(read.is_valid().is_ok());
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub player_id:                 PlayerId,
    pub player_name:               PlayerName,
    pub player_colour:             Colour,
    pub initial_construction_yard: TileCoordsXZ,
}

impl PlayerProfile {
    #[must_use]
    pub fn new(
        player_id: PlayerId,
        player_name: PlayerName,
        player_colour: Colour,
        initial_construction_yard: TileCoordsXZ,
    ) -> Self {
        Self {
            player_id,
            player_name,
            player_colour,
            initial_construction_yard,
        }
    }

    #[must_use]
    pub fn to_player_info(&self) -> PlayerInfo {
        PlayerInfo {
            id:     self.player_id,
            name:   self.player_name.clone(),
            colour: self.player_colour,
        }
    }
}
