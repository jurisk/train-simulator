use std::fs;
use std::fs::File;

use game_level_generator::height_map;
use game_level_generator::profile::Profile;
use game_level_generator::source::GeoTiffSource;
use game_level_generator::zonings;
use geotiff::GeoTiff;
use log::info;
use shared_domain::map_level::map_level::MapLevel;
use shared_domain::map_level::zoning::Zoning;
use shared_domain::scenario::Scenario;
use shared_domain::{MapId, ScenarioId};

fn generate_scenario(
    profile: &Profile,
    relative_path: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    info!("Converting profile {}", profile.name);
    let path = format!("{relative_path}{}", profile.height_map_tiff);
    let file = File::open(&path).map_err(|e| format!("Failed to open {path}: {e}"))?;
    let tiff = GeoTiff::read(file)?;
    info!("GeoTIFF {tiff:?}");
    let source = GeoTiffSource::new(tiff);
    let (terrain, water) = height_map::convert(profile, &source)?;
    let mut map_level = MapLevel::new(
        MapId(profile.name.clone()),
        terrain,
        water,
        Zoning::new(profile.output_tiles_x, profile.output_tiles_z),
    );
    zonings::augment(&mut map_level, profile);
    let scenario_id = ScenarioId(profile.name.clone());
    let players = profile.players.clone();
    let scenario = Scenario {
        scenario_id,
        players,
        map_level,
    };
    let serialized = scenario.save_to_bytes()?;
    info!("Serialized map level to {} bytes", serialized.len());
    Ok(serialized)
}

fn process_and_save(profile: &Profile) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = generate_scenario(profile, "")?;
    let output_path = format!("assets/scenarios/{}.bincode.gz", profile.name);
    fs::write(&output_path, serialized)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for profile in Profile::all() {
        process_and_save(&profile)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Too slow
    fn test_scenarios_can_be_generated() {
        for profile in Profile::all() {
            let result = generate_scenario(&profile, "../../");
            match result {
                Ok(_) => {},
                Err(e) => {
                    panic!("Failed to generate scenario for {}: {e}", profile.name);
                },
            }
        }
    }
}
