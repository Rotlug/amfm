use antenna::stations::StationList;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, Paragraph, Wrap},
};

use crate::{
    AppModel, FocusRegion, radio_info::RadioInfo, shortcuts_display::ShortcutsDisplay,
    stations_table::StationsTable, utils::center_vertical,
};

pub struct PlayScreen<'a> {
    pub model: &'a mut AppModel,
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
            .skip(self.model.table_virtual_offset)
            .take(self.model.table_size.into());

        // Areas
        let [full_area, shortcuts_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        let [main_area, sidebar_area] = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(70), Constraint::Percentage(30)],
        )
        .areas(full_area);

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
            focused: self.model.focus == FocusRegion::MainArea,
        };

        table.render(main.inner(main_area), buf);
        main.render(main_area, buf);

        let radio_info_area_inner = radio_info_block.inner(radio_info_area);

        // Radio info
        if let Some(station) = &self.model.current_station {
            let radio_info = RadioInfo {
                name: &station.name,
                current_song: self.model.queue.last(),
                is_recording: self.model.playback.is_recording(),
                last_update: &self.model.last_update,
            };

            radio_info.render(radio_info_area_inner, buf);
        } else {
            let nothing_playing = Paragraph::new("Nothing is playing")
                .alignment(Alignment::Center)
                .dim()
                .italic()
                .wrap(Wrap { trim: true });

            let height = nothing_playing.line_count(radio_info_area_inner.width) as u16;

            nothing_playing.render(
                center_vertical(radio_info_block.inner(radio_info_area_inner), height),
                buf,
            );
        }

        radio_info_block.render(radio_info_area, buf);

        // Queue
        let queue_list = List::new(self.model.queue.iter().enumerate().map(|(i, item)| {
            let title: &str = &item.tags.title;

            if i == 0 {
                Span::styled(title, Style::default().dim().italic())
            } else {
                Span::raw(title)
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

        // Shortcuts
        let shortcuts = ShortcutsDisplay {};
        shortcuts.render(shortcuts_area, buf);
    }
}
