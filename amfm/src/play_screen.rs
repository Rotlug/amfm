use antenna::stations::StationList;
use ratatui::{
    prelude::*,
    text::ToSpan,
    widgets::{Block, Borders, List, Paragraph},
};

use crate::{
    AppModel, FocusRegion, radio_info::RadioInfo, stations_table::StationsTable,
    utils::center_vertical,
};

pub struct PlayScreen<'a> {
    pub model: &'a mut AppModel,
    pub table_size: usize,
}

impl Widget for PlayScreen<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let stations_iter = self
            .model
            .stations
            .search(self.model.stations_search.value())
            .take(self.table_size);

        // Areas
        let [main_area, sidebar_area] = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(70), Constraint::Percentage(30)],
        )
        .areas(area);

        let [main_area, search_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Fill(1),
                Constraint::Length(if self.model.search_toggled { 1 } else { 0 }),
            ],
        )
        .areas(main_area);

        // Blocks
        let mut main = Block::new().borders(Borders::all()).title_top("Stations");
        if self.model.focus != FocusRegion::MainArea {
            main = main.border_style(Style::new().dim())
        }

        let mut radio_info_block = Block::new().borders(Borders::all()).title_top("Info");
        if self.model.focus != FocusRegion::RadioInfo {
            radio_info_block = radio_info_block.border_style(Style::new().dim())
        }
        let mut queue_block = Block::new().borders(Borders::all()).title_top("Queue");
        if self.model.focus != FocusRegion::Queue {
            queue_block = queue_block.border_style(Style::new().dim())
        }

        let [radio_info_area, queue_area] =
            Layout::vertical([Constraint::Max(10), Constraint::Fill(1)]).areas(sidebar_area);

        // Main Area
        let table = StationsTable {
            stations: Box::new(stations_iter),
            state: &mut self.model.stations_table_state,
        };

        table.render(main.inner(main_area), buf);
        main.render(main_area, buf);

        // Radio info
        if let Some(station) = &self.model.current_station {
            let radio_info = RadioInfo {
                name: &station.name,
                current_song: &self.model.current_title,
                is_recording: self.model.playback.is_recording(),
                last_update: &self.model.last_update,
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
        let queue_list = List::new(self.model.queue.iter().enumerate().map(|s| {
            if s.0 == 0 {
                s.1.title.clone().dim().italic()
            } else {
                s.1.title.clone().not_dim()
            }
        }))
        .highlight_style(Style::new().black().on_white());

        StatefulWidget::render(
            queue_list,
            queue_block.inner(queue_area),
            buf,
            &mut self.model.queue_list_state,
        );

        queue_block.render(queue_area, buf);

        // Search bar
        let text_input = Paragraph::new(self.model.stations_search.value()).cyan();
        text_input.render(search_area, buf);
    }
}
