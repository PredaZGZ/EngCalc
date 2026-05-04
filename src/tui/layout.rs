use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub title_area: Rect,
    pub input_area: Rect,
    pub result_area: Rect,
    #[allow(dead_code)]
    pub main_area: Rect,
    pub history_area: Rect,
    pub vars_area: Rect,
    pub footer_area: Rect,
}

pub fn build_layout(area: Rect) -> AppLayout {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(1),
        ])
        .split(area);

    let side = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(main[3]);

    AppLayout {
        title_area: main[0],
        input_area: main[1],
        result_area: main[2],
        main_area: main[3],
        history_area: side[0],
        vars_area: side[1],
        footer_area: main[4],
    }
}
