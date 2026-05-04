use crate::app::App;
use crate::tui::layout;
use crate::tui::theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(f: &mut Frame, app: &mut App) {
    let rects = layout::build_layout(f.area());

    render_title(f, &rects, app);
    render_input(f, &rects, app);
    render_result(f, &rects, app);
    render_history(f, &rects, app);
    render_vars(f, &rects, app);
    render_footer(f, &rects);
}

fn render_title(f: &mut Frame, rects: &layout::AppLayout, app: &App) {
    let mode_label = match app.mode {
        crate::app::AppMode::Normal => "normal",
        crate::app::AppMode::Command => "command",
        crate::app::AppMode::Help => "help",
    };

    let title_text = format!(" engcalc v{} [{}] ", env!("CARGO_PKG_VERSION"), mode_label);
    let title = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .title(title_text)
        .title_style(theme::title_style());

    let para = Paragraph::new("").block(title);
    f.render_widget(para, rects.title_area);
}

fn render_input(f: &mut Frame, rects: &layout::AppLayout, app: &App) {
    let input_text = if app.mode == crate::app::AppMode::Command {
        format!("> {}", app.input.content)
    } else {
        format!("> {}", app.input.content)
    };

    let input_block = Block::default()
        .title(" expression ")
        .title_style(theme::input_style())
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    let para = Paragraph::new(input_text)
        .block(input_block)
        .style(theme::input_style());

    f.render_widget(para, rects.input_area);

    let cursor_x = rects.input_area.x + 2 + app.input.cursor_pos as u16;
    let cursor_y = rects.input_area.y + 1;
    f.set_cursor_position(Position::new(cursor_x, cursor_y));
}

fn render_result(f: &mut Frame, rects: &layout::AppLayout, app: &App) {
    let result_block = Block::default()
        .title(" result ")
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    let text = if let Some(ref err) = app.last_error {
        let mut t = Text::default();
        let span = Span::styled(
            format!(" Error: {}", err),
            Style::default().fg(theme::ERROR_COLOR),
        );
        t.extend([Line::from(vec![span])]);
        t
    } else if let Some(ref val) = app.last_result {
        let mut t = Text::default();
        let span = Span::styled(val, Style::default().fg(theme::RESULT_COLOR));
        t.extend([Line::from(vec![span])]);
        t
    } else {
        Text::default()
    };

    let para = Paragraph::new(text).block(result_block);
    f.render_widget(para, rects.result_area);
}

fn render_history(f: &mut Frame, rects: &layout::AppLayout, app: &App) {
    let history_block = Block::default()
        .title(" history ")
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    let mut lines = Vec::new();

    let entries = app.history.last_n(20);
    for entry in entries.iter().rev() {
        let line = if entry.is_error {
            Line::from(vec![
                Span::styled(
                    format!("{} = ", entry.expression),
                    Style::default().fg(Color::White),
                ),
                Span::styled(&entry.result, Style::default().fg(theme::ERROR_COLOR)),
            ])
        } else {
            Line::from(vec![
                Span::styled(
                    format!("{} = ", entry.expression),
                    Style::default().fg(Color::White),
                ),
                Span::styled(&entry.result, Style::default().fg(theme::RESULT_COLOR)),
            ])
        };
        lines.push(line);
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "(no history yet)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let para = Paragraph::new(Text::from(lines)).block(history_block);
    f.render_widget(para, rects.history_area);
}

fn render_vars(f: &mut Frame, rects: &layout::AppLayout, app: &App) {
    let vars_block = Block::default()
        .title(" variables / constants ")
        .borders(Borders::ALL)
        .border_style(theme::border_style());

    let mut lines = Vec::new();

    for (name, value) in app.env.iter() {
        let display = crate::core::formatter::format_value(value);
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<6} = ", name),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(display, Style::default().fg(Color::White)),
        ]));
    }

    let const_names = crate::core::constants::list();
    for (name, _desc, val) in &const_names {
        let display = crate::core::formatter::format_value(&crate::core::value::Value::new(*val));
        lines.push(Line::from(vec![
            Span::styled(format!("{:<6} ≈ ", name), Style::default().fg(Color::Cyan)),
            Span::styled(display, Style::default().fg(Color::White)),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "(no variables)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let para = Paragraph::new(Text::from(lines)).block(vars_block);
    f.render_widget(para, rects.vars_area);
}

fn render_footer(f: &mut Frame, rects: &layout::AppLayout) {
    let footer_text =
        " Enter=eval · Ctrl+L=clear · ↑/↓=history · Tab=complete · Esc=quit · :=command";
    let footer = Paragraph::new(footer_text)
        .style(theme::footer_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme::border_style()),
        );
    f.render_widget(footer, rects.footer_area);
}
