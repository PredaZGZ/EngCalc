use crate::app::App;
use crate::tui::layout;
use crate::tui::theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph};

pub fn render(f: &mut Frame, app: &mut App) {
    let rects = layout::build_layout(f.area());

    render_title(f, &rects, app);
    render_input(f, &rects, app);
    render_result(f, &rects, app);
    render_history(f, &rects, app);
    render_vars(f, &rects, app);
    render_footer(f, &rects);

    if app.show_consts {
        render_consts_overlay(f, app);
    }
    if app.show_help {
        render_help_overlay(f, app);
    }
    if app.show_functions {
        render_functions_overlay(f, app);
    }
}

fn render_title(f: &mut Frame, _rects: &layout::AppLayout, _app: &App) {
    let title_line = Line::from(vec![
        Span::styled(
            " engcalc",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" v", theme::DIM),
        Span::styled(env!("CARGO_PKG_VERSION"), theme::DIM),
    ]);

    let para = Paragraph::new(title_line);
    f.render_widget(para, _rects.title_area);
}

fn render_input(f: &mut Frame, rects: &layout::AppLayout, app: &App) {
    let content = app.input.content();
    let cursor = app.input.cursor_pos();

    let prompt_str = "> ";
    let mut spans = vec![Span::styled(prompt_str, theme::prompt())];

    // Tokenize input to colorize constants and variables
    let tokens = tokenize_input(&content, &app.user_vars);
    for (token, is_const, is_var) in &tokens {
        let style = if *is_const {
            Style::default().fg(Color::Rgb(255, 105, 180))
        } else if *is_var {
            Style::default().fg(theme::RESULT)
        } else {
            theme::bright()
        };
        spans.push(Span::styled(token.clone(), style));
    }

    let input_text = Line::from(spans);

    let input_block = Block::default()
        .title(Line::from(vec![Span::styled(
            " expression ",
            Style::default().fg(theme::DIM),
        )]))
        .borders(Borders::ALL)
        .border_style(theme::border());

    let para = Paragraph::new(input_text).block(input_block);
    f.render_widget(para, rects.input_area);

    let prompt_width = 2u16;
    let cursor_x = rects.input_area.x + 1 + prompt_width + cursor as u16;
    let cursor_y = rects.input_area.y + 1;
    if cursor_x < rects.input_area.x + rects.input_area.width - 1 {
        f.set_cursor_position(Position::new(cursor_x, cursor_y));
    }
}

/// Tokenize input into words and non-word segments.
/// Returns (text, is_constant, is_variable).
fn tokenize_input<'a>(
    input: &'a str,
    vars: &std::collections::HashMap<String, crate::core::value::Value>,
) -> Vec<(String, bool, bool)> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_word = false;

    for ch in input.chars() {
        let is_word_char = ch.is_alphanumeric() || ch == '_';
        if is_word_char != in_word {
            if !current.is_empty() {
                let (is_const, is_var) = classify_word(&current, vars);
                tokens.push((current.clone(), is_const, is_var));
                current.clear();
            }
            in_word = is_word_char;
        }
        current.push(ch);
    }
    if !current.is_empty() {
        let (is_const, is_var) = classify_word(&current, vars);
        tokens.push((current, is_const, is_var));
    }

    tokens
}

fn classify_word(
    word: &str,
    vars: &std::collections::HashMap<String, crate::core::value::Value>,
) -> (bool, bool) {
    for c in crate::core::constants::list() {
        if c.name == word {
            return (true, false);
        }
    }
    if vars.contains_key(word) {
        return (false, true);
    }
    (false, false)
}

