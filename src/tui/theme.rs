use ratatui::style::{Color, Modifier, Style};

pub const ACCENT: Color = Color::Cyan;
pub const ACCENT_DIM: Color = Color::Rgb(100, 180, 255);
pub const RESULT: Color = Color::Green;
pub const ERROR: Color = Color::Rgb(255, 100, 100);
pub const PROMPT: Color = Color::Rgb(250, 200, 100);
pub const DIM: Color = Color::Rgb(90, 90, 120);
pub const BRIGHT: Color = Color::White;
pub const BORDER: Color = Color::Rgb(60, 60, 90);

pub fn accent() -> Style {
    Style::default().fg(ACCENT)
}

pub fn accent_dim() -> Style {
    Style::default().fg(ACCENT_DIM)
}

pub fn result() -> Style {
    Style::default().fg(RESULT).add_modifier(Modifier::BOLD)
}

pub fn error() -> Style {
    Style::default().fg(ERROR)
}

pub fn prompt() -> Style {
    Style::default().fg(PROMPT)
}

pub fn dim() -> Style {
    Style::default().fg(DIM)
}

pub fn bright() -> Style {
    Style::default().fg(BRIGHT)
}

pub fn border() -> Style {
    Style::default().fg(BORDER)
}
