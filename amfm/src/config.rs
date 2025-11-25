use std::path::PathBuf;

use antenna::stations::Station;
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about=None)]
pub struct Config {
    /// How many songs can be in the queue at any given time
    #[arg(short, long)]
    max_queue_size: Option<usize>,

    /// Where queued songs get stored if the user does not
    /// Save them permenantly

    #[arg(long)]
    temp_song_location: Option<PathBuf>,

    /// Where permenantly saved songs go
    #[arg(long)]
    saved_song_location: Option<PathBuf>,

    /// Start the program already playing some station (URL)
    #[arg(short, long)]
    initial_station: Option<String>,
}

impl Config {
    pub fn station(&self) -> Option<Station> {
        if let Some(url) = &self.initial_station {
            return Some(Station {
                id: url.to_string(),
                url: url.to_string(),
                name: url.to_string(),
                country: "Local".to_string(),
            });
        }

        None
    }
}
