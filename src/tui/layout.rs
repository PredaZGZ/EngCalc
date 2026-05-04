use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub title_area: Rect,
    pub input_area: Rect,
    pub result_area: Rect,
    pub history_area: Rect,
    pub vars_area: Rect,
    pub footer_area: Rect,
}

pub fn build_layout(area: Rect) -> AppLayout {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Length(3), // input
            Constraint::Length(3), // result
            Constraint::Min(5),    // main: history + vars
            Constraint::Length(2), // footer
        ])
        .split(area);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[3]);

    AppLayout {
        title_area: chunks[0],
        input_area: chunks[1],
        result_area: chunks[2],
        history_area: main_chunks[0],
        vars_area: main_chunks[1],
        footer_area: chunks[4],
    }
}
