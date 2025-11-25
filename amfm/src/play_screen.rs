use antenna::{playback::PlaybackManager, stations::Station};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListState, Paragraph, Widget},
};

use ratatui::prelude::*;

use crate::{FocusRegion, radio_info::RadioInfo, song_queue::SongQueue, utils::center_vertical};

pub struct PlayScreen<'a> {
    pub playback: &'a PlaybackManager,
    pub current_station: Option<Station>,
    pub current_title: &'a str,

    pub queue: &'a SongQueue,
    pub queue_list_state: &'a mut ListState,

    pub focus: &'a FocusRegion,
}

impl Widget for PlayScreen<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // Areas
        let [main_area, sidebar_area] = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(70), Constraint::Percentage(30)],
        )
        .areas(area);

        // Blocks
        let mut main = Block::new().borders(Borders::all()).title_top("Main Area");
        if *self.focus != FocusRegion::MainArea {
            main = main.border_style(Style::new().dim())
        }

        main.render(main_area, buf);

        let mut radio_info_block = Block::new().borders(Borders::all()).title_top("Info");
        if *self.focus != FocusRegion::RadioInfo {
            radio_info_block = radio_info_block.border_style(Style::new().dim())
        }
        let mut queue_block = Block::new().borders(Borders::all()).title_top("Queue");
        if *self.focus != FocusRegion::Queue {
            queue_block = queue_block.border_style(Style::new().dim())
        }

        let [radio_info_area, queue_area] =
            Layout::vertical([Constraint::Max(10), Constraint::Fill(1)]).areas(sidebar_area);

        // Radio info
        if let Some(station) = self.current_station {
            let radio_info = RadioInfo {
                name: &station.name,
                current_song: self.current_title,
                is_recording: self.playback.is_recording(),
            };

            radio_info.render(radio_info_block.inner(radio_info_area), buf);
        } else {
            let nothing_playing = Paragraph::new("Nothing is playing")
                .alignment(Alignment::Center)
                .dim()
                .italic();

            let height = nothing_playing.line_count(radio_info_area.width) as u16;

            nothing_playing.render(center_vertical(radio_info_area, height), buf);
        }

        radio_info_block.render(radio_info_area, buf);

        // Queue
        let queue_list = List::new(self.queue.iter().map(|s| s.title.clone()))
            .highlight_style(Style::new().black().on_white());

        StatefulWidget::render(
            queue_list,
            queue_block.inner(queue_area),
            buf,
            self.queue_list_state,
        );

        queue_block.render(queue_area, buf);
    }
}