fn render_result(f: &mut Frame, rects: &layout::AppLayout, app: &App) {
    let result_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border());

    let mut lines = Vec::new();

    if let Some(ref err) = app.last_error {
        lines.push(Line::from(vec![
            Span::styled("  ✗ ", theme::error()),
            Span::styled(err.clone(), theme::error()),
        ]));
    } else if let Some(ref val) = app.last_result {
        lines.push(Line::from(vec![
            Span::styled(
                "  = ",
                Style::default()
                    .fg(theme::RESULT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(val.clone(), theme::result()),
        ]));
    } else {
        lines.push(Line::from(vec![Span::styled(
            "  (result will appear here)",
            theme::dim(),
        )]));
    }

    let para = Paragraph::new(Text::from(lines)).block(result_block);
    f.render_widget(para, rects.result_area);
}

fn render_history(f: &mut Frame, rects: &layout::AppLayout, app: &App) {
    let history_block = Block::default()
        .title(Line::from(vec![Span::styled(
            " HISTORY ",
            Style::default().fg(theme::DIM),
        )]))
        .borders(Borders::ALL)
        .border_style(theme::border());

    let mut lines = Vec::new();

    let entries = app.history.last_n(30);
    for entry in entries.iter().rev() {
        if entry.is_error {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} = ", entry.expression),
                    Style::default().fg(theme::DIM),
                ),
                Span::styled(&entry.result, theme::error()),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} = ", entry.expression),
                    Style::default().fg(theme::DIM),
                ),
                Span::styled(&entry.result, theme::result()),
            ]));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  (no history yet)",
            theme::dim(),
        )]));
    }

    let para = Paragraph::new(Text::from(lines)).block(history_block);
    f.render_widget(para, rects.history_area);
}

fn render_vars(f: &mut Frame, rects: &layout::AppLayout, app: &App) {
    let vars_block = Block::default()
        .title(Line::from(vec![Span::styled(
            " VARIABLES & FUNCTIONS ",
            Style::default().fg(theme::DIM),
        )]))
        .borders(Borders::ALL)
        .border_style(theme::border());

    let mut lines = Vec::new();

    // Collect and sort variables
    let mut vars: Vec<_> = app.user_vars.iter().collect();
    vars.sort_by(|a, b| a.0.cmp(b.0));

    // Collect and sort functions
    let mut funcs: Vec<_> = app.env.iter_functions().collect();
    funcs.sort_by(|a, b| a.0.cmp(b.0));

    if vars.is_empty() && funcs.is_empty() {
        lines.push(Line::from(vec![Span::styled("  (empty)", theme::dim())]));
    } else {
        // Show variables first
        let max_name_len = vars
            .iter()
            .map(|(n, _)| n.len())
            .chain(funcs.iter().map(|(n, _)| n.len()))
            .max()
            .unwrap_or(4)
            .min(12);

        for (name, value) in &vars {
            let display = crate::core::formatter::format_value(value);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<width$} = ", name, width = max_name_len),
                    theme::accent_dim(),
                ),
                Span::styled(display, theme::bright()),
            ]));
        }

        // Show functions
        for (name, func) in &funcs {
            let params = func.params.join(", ");
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<width$} ", name, width = max_name_len),
                    Style::default().fg(Color::Rgb(255, 105, 180)), // Pink for functions
                ),
                Span::styled(format!("({})", params), theme::dim()),
            ]));
        }
    }

    let para = Paragraph::new(Text::from(lines)).block(vars_block);
    f.render_widget(para, rects.vars_area);
}

fn render_footer(f: &mut Frame, rects: &layout::AppLayout) {
    let footer_text = Line::from(vec![
        Span::styled(" Enter ", Style::default().fg(theme::ACCENT)),
        Span::styled("eval  ", theme::DIM),
        Span::styled(" F1 ", Style::default().fg(theme::ACCENT)),
        Span::styled("help  ", theme::DIM),
        Span::styled(" F2 ", Style::default().fg(theme::ACCENT)),
        Span::styled("consts  ", theme::DIM),
        Span::styled(" F4 ", Style::default().fg(theme::ACCENT)),
        Span::styled("funcs  ", theme::DIM),
        Span::styled(" Esc ", Style::default().fg(theme::ACCENT)),
        Span::styled("clear", theme::DIM),
    ]);

    let footer = Paragraph::new(footer_text).style(theme::dim());
    f.render_widget(footer, rects.footer_area);
}

