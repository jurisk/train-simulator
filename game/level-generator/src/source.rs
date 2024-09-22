use geotiff::GeoTiff;

pub trait Source {
    fn pixel_at(&self, x: usize, y: usize) -> f32;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

#[expect(clippy::module_name_repetitions)]
pub struct GeoTiffSource {
    tiff: GeoTiff,
}

impl GeoTiffSource {
    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    pub fn new(tiff: GeoTiff) -> Self {
        assert_eq!(tiff.num_samples, 1);
        Self { tiff }
    }
}

impl Source for GeoTiffSource {
    fn pixel_at(&self, x: usize, y: usize) -> f32 {
        self.tiff.get_value_at(x, y, 0)
    }

    fn width(&self) -> usize {
        self.tiff.raster_width
    }

    fn height(&self) -> usize {
        self.tiff.raster_height
    }
}
