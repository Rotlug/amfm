use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    widgets::{Paragraph, Widget},
};

pub struct ShortcutsDisplay {}

impl Widget for ShortcutsDisplay {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let shortcuts = [
            shortcut("/", "Search"),
            shortcut("s", "Stop playback"),
            shortcut("q", "Quit"),
            shortcut("y", "Copy URL"),
        ];

        let constraints = shortcuts.iter().map(|s| Constraint::Length(s.0 as u16));

        let areas = Layout::horizontal(constraints).spacing(1).split(area);

        for (area, shortcut) in areas.iter().zip(shortcuts.map(|s| s.1)) {
            shortcut.render(*area, buf);
        }
    }
}

fn shortcut<'a>(key: &str, action: &str) -> (usize, Paragraph<'a>) {
    let text = format!("[{key}] {action}");
    (text.len(), Paragraph::new(text).light_magenta())
}
