use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Widget, Wrap};

use crate::utils::center_horizontal;

const RECORDING_TEXT: &str = "REC";
const NOT_RECORDING_TEXT: &str = "IDLE";

pub struct RadioInfo<'a> {
    pub name: &'a str,
    pub current_song: &'a str,
    pub is_recording: bool,
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
        let current_song = Paragraph::new(self.current_song)
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
            recording = recording.white().on_red()
        } else {
            recording = recording.black().on_white()
        }

        let [name_area, current_song_area, recording_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(name.line_count(area.width) as u16),
                Constraint::Length(current_song.line_count(area.width) as u16),
                Constraint::Length(1),
            ],
        )
        .areas(area);

        name.render(name_area, buf);
        current_song.render(current_song_area, buf);
        recording.render(
            center_horizontal(recording_area, rec_text.len() as u16),
            buf,
        );
    }
}
