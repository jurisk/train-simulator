use std::io::{Read, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
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
    pub fn load_from_bytes(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;

        let scenario: Self = bincode::deserialize(&decompressed_data)?;
        scenario.is_valid()?;

        Ok(scenario)
    }

    #[expect(clippy::missing_errors_doc)]
    pub fn save_to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let serialized_data = bincode::serialize(self)?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&serialized_data)?;
        let compressed_data = encoder.finish()?;

        Ok(compressed_data)
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
            assert!(Scenario::load_from_bytes(scenario).is_ok());
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
