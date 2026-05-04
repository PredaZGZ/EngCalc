use ratatui::style::{Color, Style};

pub const TITLE_COLOR: Color = Color::Cyan;
pub const INPUT_PROMPT_COLOR: Color = Color::Yellow;
pub const RESULT_COLOR: Color = Color::Green;
pub const ERROR_COLOR: Color = Color::Red;
pub const HISTORY_HEADER_COLOR: Color = Color::Rgb(150, 150, 200);
pub const VARS_HEADER_COLOR: Color = Color::Rgb(150, 150, 200);
pub const FOOTER_COLOR: Color = Color::DarkGray;
pub const BORDER_COLOR: Color = Color::DarkGray;
pub const DEFAULT_BG: Color = Color::Rgb(30, 30, 46);

pub fn input_style() -> Style {
    Style::default().fg(INPUT_PROMPT_COLOR)
}

pub fn result_style() -> Style {
    Style::default().fg(RESULT_COLOR)
}

pub fn error_style() -> Style {
    Style::default().fg(ERROR_COLOR)
}

pub fn footer_style() -> Style {
    Style::default().fg(FOOTER_COLOR)
}

pub fn border_style() -> Style {
    Style::default().fg(BORDER_COLOR)
}

pub fn title_style() -> Style {
    Style::default().fg(TITLE_COLOR)
}
