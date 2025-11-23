use ratatui::layout::{Constraint, Flex, Layout, Rect};

pub fn margins(area: Rect, remaining_percentage: u16) -> Rect {
    let [area] = Layout::horizontal([Constraint::Percentage(remaining_percentage)])
        .flex(Flex::Center)
        .areas(area);

    area
}

pub fn center_vertical(area: Rect, height: u16) -> Rect {
    let [area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);

    area
}

pub fn center_horizontal(area: Rect, width: u16) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    area
}
