use std::sync::mpsc;

use antenna::playback::PlaybackManager;

fn main() {
    let mut player = PlaybackManager::default();

    let (tx, rx) = mpsc::channel();

    player
        .start(
            "http://14543.live.streamtheworld.com/977_SMOOJAZZ_SC",
            Some(tx),
        )
        .expect("failed");

    while let Ok(new_song) = rx.recv() {
        println!("{new_song}");
    }
}
