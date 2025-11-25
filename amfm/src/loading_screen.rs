use ratatui::prelude::*;
use ratatui::widgets::{Gauge, Paragraph, Widget};

use crate::utils::{center_vertical, margins};

pub struct LoadingScreen {
    pub percentage: u64,
}

impl Widget for LoadingScreen {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let gague = Gauge::default()
            .percent(self.percentage as u16)
            .gauge_style(Style::default().magenta());

        let title = Paragraph::new("Fetching stations...")
            .italic()
            .alignment(Alignment::Center);

        let subtitle = Paragraph::new("(this will only happen once)")
            .italic()
            .alignment(Alignment::Center)
            .dim();

        let area = margins(area, 90);
        let area = center_vertical(area, 3);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);

        title.render(layout[0], buf);
        gague.render(layout[1], buf);
        subtitle.render(layout[2], buf);
    }
}
