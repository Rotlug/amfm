use antenna::stations::Station;
use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
    widgets::{Row, StatefulWidget, Table, TableState, Widget},
};

pub struct StationsTable<'a> {
    pub stations: Box<dyn Iterator<Item = &'a Station> + 'a>,
    pub state: &'a mut TableState,
}

impl Widget for StationsTable<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let rows = self.stations.map(|s| station_to_row(s));

        let widths = [Constraint::Fill(1), Constraint::Max(30)];

        let table = Table::new(rows, widths)
            .column_spacing(1)
            .header(header())
            .row_highlight_style(Style::new().white().on_green().bold());

        StatefulWidget::render(table, area, buf, self.state);
    }
}

fn header<'a>() -> Row<'a> {
    Row::new(vec!["Name", "Country"]).black().on_white()
}

fn station_to_row(station: &Station) -> Row<'_> {
    Row::new(vec![station.name.as_str(), station.country.as_str()])
}
