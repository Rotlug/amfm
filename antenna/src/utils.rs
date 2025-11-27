use std::{fs, path::PathBuf};

pub fn get_cache_dir() -> PathBuf {
    let dir = dirs::cache_dir()
        .expect("Cache path should always exist")
        .join("amfm");

    // Create dir if it doesn't exist
    if !fs::exists(dir.as_path()).expect("Couldn't check if cache directory exists") {
        fs::create_dir(dir.as_path()).expect("Creating directory has failed");
    }

    dir
}
