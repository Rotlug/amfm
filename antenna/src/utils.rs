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

pub fn get_temp_dir() -> PathBuf {
    std::env::temp_dir()
}

pub fn get_music_directory() -> PathBuf {
    dirs::audio_dir().expect("Audio directory should exist")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_dir() {
        get_cache_dir();

        assert!(fs::exists(dirs::cache_dir().unwrap().join("amfm")).unwrap());

        // Check again to see that it doesn't crash when attempting
        // To create the directory twice
        get_cache_dir();
    }

    #[test]
    fn music_dir() {
        get_music_directory();
    }
}
