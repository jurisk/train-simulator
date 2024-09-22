pub struct Profile {
    pub name: String,
    pub height_map_tiff: String,
    pub output_tiles_x: usize,
    pub output_tiles_z: usize,
    pub y_coef: f32,
    pub mountain_compression_coefficient: f32,
    pub mountain_threshold: f32,
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
            },
        ]
    }
}
