use std::{env, fs, path::PathBuf};

use antenna::stations::Station;
use clap::Parser;

fn get_default_temp_dir() -> PathBuf {
    let dir = env::temp_dir().join("amfm");
    fs::create_dir_all(&dir).expect("Could not create temp directory!"); // Ensure path exists

    dir
}

fn get_default_save_directory() -> PathBuf {
    let dir = dirs::audio_dir()
        .unwrap_or_else(|| env::current_dir().expect("Current dir not found!"))
        .join("amfm");

    fs::create_dir_all(&dir).expect("Could not create saved dir!");

    dir
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct Config {
    /// How many songs can be in the queue at any given time
    #[arg(short, long, default_value_t = 10)]
    pub max_queue_size: usize,

    /// Where queued songs get stored if the user does not
    /// Save them permenantly

    #[arg(long, default_value = get_default_temp_dir().into_os_string())]
    pub temp_song_location: PathBuf,

    /// Where permenantly saved songs go
    #[arg(long, default_value = get_default_save_directory().into_os_string())]
    pub saved_song_location: PathBuf,

    /// Start the program already playing some station (URL)
    #[arg(short, long)]
    pub initial_station: Option<String>,
}

impl Config {
    pub fn station(&self) -> Option<Station> {
        if let Some(url) = &self.initial_station {
            return Some(Station {
                id: url.to_string(),
                url: url.to_string(),
                name: url.to_string(),
                country: "Local".to_string(),
                votes: 0,
            });
        }

        None
    }
}