fn render_consts_overlay(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let overlay_w = 60.min(area.width);
    let overlay_h = 22.min(area.height);
    let overlay_x = (area.width - overlay_w) / 2;
    let overlay_y = (area.height - overlay_h) / 2;
    let overlay = Rect::new(overlay_x, overlay_y, overlay_w, overlay_h);

    let filtered = crate::core::constants::search(&app.consts_search);
    let visible_count = 16usize;

    let mut lines = Vec::new();

    // Search bar
    let search_bar = Line::from(vec![
        Span::styled(" / ", theme::accent()),
        Span::styled(&app.consts_search, theme::bright()),
        if app.consts_search.is_empty() {
            Span::styled("type to filter...", theme::dim())
        } else {
            Span::default()
        },
    ]);
    lines.push(search_bar);
    lines.push(Line::from(""));

    // Results
    if filtered.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  (no results)",
            theme::dim(),
        )]));
    } else {
        // Calculate window start based on selection
        let total = filtered.len();
        let selected = app.consts_selected;
        let window_start = if total <= visible_count {
            0
        } else if selected < visible_count / 2 {
            0
        } else if selected > total - visible_count / 2 {
            total - visible_count
        } else {
            selected - visible_count / 2
        };

        let window_end = (window_start + visible_count).min(total);

        // Show scroll indicator if needed
        if window_start > 0 {
            lines.push(Line::from(vec![Span::styled("  ▲ ...", theme::dim())]));
        }

        for i in window_start..window_end {
            let c = &filtered[i];
            let is_selected = i == selected;
            let v = crate::core::value::Value::new(c.value);
            let display = crate::core::formatter::format_value(&v);
            let arrow = if is_selected { "▸ " } else { "  " };

            if is_selected {
                lines.push(Line::from(vec![
                    Span::styled(arrow, theme::accent()),
                    Span::styled(
                        format!("{:<5} ", c.name),
                        theme::accent().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(display, theme::bright()),
                    Span::styled("  ", theme::DIM),
                    Span::styled(c.description, theme::dim()),
                    Span::styled(format!("  [{}]", c.units), theme::DIM),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(arrow, theme::dim()),
                    Span::styled(format!("{:<5} ", c.name), theme::accent_dim()),
                    Span::styled(display, Style::default().fg(Color::Gray)),
                    Span::styled("  ", theme::DIM),
                    Span::styled(c.description, theme::dim()),
                    Span::styled(format!("  [{}]", c.units), theme::DIM),
                ]));
            }
        }

        if window_end < total {
            lines.push(Line::from(vec![Span::styled(
                format!("  ▼ ... {} more", total - window_end),
                theme::dim(),
            )]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" Enter", theme::accent()),
        Span::styled(" insert  ", theme::dim()),
        Span::styled(" ↑↓", theme::accent()),
        Span::styled(" nav  ", theme::dim()),
        Span::styled(" Esc", theme::accent()),
        Span::styled(" close  ", theme::dim()),
        Span::styled(" /", theme::accent()),
        Span::styled(" search", theme::dim()),
    ]));

    // Clear the area behind the overlay
    f.render_widget(Clear, overlay);

    let block = Block::default()
        .title(" constants [F2] ")
        .title_style(theme::accent())
        .borders(Borders::ALL)
        .border_style(theme::accent())
        .style(Style::default().bg(Color::Black));

    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, overlay);
}

fn render_functions_overlay(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let overlay_w = 60.min(area.width);
    let overlay_h = 24.min(area.height);
    let overlay_x = (area.width - overlay_w) / 2;
    let overlay_y = (area.height - overlay_h) / 2;
    let overlay = Rect::new(overlay_x, overlay_y, overlay_w, overlay_h);

    let filtered = app.filtered_functions();
    let visible_count = 16usize; // Number of items visible at once

    let mut lines = Vec::new();

    // Search bar
    let search_bar = Line::from(vec![
        Span::styled(" / ", theme::accent()),
        Span::styled(&app.funcs_search, theme::bright()),
        if app.funcs_search.is_empty() {
            Span::styled("type to filter...", theme::dim())
        } else {
            Span::default()
        },
    ]);
    lines.push(search_bar);
    lines.push(Line::from(""));

    // Results
    if filtered.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  (no results)",
            theme::dim(),
        )]));
    } else {
        // Calculate window start based on selection
        let total = filtered.len();
        let selected = app.funcs_selected;
        let window_start = if total <= visible_count {
            0
        } else if selected < visible_count / 2 {
            0
        } else if selected > total - visible_count / 2 {
            total - visible_count
        } else {
            selected - visible_count / 2
        };

        let window_end = (window_start + visible_count).min(total);

        // Show scroll indicator if needed
        if window_start > 0 {
            lines.push(Line::from(vec![Span::styled("  ▲ ...", theme::dim())]));
        }

        for i in window_start..window_end {
            let func = &filtered[i];
            let is_selected = i == selected;
            let arrow = if is_selected { "▸ " } else { "  " };

            if is_selected {
                lines.push(Line::from(vec![
                    Span::styled(arrow, theme::accent()),
                    Span::styled(
                        format!("{:<8}", func.name),
                        Style::default()
                            .fg(Color::Rgb(255, 105, 180))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("({})", func.params), theme::dim()),
                    Span::styled("  — ", theme::dim()),
                    Span::styled(func.description, theme::dim()),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(arrow, theme::dim()),
                    Span::styled(
                        format!("{:<8}", func.name),
                        Style::default().fg(Color::Rgb(255, 105, 180)),
                    ),
                    Span::styled(format!("({})", func.params), theme::dim()),
                    Span::styled("  — ", theme::dim()),
                    Span::styled(func.description, theme::dim()),
                ]));
            }
        }

        if window_end < total {
            lines.push(Line::from(vec![Span::styled(
                format!("  ▼ ... {} more", total - window_end),
                theme::dim(),
            )]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" Enter", theme::accent()),
        Span::styled(" insert  ", theme::dim()),
        Span::styled(" ↑↓", theme::accent()),
        Span::styled(" nav  ", theme::dim()),
        Span::styled(" Esc", theme::accent()),
        Span::styled(" close  ", theme::dim()),
        Span::styled(" /", theme::accent()),
        Span::styled(" search", theme::dim()),
    ]));

    // Clear the area behind the overlay
    f.render_widget(Clear, overlay);

    let block = Block::default()
        .title(" functions [F4] ")
        .title_style(theme::accent())
        .borders(Borders::ALL)
        .border_style(theme::accent())
        .style(Style::default().bg(Color::Black));

    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, overlay);
}

