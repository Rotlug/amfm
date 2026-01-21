use antenna::playback::PlaybackUpdate;
use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Widget, Wrap};

use crate::song_queue::Song;
use crate::utils::center_horizontal;

const RECORDING_TEXT: &str = "REC";
const NOT_RECORDING_TEXT: &str = "IDLE";

pub struct RadioInfo<'a> {
    pub name: &'a str,
    pub current_song: Option<&'a Song>,
    pub is_recording: bool,
    pub last_update: &'a PlaybackUpdate,
}

impl Widget for RadioInfo<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // Station Name
        let name = Paragraph::new(self.name)
            .alignment(Alignment::Center)
            .bold()
            .wrap(Wrap { trim: true });

        // Current song title
        let displayed_name = if let Some(song) = self.current_song {
            &song.to_string()
        } else {
            ""
        };

        let current_song = Paragraph::new(displayed_name)
            .alignment(Alignment::Center)
            .italic()
            .dim()
            .wrap(Wrap { trim: true });

        // Recording Indicator
        let rec_text = if self.is_recording {
            RECORDING_TEXT
        } else {
            NOT_RECORDING_TEXT
        };

        let mut recording = Paragraph::new(rec_text);
        if self.is_recording {
            recording = recording.white().on_red();
        } else {
            recording = recording.black().on_white();
        }

        // Last Update
        let last_update = match self.last_update {
            PlaybackUpdate::Loading => Paragraph::new("Loading...").dim().italic().centered(),
            PlaybackUpdate::Stopped => Paragraph::new("Stopped.").dim().italic().centered(),
            PlaybackUpdate::Error(msg) => Paragraph::new(msg.as_str())
                .red()
                .wrap(Wrap { trim: true })
                .centered(),
            _ => Paragraph::new(""),
        };

        let [
            name_area,
            current_song_area,
            recording_area,
            last_update_area,
        ] = Layout::vertical([
            Constraint::Length(name.line_count(area.width) as u16),
            Constraint::Length(current_song.line_count(area.width) as u16),
            Constraint::Length(1),
            Constraint::Length(last_update.line_count(area.width) as u16),
        ])
        .areas(area);

        name.render(name_area, buf);
        current_song.render(current_song_area, buf);
        recording.render(
            center_horizontal(recording_area, rec_text.len() as u16),
            buf,
        );
        last_update.render(last_update_area, buf);
    }
}
