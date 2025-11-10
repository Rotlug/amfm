use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use gstreamer::prelude::ElementExt;

/// Playback manager that ensures only one playback may play
/// At any given time.
pub struct PlaybackManager {
    current_player: Arc<Mutex<Option<gstreamer::Element>>>,
}

impl Default for PlaybackManager {
    fn default() -> Self {
        gstreamer::init().ok();
        Self {
            current_player: Arc::new(Mutex::new(None)),
        }
    }
}

impl PlaybackManager {
    pub fn start(&self, url: &str) -> Result<(), Box<dyn Error>> {
        // Stop the playback from the old stream
        if let Some(old) = self.current_player.lock().unwrap().take() {
            old.set_state(gstreamer::State::Null)?;
        }

        let playbin = gstreamer::ElementFactory::make("playbin")
            .property("uri", url)
            .build()?;

        playbin.set_state(gstreamer::State::Playing)?;

        // Switch the referenced element in current_player to the current playbin
        *self.current_player.lock().unwrap() = Some(playbin);

        Ok(())
    }

    pub fn stop(&self) -> Result<(), Box<dyn Error>> {
        if let Some(old) = self.current_player.lock().unwrap().take() {
            old.set_state(gstreamer::State::Null)?;
        }

        *self.current_player.lock().unwrap() = None;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_start() {
        let mgr = PlaybackManager::default();

        assert!(mgr.start("http://hydra.cdnstream.com/1521_128").is_ok())
    }

    #[test]
    fn simulate_start_switch_and_stop() {
        let mgr = PlaybackManager::default();

        assert!(mgr.start("http://hydra.cdnstream.com/1521_128").is_ok());
        assert!(mgr.start("https://ice1.somafm.com/reggae-256-mp3").is_ok());
        assert!(
            mgr.start("https://liveaudio.lamusica.com/MIA_WCMQ_icy")
                .is_ok()
        );

        assert!(mgr.stop().is_ok());
    }
}
