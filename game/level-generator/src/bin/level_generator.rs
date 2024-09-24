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

fn convert_profile(profile: &Profile) -> Result<(), Box<dyn std::error::Error>> {
    info!("Converting profile {}", profile.name);
    let file = File::open(&profile.height_map_tiff)?;
    let tiff = GeoTiff::read(file)?;
    info!("GeoTIFF {tiff:?}");
    let source = GeoTiffSource::new(tiff);
    let (terrain, water) = height_map::convert(profile, &source)?;
    let mut map_level = MapLevel::new(
        terrain,
        water,
        Zoning::new(profile.output_tiles_x, profile.output_tiles_z),
    );
    zonings::augment(&mut map_level);
    let serialized = bincode::serialize(&map_level)?;
    info!("Serialized map level to {} bytes", serialized.len());
    let output_path = format!("assets/map_levels/{}.bincode", profile.name);
    fs::write(&output_path, serialized)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for profile in Profile::all() {
        convert_profile(&profile)?;
    }

    Ok(())
}
