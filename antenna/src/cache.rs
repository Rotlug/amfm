use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    sync::mpsc::{self, Receiver},
    thread,
};

use crate::{
    stations::Station,
    utils::{self},
};

const STATIONS_URL: &str = "http://37.27.202.89/json/stations";

/// Error type that is returned from cache-related operations - Encoding, decoding, fetching from the server, io and such..
#[derive(Debug)]
pub enum CacheError {
    NetworkError(reqwest::Error),
    IoError(io::Error),
    BinEncodeError(bincode::error::EncodeError),
    BinDecodeError(bincode::error::DecodeError),
    JsonDecodeError(serde_json::Error),
}

type CacheResultHandle = thread::JoinHandle<Result<Vec<Station>, CacheError>>;

/// Download and store the json file containing all stations.
/// Then, read and convert said json file into a .bin file for faster loading times
///
/// Returns 2 values:
/// 1: Receiver - use to listen to percentage updates in the download
/// 2: JoinHandle - use to get Error and block until download completion
pub fn make_cache() -> (Receiver<u64>, CacheResultHandle) {
    let (tx, rx) = mpsc::channel();

    let handle: CacheResultHandle = thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(STATIONS_URL)
            .send()
            .map_err(CacheError::NetworkError)?;

        let total_size = response
            .content_length()
            .filter(|&len| len > 0)
            .unwrap_or(60_000_000);

        let mut source = BufReader::new(response);
        let mut buffer = [0u8; 8192];

        let mut file = File::create(utils::get_cache_dir().join("stations.json"))
            .map_err(CacheError::IoError)?;

        let mut downloaded = 0;

        loop {
            let bytes_read = match source.read(&mut buffer) {
                Ok(0) => break, // 0 Indicates end of file
                Err(err) => return Err(CacheError::IoError(err)),
                Ok(n) => n,
            };

            if let Err(err) = file.write_all(&buffer[..bytes_read]) {
                return Err(CacheError::IoError(err));
            }

            downloaded += bytes_read as u64;
            if tx.send(downloaded * 100 / total_size).is_err() {
                break;
            }
        }

        let result = create_bin()?;
        Ok(result)
    });

    (rx, handle)
}

/// Creates a binary version of the json file for faster access
fn create_bin() -> Result<Vec<Station>, CacheError> {
    let file =
        File::open(utils::get_cache_dir().join("stations.json")).map_err(CacheError::IoError)?;

    let reader = BufReader::new(file);

    let data: Vec<Station> =
        serde_json::from_reader(reader).map_err(CacheError::JsonDecodeError)?;

    let mut output_file =
        File::create(utils::get_cache_dir().join("stations.bin")).map_err(CacheError::IoError)?;

    bincode::serde::encode_into_std_write(&data, &mut output_file, bincode::config::standard())
        .map_err(CacheError::BinEncodeError)?;

    Ok(data)
}

/// Reads bin file and outputs list of stations
pub fn read_bin_cache() -> Result<Vec<Station>, CacheError> {
    let file =
        File::open(utils::get_cache_dir().join("stations.bin")).map_err(CacheError::IoError)?;
    let mut reader = BufReader::new(file);

    let data: Vec<Station> =
        bincode::serde::decode_from_reader(&mut reader, bincode::config::standard())
            .map_err(CacheError::BinDecodeError)?;

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read_from_cache() {
        let cache_path = utils::get_cache_dir().join("stations.bin");

        // Only run if the file exists
        if !cache_path.exists() {
            eprintln!(
                "Skipping can_read_from_cache: no cache file at {:?}",
                cache_path
            );
            return;
        }

        let data = read_bin_cache().expect("Failed to read existing cache");
        assert!(
            data.len() > 50_000,
            "Expected >50,000 stations, got {}",
            data.len()
        );
    }
}
