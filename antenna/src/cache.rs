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

pub type CacheResultHandle = thread::JoinHandle<Result<Vec<Station>, CacheError>>;

#[derive(Debug)]
pub struct CacheResult {
    /// This reciever is used to get new loading percentage data
    pub rx: Receiver<u64>,
    /// This handle is used to block until loading completion and check for errors
    pub handle: CacheResultHandle,
}

/// Download and store the json file containing all stations.
/// Then, read and convert said json file into a .bin file for faster loading times
pub fn make_cache() -> CacheResult {
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

    CacheResult { rx, handle }
}

/// Creates a binary version of the json file for faster access
fn create_bin() -> Result<Vec<Station>, CacheError> {
    let file =
        File::open(utils::get_cache_dir().join("stations.json")).map_err(CacheError::IoError)?;

    let reader = BufReader::new(file);

    let mut data: Vec<Station> =
        serde_json::from_reader(reader).map_err(CacheError::JsonDecodeError)?;

    // Trim all station names
    for station in &mut data {
        station.name = station.name.trim().to_string();
    }

    // Sort from most votes to least
    data.sort_by(|a, b| b.cmp(a));

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
