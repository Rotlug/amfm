use std::{error::Error, sync::mpsc::Sender, thread};

use gstreamer::{MessageView, prelude::ElementExt};

/// Playback manager that ensures only one playback may play
/// At any given time.
pub struct PlaybackManager {
    current_player: Option<gstreamer::Element>,
}

impl Default for PlaybackManager {
    fn default() -> Self {
        gstreamer::init().ok();
        Self {
            current_player: None,
        }
    }
}

impl PlaybackManager {
    pub fn start(&mut self, url: &str, tx: Option<Sender<String>>) -> Result<(), Box<dyn Error>> {
        // Stop the playback from the old stream
        if let Some(old) = &self.current_player {
            old.set_state(gstreamer::State::Null)?;
        }

        let playbin = gstreamer::ElementFactory::make("playbin")
            .property("uri", url)
            .build()?;

        playbin.set_state(gstreamer::State::Playing)?;

        // Switch the referenced element in current_player to the current playbin
        self.current_player = Some(playbin.clone());

        // If TX is None then we don't care about receiving meta-data, so we can stop this function
        // Right now.
        if tx.is_none() {
            return Ok(());
        }

        let bus = playbin.bus().unwrap();

        // This thread sends metadata updates through tx as long as the stream is active
        thread::spawn(move || {
            let tx = tx.expect("tx should be OK");

            // Variable used for preventing duplicate song titles
            // From being sent
            let mut previous_title = String::new();

            for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
                match msg.view() {
                    MessageView::Eos(..) => {
                        playbin
                            .set_state(gstreamer::State::Null)
                            .expect("Should have been able to stop");
                        break;
                    }
                    MessageView::Error(err) => {
                        eprintln!("Error: {err}:?");
                        playbin
                            .set_state(gstreamer::State::Null)
                            .expect("Should have been able to stop");
                        break;
                    }
                    MessageView::Tag(tag) => {
                        let tags = tag.tags();
                        if let Some(title) = tags.index::<gstreamer::tags::Title>(0) {
                            let title = title.get().to_string();

                            // Ignore duplicate songs
                            if title == previous_title {
                                continue;
                            }

                            previous_title = title.clone();
                            let _ = tx.send(title);
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(old) = &self.current_player {
            old.set_state(gstreamer::State::Null)?;
        }

        self.current_player = None;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_start() {
        let mut mgr = PlaybackManager::default();

        assert!(
            mgr.start("http://hydra.cdnstream.com/1521_128", None)
                .is_ok()
        )
    }

    #[test]
    fn simulate_start_switch_and_stop() {
        let mut mgr = PlaybackManager::default();

        assert!(
            mgr.start("http://hydra.cdnstream.com/1521_128", None)
                .is_ok()
        );
        assert!(
            mgr.start("https://ice1.somafm.com/reggae-256-mp3", None)
                .is_ok()
        );
        assert!(
            mgr.start("https://liveaudio.lamusica.com/MIA_WCMQ_icy", None)
                .is_ok()
        );

        assert!(mgr.stop().is_ok());
    }
}
