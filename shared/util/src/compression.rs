use std::io::Read;
use std::io::Write;

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[expect(clippy::missing_errors_doc)]
pub fn load_from_bytes<T: DeserializeOwned>(data: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;

    let result: T = bincode::deserialize(&decompressed_data)?;

    Ok(result)
}

#[expect(clippy::missing_errors_doc)]
pub fn save_to_bytes<T: Serialize>(object: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let serialized_data = bincode::serialize(object)?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&serialized_data)?;
    let compressed_data = encoder.finish()?;

    Ok(compressed_data)
}
