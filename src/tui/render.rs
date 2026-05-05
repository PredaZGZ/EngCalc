use crate::app::App;
use crate::tui::layout;
use crate::tui::theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

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

    // Tokenize input to colorize with syntax validation
    let tokens = tokenize_input_with_validation(&content, &app.user_vars, &app.env);
    
    // Track cursor position adjustment for Greek letter conversion
    let mut visual_cursor_pos = cursor as i32;
    let mut current_pos = 0;
    
    for (token, token_type) in &tokens {
        let style = match token_type {
            TokenType::Constant => Style::default().fg(Color::Rgb(255, 105, 180)),
            TokenType::Variable => Style::default().fg(theme::RESULT),
            TokenType::Function => Style::default().fg(Color::Rgb(100, 200, 255)),
            TokenType::Operator => Style::default().fg(Color::Rgb(200, 200, 200)),
            TokenType::Number => Style::default().fg(Color::Rgb(180, 220, 150)),
            TokenType::Valid => theme::bright(),
        };
        // Convert Greek names to symbols for display
        let display_token = if let Some(symbol) = crate::core::greek::name_to_symbol(token) {
            // Adjust cursor position if it's after this token
            let token_end = current_pos + token.len();
            if cursor > current_pos && cursor <= token_end {
                // Cursor is within this token, adjust relative to symbol
                let offset = token.len() - 1; // 1 char for the symbol
                visual_cursor_pos -= offset as i32;
            } else if cursor > token_end {
                // Cursor is after this token
                let offset = token.len() - 1;
                visual_cursor_pos -= offset as i32;
            }
            symbol.to_string()
        } else {
            token.clone()
        };
        current_pos += token.len();
        spans.push(Span::styled(display_token, style));
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
    let cursor_x = rects.input_area.x + 1 + prompt_width + visual_cursor_pos.max(0) as u16;
    let cursor_y = rects.input_area.y + 1;
    if cursor_x < rects.input_area.x + rects.input_area.width - 1 {
        f.set_cursor_position(Position::new(cursor_x, cursor_y));
    }

    // Render autocomplete popup if visible (takes priority)
    if app.show_autocomplete && !app.autocomplete_suggestions.is_empty() {
        render_autocomplete_popup(f, app, cursor_x, cursor_y);
    }
    // Only show signature help if autocomplete is NOT visible
    else if app.show_signature_help {
        render_signature_help(f, app, cursor_x, cursor_y);
    }
}

fn render_autocomplete_popup(f: &mut Frame, app: &App, cursor_x: u16, cursor_y: u16) {
    let suggestions = &app.autocomplete_suggestions;
    let selected = app.autocomplete_selected;

    if suggestions.is_empty() {
        return;
    }

    // Build simple text lines
    let mut content = String::new();
    let visible_count = 4usize;
    let total = suggestions.len();
    
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

    for i in window_start..window_end {
        let suggestion = &suggestions[i];
        // Split to get just the signature part before '|'
        let display = suggestion.split('|').next().unwrap_or(suggestion);
        
        if i == selected {
            content.push_str(&format!("> {}\n", display));
        } else {
            content.push_str(&format!("  {}\n", display));
        }
    }

    if total > visible_count {
        content.push_str(&format!("  ... {} more\n", total - window_end));
    }

    // Fixed dimensions
    let popup_width = 35u16;
    let popup_height = suggestions.len().min(visible_count) as u16 + 2;

    // Position popup
    let area = f.area();
    let popup_x = cursor_x.min(area.width.saturating_sub(popup_width));
    let popup_y = if cursor_y + popup_height > area.height {
        cursor_y.saturating_sub(popup_height + 1)
    } else {
        cursor_y + 1
    };

    let popup_rect = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear and render
    f.render_widget(Clear, popup_rect);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::accent())
        .style(Style::default().bg(Color::Rgb(40, 40, 50)));

    let para = Paragraph::new(content).block(block);
    f.render_widget(para, popup_rect);
}

