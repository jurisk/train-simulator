use shared_domain::scenario::PlayerProfile;
use shared_domain::server_response::Colour;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::{PlayerId, PlayerName};

pub struct Profile {
    pub name: String,
    pub height_map_tiff: String,
    pub output_tiles_x: usize,
    pub output_tiles_z: usize,
    pub y_coef: f32,
    pub mountain_compression_coefficient: f32,
    pub mountain_threshold: f32,
    pub players: Vec<PlayerProfile>,
}

impl Profile {
    #[must_use]
    pub fn all() -> Vec<Profile> {
        vec![
            Profile {
                name: "europe".to_string(),
                height_map_tiff: "misc/assets-original/height-maps/europe/europe_etopo_2022.tiff"
                    .to_string(),
                output_tiles_x: 256,
                output_tiles_z: 192,
                y_coef: 0.5,
                mountain_compression_coefficient: 500.0,
                mountain_threshold: 2000.0,
                players: vec![
                    PlayerProfile::new(
                        PlayerId::random(),
                        PlayerName::new("West".to_string()),
                        Colour::rgb(153, 51, 255),
                        TileCoordsXZ::new(60, 150),
                    ),
                    PlayerProfile::new(
                        PlayerId::random(),
                        PlayerName::new("East".to_string()),
                        Colour::rgb(255, 51, 51),
                        TileCoordsXZ::new(220, 85),
                    ),
                ],
            },
            Profile {
                name: "usa_east".to_string(),
                height_map_tiff: "misc/assets-original/height-maps/usa/usa_east_etopo_2022.tiff"
                    .to_string(),
                output_tiles_x: 256,
                output_tiles_z: 192,
                y_coef: 0.5,
                mountain_compression_coefficient: 400.0,
                mountain_threshold: 1200.0,
                players: vec![
                    PlayerProfile::new(
                        PlayerId::random(),
                        PlayerName::new("Union".to_string()),
                        Colour::rgb(153, 51, 255),
                        TileCoordsXZ::new(20, 20),
                    ),
                    PlayerProfile::new(
                        PlayerId::random(),
                        PlayerName::new("Alliance".to_string()),
                        Colour::rgb(255, 51, 51),
                        TileCoordsXZ::new(150, 135),
                    ),
                ],
            },
        ]
    }
}