fn render_help_overlay(f: &mut Frame, _app: &App) {
    let area = f.area();
    let overlay_w = 56.min(area.width);
    let overlay_h = 30.min(area.height);
    let overlay_x = (area.width - overlay_w) / 2;
    let overlay_y = (area.height - overlay_h) / 2;
    let overlay = Rect::new(overlay_x, overlay_y, overlay_w, overlay_h);

    let lines = vec![
        Line::from(vec![Span::styled(
            " Help",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter", theme::accent_dim()),
            Span::styled("  Evaluate expression", theme::bright()),
        ]),
        Line::from(vec![
            Span::styled("  Esc", theme::accent_dim()),
            Span::styled("    Clear input", theme::bright()),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+C", theme::accent_dim()),
            Span::styled(" Quit", theme::bright()),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+L", theme::accent_dim()),
            Span::styled(" Clear screen", theme::bright()),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+U", theme::accent_dim()),
            Span::styled(" Clear input", theme::bright()),
        ]),
        Line::from(vec![
            Span::styled("  ↑/↓", theme::accent_dim()),
            Span::styled("    History navigation", theme::bright()),
        ]),
        Line::from(vec![
            Span::styled("  Tab", theme::accent_dim()),
            Span::styled("    Autocomplete", theme::bright()),
        ]),
        Line::from(vec![
            Span::styled("  ←/→", theme::accent_dim()),
            Span::styled("  Move cursor", theme::bright()),
        ]),
        Line::from(vec![
            Span::styled("  Backspace", theme::accent_dim()),
            Span::styled(" Delete before", theme::bright()),
        ]),
        Line::from(vec![
            Span::styled("  Delete", theme::accent_dim()),
            Span::styled("  Delete after", theme::bright()),
        ]),
        Line::from(vec![
            Span::styled("  F1", theme::accent_dim()),
            Span::styled("     This help", theme::bright()),
        ]),
        Line::from(vec![
            Span::styled("  F2", theme::accent_dim()),
            Span::styled("     Show constants", theme::bright()),
        ]),
        Line::from(vec![
            Span::styled("  F3", theme::accent_dim()),
            Span::styled("     Reset (clear all)", theme::bright()),
        ]),
        Line::from(vec![
            Span::styled("  F4", theme::accent_dim()),
            Span::styled("     Show functions", theme::bright()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("  Commands:", theme::accent_dim())]),
        Line::from(vec![Span::styled(
            "  :help  :clear  :vars  :consts  :history  :quit",
            theme::bright(),
        )]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled("  press F1 to close", theme::dim())]),
    ];

    // Clear the area behind the overlay
    f.render_widget(Clear, overlay);

    let block = Block::default()
        .title(" help [F1] ")
        .title_style(theme::accent())
        .borders(Borders::ALL)
        .border_style(theme::accent())
        .style(Style::default().bg(Color::Black));

    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, overlay);
}
