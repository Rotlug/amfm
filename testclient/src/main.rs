use std::sync::mpsc;

use antenna::playback::PlaybackManager;

fn main() {
    let (tx, rx) = mpsc::channel();
    let mut player = PlaybackManager::new(tx);

    player.set_source_uri("http://15693.live.streamtheworld.com:3690/977_SMOOJAZZ_SC");

    player.play();

    let mut count = 0;
    while let Ok(update) = rx.recv() {
        println!("{update:?}");
        count += 1;
        if count == 3 {
            player.stop();
        }
    }
}