fn render_signature_help(f: &mut Frame, app: &App, cursor_x: u16, cursor_y: u16) {
    let func_name = match &app.signature_help_func {
        Some(name) => name,
        None => return,
    };

    // Get function info
    let func_info = match crate::core::functions::get_function_info(func_name) {
        Some(info) => info,
        None => return,
    };

    // Check if params_detail is empty
    if func_info.params_detail.is_empty() {
        return;
    }

    let param_index = app.signature_help_param_index.min(func_info.params_detail.len().saturating_sub(1));

    // Build the signature line
    let mut signature = format!("{}(", func_info.name);
    for (i, param) in func_info.params_detail.iter().enumerate() {
        if i > 0 {
            signature.push_str(", ");
        }
        if i == param_index {
            signature.push_str(&format!("[{}]", param.name));
        } else {
            signature.push_str(param.name);
        }
    }
    signature.push(')');

    // Get current parameter info
    let current_param = func_info.params_detail.get(param_index)
        .map(|p| format!("{}: {}", p.name, p.description))
        .unwrap_or_default();

    // Build simple text content
    let content = format!(
        "{}\n\n{}\nparam {}/{}",
        signature,
        current_param,
        param_index + 1,
        func_info.params_detail.len()
    );

    // Calculate dimensions
    let width = 50u16;
    let height = 5u16;

    // Position popup below the cursor
    let area = f.area();
    let popup_x = cursor_x.min(area.width.saturating_sub(width));
    let popup_y = if cursor_y + height > area.height {
        cursor_y.saturating_sub(height + 1)
    } else {
        cursor_y + 1
    };

    let popup_rect = Rect::new(popup_x, popup_y, width, height);

    // Clear area
    f.render_widget(Clear, popup_rect);

    // Create block with content
    let block = Block::default()
        .title(" Help ")
        .title_style(theme::accent())
        .borders(Borders::ALL)
        .border_style(theme::accent())
        .style(Style::default().bg(Color::Rgb(30, 30, 40)));

    let para = Paragraph::new(content).block(block);
    f.render_widget(para, popup_rect);
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenType {
    Valid,
    Constant,
    Variable,
    Function,
    Operator,
    Number,
}

/// Tokenize input with syntax validation for real-time error highlighting
fn tokenize_input_with_validation(
    input: &str,
    vars: &std::collections::HashMap<String, crate::core::value::Value>,
    env: &crate::core::env::Environment,
) -> Vec<(String, TokenType)> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_word = false;

    for ch in input.chars() {
        let is_word_char = ch.is_alphanumeric() || ch == '_';
        if is_word_char != in_word {
            if !current.is_empty() {
                let token_type = classify_token(&current, vars, env);
                tokens.push((current.clone(), token_type));
                current.clear();
            }
            in_word = is_word_char;
        }
        current.push(ch);
    }
    if !current.is_empty() {
        let token_type = classify_token(&current, vars, env);
        tokens.push((current, token_type));
    }

    tokens
}

