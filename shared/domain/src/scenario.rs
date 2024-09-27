use serde::{Deserialize, Serialize};

use crate::ScenarioId;
use crate::map_level::map_level::MapLevel;
use crate::server_response::PlayerInfo;

pub const EUROPE_SCENARIO_BINCODE: &[u8] =
    include_bytes!("../../../assets/scenarios/europe.bincode");
// TODO: Have a full USA map and use that
pub const USA_SCENARIO_BINCODE: &[u8] =
    include_bytes!("../../../assets/scenarios/usa_east.bincode");

#[derive(Serialize, Deserialize, Clone)]
pub struct Scenario {
    pub scenario_id:  ScenarioId,
    pub player_infos: Vec<PlayerInfo>,
    pub map_level:    MapLevel,
}

impl Scenario {
    #[expect(clippy::missing_errors_doc)]
    pub fn load_bincode(bincode: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let scenario: Self = bincode::deserialize(bincode)?;
        match scenario.is_valid() {
            Ok(()) => Ok(scenario),
            Err(err) => Err(err.into()),
        }
    }

    #[expect(clippy::missing_errors_doc)]
    pub fn is_valid(&self) -> Result<(), String> {
        self.map_level.is_valid()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenarios_can_be_deserialised() {
        let scenarios = [USA_SCENARIO_BINCODE, EUROPE_SCENARIO_BINCODE];
        for scenario in scenarios {
            assert!(Scenario::load_bincode(scenario).is_ok());
        }
    }
}
