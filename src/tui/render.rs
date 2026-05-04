use crate::app::App;
use crate::tui::layout;
use crate::tui::theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

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
}

fn render_title(f: &mut Frame, rects: &layout::AppLayout, _app: &App) {
    let title_line = Line::from(vec![
        Span::styled(
            " engcalc",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  ─────────────────────────────────────────────────────────",
            Style::default().fg(theme::BORDER),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::NONE)
        .padding(Padding::ZERO);

    let para = Paragraph::new(title_line).block(block);
    f.render_widget(para, rects.title_area);
}

fn render_input(f: &mut Frame, rects: &layout::AppLayout, app: &App) {
    let prompt_str = "> ";
    let input_text = Line::from(vec![
        Span::styled(prompt_str, theme::prompt()),
        Span::styled(app.input.content(), theme::bright()),
    ]);

    let input_block = Block::default()
        .title(Line::from(vec![Span::styled(
            " expression ",
            Style::default().fg(theme::DIM),
        )]))
        .borders(Borders::ALL)
        .border_style(theme::border());

    let para = Paragraph::new(input_text).block(input_block);
    f.render_widget(para, rects.input_area);

    let prompt_width = 3u16;
    let cursor_x = rects.input_area.x + 1 + prompt_width + app.input.cursor_pos() as u16;
    let cursor_y = rects.input_area.y + 1;
    if cursor_x < rects.input_area.x + rects.input_area.width - 1 {
        f.set_cursor_position(Position::new(cursor_x, cursor_y));
    }
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
            " VARIABLES ",
            Style::default().fg(theme::DIM),
        )]))
        .borders(Borders::ALL)
        .border_style(theme::border());

    let mut lines = Vec::new();

    let mut vars: Vec<_> = app.user_vars.iter().collect();
    vars.sort_by(|a, b| a.0.cmp(b.0));

    if vars.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  (no variables yet)",
            theme::dim(),
        )]));
        lines.push(Line::from(vec![
            Span::styled("  try: ", theme::dim()),
            Span::styled("x = 42", theme::accent_dim()),
        ]));
    } else {
        let max_name_len = vars.iter().map(|(n, _)| n.len()).max().unwrap_or(4).min(12);
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
    }

    let para = Paragraph::new(Text::from(lines)).block(vars_block);
    f.render_widget(para, rects.vars_area);
}

fn render_footer(f: &mut Frame, rects: &layout::AppLayout) {
    let footer_text = Line::from(vec![
        Span::styled(" Enter ", Style::default().fg(theme::ACCENT)),
        Span::styled("eval  ", theme::DIM),
        Span::styled(" Esc ", Style::default().fg(theme::ACCENT)),
        Span::styled("quit  ", theme::DIM),
        Span::styled(" ↑↓ ", Style::default().fg(theme::ACCENT)),
        Span::styled("history  ", theme::DIM),
        Span::styled(" F1 ", Style::default().fg(theme::ACCENT)),
        Span::styled("help  ", theme::DIM),
        Span::styled(" F2 ", Style::default().fg(theme::ACCENT)),
        Span::styled("consts  ", theme::DIM),
        Span::styled(" Tab ", Style::default().fg(theme::ACCENT)),
        Span::styled("complete", theme::DIM),
    ]);

    let footer = Paragraph::new(footer_text).style(theme::dim());
    f.render_widget(footer, rects.footer_area);
}

fn render_consts_overlay(f: &mut Frame, _app: &App) {
    let area = f.area();
    let overlay_w = 50.min(area.width);
    let overlay_h = 20.min(area.height);
    let overlay_x = (area.width - overlay_w) / 2;
    let overlay_y = (area.height - overlay_h) / 2;
    let overlay = Rect::new(overlay_x, overlay_y, overlay_w, overlay_h);

    let mut lines = Vec::new();
    lines.push(Line::from(vec![Span::styled(
        " Constants",
        Style::default()
            .fg(theme::ACCENT)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    for (name, _desc, val) in crate::core::constants::list() {
        let v = crate::core::value::Value::new(val);
        let display = crate::core::formatter::format_value(&v);
        lines.push(Line::from(vec![
            Span::styled(format!("  {:>4}  ≈ ", name), theme::accent_dim()),
            Span::styled(display, theme::bright()),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "  press F2 to close",
        theme::dim(),
    )]));

    let block = Block::default()
        .title(" constants [F2] ")
        .title_style(theme::accent())
        .borders(Borders::ALL)
        .border_style(theme::accent());

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
            Span::styled("    Quit", theme::bright()),
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
        Line::from(""),
        Line::from(vec![Span::styled("  Commands:", theme::accent_dim())]),
        Line::from(vec![Span::styled(
            "  :help  :clear  :vars  :consts  :history  :quit",
            theme::bright(),
        )]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  press F1 or Esc to close",
            theme::dim(),
        )]),
    ];

    let block = Block::default()
        .title(" help [F1] ")
        .title_style(theme::accent())
        .borders(Borders::ALL)
        .border_style(theme::accent());

    let para = Paragraph::new(Text::from(lines)).block(block);
    f.render_widget(para, overlay);
}