fn classify_token(
    token: &str,
    vars: &std::collections::HashMap<String, crate::core::value::Value>,
    env: &crate::core::env::Environment,
) -> TokenType {
    // Check if it's a number
    if token.chars().all(|c| c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '-' || c == '+')
        && token.chars().any(|c| c.is_ascii_digit())
    {
        return TokenType::Number;
    }

    // Check if it's an operator
    if token.chars().all(|c| "+-*/^%=<>!&|~".contains(c)) {
        return TokenType::Operator;
    }

    // Check parentheses and brackets - always valid
    if token.chars().all(|c| "()[]{}".contains(c)) {
        return TokenType::Valid;
    }

    // Check constants
    for c in crate::core::constants::list() {
        if c.name == token {
            return TokenType::Constant;
        }
    }

    // Check built-in functions
    if crate::core::functions::is_function(token) {
        return TokenType::Function;
    }

    // Check user-defined functions
    if env.get_function(token).is_some() {
        return TokenType::Function;
    }

    // Check variables
    if vars.contains_key(token) {
        return TokenType::Variable;
    }

    // Check if it's a unit
    if crate::core::units::is_valid_unit(token) {
        return TokenType::Valid;
    }

    // Single character identifiers might be parameters (valid in context)
    if token.len() == 1 && token.chars().next().unwrap().is_ascii_lowercase() {
        return TokenType::Variable; // Likely a parameter like 'x'
    }

    // Check if it's a Greek letter name (will be rendered as symbol)
    if crate::core::greek::is_greek_name(token) {
        return TokenType::Variable;
    }

    TokenType::Valid
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
        // Convert Greek names to symbols in expression
        let display_expr = crate::core::greek::replace_greek_names(&entry.expression);
        if entry.is_error {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} = ", display_expr),
                    Style::default().fg(theme::DIM),
                ),
                Span::styled(&entry.result, theme::error()),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} = ", display_expr),
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
    // Make overlay wider to accommodate details panel
    let overlay_w = 80.min(area.width);
    let overlay_h = 26.min(area.height);
    let overlay_x = (area.width - overlay_w) / 2;
    let overlay_y = (area.height - overlay_h) / 2;
    let overlay = Rect::new(overlay_x, overlay_y, overlay_w, overlay_h);

    // Clear the area behind the overlay
    f.render_widget(Clear, overlay);

    // Split overlay into two columns: function list and details
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(overlay);

    let filtered = app.filtered_functions();
    let visible_count = 14usize;

    // Left column: Function list
    let mut list_lines = Vec::new();

    // Search bar
    let search_bar = Line::from(vec![
        Span::styled(" / ", theme::accent()),
        Span::styled(&app.funcs_search, theme::bright()),
        if app.funcs_search.is_empty() {
            Span::styled("filter...", theme::dim())
        } else {
            Span::default()
        },
    ]);
    list_lines.push(search_bar);
    list_lines.push(Line::from(""));

    // Results
    if filtered.is_empty() {
        list_lines.push(Line::from(vec![Span::styled(
            "  (no results)",
            theme::dim(),
        )]));
    } else {
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

        if window_start > 0 {
            list_lines.push(Line::from(vec![Span::styled("  ▲ ...", theme::dim())]));
        }

        for i in window_start..window_end {
            let func = &filtered[i];
            let is_selected = i == selected;
            let arrow = if is_selected { "▸ " } else { "  " };

            if is_selected {
                list_lines.push(Line::from(vec![
                    Span::styled(arrow, theme::accent()),
                    Span::styled(
                        format!("{}", func.name),
                        Style::default()
                            .fg(Color::Rgb(255, 105, 180))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("({})", func.params), theme::dim()),
                ]));
            } else {
                list_lines.push(Line::from(vec![
                    Span::styled(arrow, theme::dim()),
                    Span::styled(
                        format!("{}", func.name),
                        Style::default().fg(Color::Rgb(255, 105, 180)),
                    ),
                    Span::styled(format!("({})", func.params), theme::dim()),
                ]));
            }
        }

        if window_end < total {
            list_lines.push(Line::from(vec![Span::styled(
                format!("  ▼ ... {} more", total - window_end),
                theme::dim(),
            )]));
        }
    }

    // Left block
    let left_block = Block::default()
        .title(" Functions [F4] ")
        .title_style(theme::accent())
        .borders(Borders::ALL)
        .border_style(theme::accent())
        .style(Style::default().bg(Color::Black));

    let left_para = Paragraph::new(Text::from(list_lines)).block(left_block);
    f.render_widget(left_para, columns[0]);

    // Right column: Function details
    let mut detail_lines = Vec::new();

    if !filtered.is_empty() && app.funcs_selected < filtered.len() {
        let func = &filtered[app.funcs_selected];

        // Function name and category
        detail_lines.push(Line::from(vec![
            Span::styled(func.name, Style::default().fg(Color::Rgb(255, 105, 180)).add_modifier(Modifier::BOLD)),
        ]));
        detail_lines.push(Line::from(vec![
            Span::styled(format!("Category: "), theme::dim()),
            Span::styled(func.category, theme::accent()),
        ]));
        detail_lines.push(Line::from(""));

        // Parameters
        detail_lines.push(Line::from(vec![
            Span::styled("Parameters:", Style::default().add_modifier(Modifier::UNDERLINED)),
        ]));
        detail_lines.push(Line::from(vec![
            Span::styled(format!("  {}", func.params), theme::bright()),
        ]));
        detail_lines.push(Line::from(""));

        // Description
        detail_lines.push(Line::from(vec![
            Span::styled("Description:", Style::default().add_modifier(Modifier::UNDERLINED)),
        ]));
        // Wrap description if needed
        let desc = func.description;
        detail_lines.push(Line::from(vec![
            Span::styled(format!("  {}", desc), theme::bright()),
        ]));
        detail_lines.push(Line::from(""));

        // Example
        detail_lines.push(Line::from(vec![
            Span::styled("Example:", Style::default().add_modifier(Modifier::UNDERLINED)),
        ]));
        detail_lines.push(Line::from(vec![
            Span::styled(format!("  {}", func.example), Style::default().fg(Color::Rgb(180, 220, 150))),
        ]));
    } else {
        detail_lines.push(Line::from(vec![
            Span::styled("Select a function", theme::dim()),
        ]));
        detail_lines.push(Line::from(""));
        detail_lines.push(Line::from(vec![
            Span::styled("Details will appear here", theme::dim()),
        ]));
    }

    // Right block
    let right_block = Block::default()
        .title(" Details ")
        .title_style(theme::accent())
        .borders(Borders::ALL)
        .border_style(theme::accent())
        .style(Style::default().bg(Color::Black));

    let right_para = Paragraph::new(Text::from(detail_lines)).block(right_block);
    f.render_widget(right_para, columns[1]);

    // Footer with instructions (rendered over the bottom of overlay)
    let footer_area = Rect::new(
        overlay.x,
        overlay.y + overlay.height - 2,
        overlay.width,
        1,
    );
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Enter", theme::accent()),
        Span::styled("=insert ", theme::dim()),
        Span::styled("↑↓", theme::accent()),
        Span::styled("=nav ", theme::dim()),
        Span::styled("Esc", theme::accent()),
        Span::styled("=close ", theme::dim()),
        Span::styled("/", theme::accent()),
        Span::styled("=search", theme::dim()),
    ]));
    f.render_widget(footer, footer_area);
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
