use antenna::{playback::PlaybackManager, stations::Station};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout},
    widgets::{Block, Borders, Paragraph, Widget},
};

use ratatui::prelude::*;

use crate::radio_info::RadioInfo;

pub struct MainScreen<'a> {
    pub playback: &'a PlaybackManager,
    pub current_station: Option<&'a Station>,
    pub current_title: &'a str,
}

impl Widget for MainScreen<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [main_area, sidebar_area] = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(70), Constraint::Percentage(30)],
        )
        .areas(area);

        let main = Block::new().borders(Borders::all()).title_top("Main Area");

        let sidebar = Block::new().borders(Borders::all()).title_top("Sidebar");

        let sidebar_area_inner = sidebar.inner(sidebar_area);

        main.render(main_area, buf);
        sidebar.render(sidebar_area, buf);

        let [radio_info_area, _] =
            Layout::vertical([Constraint::Min(1), Constraint::Fill(1)]).areas(sidebar_area_inner);

        let radio_info = RadioInfo {
            name: &self.current_station.unwrap().name, // FIXME Proper "Nothing Playing"" State needed
            current_song: self.current_title,
            is_recording: self.playback.is_recording(),
        };

        radio_info.render(radio_info_area, buf);
    }
}
