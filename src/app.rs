use crate::core::env::Environment;
use crate::core::formatter;
use crate::storage::history::History;
use crate::tui::events::Action;
use crate::tui::input::InputBuffer;
use std::collections::HashMap;

pub struct App {
    pub input: InputBuffer,
    pub env: Environment,
    pub user_vars: HashMap<String, crate::core::value::Value>,
    pub history: History,
    pub last_result: Option<String>,
    pub last_error: Option<String>,
    pub history_index: Option<usize>,
    pub running: bool,
    pub is_command_mode: bool,
    pub show_consts: bool,
    pub show_help: bool,
    pub show_functions: bool,
    pub consts_search: String,
    pub consts_selected: usize,
    pub funcs_search: String,
    pub funcs_selected: usize,
}

impl App {
    pub fn new() -> Self {
        let history = History::load().unwrap_or_default();

        Self {
            input: InputBuffer::new(),
            env: Environment::new(),
            user_vars: HashMap::new(),
            history,
            last_result: None,
            last_error: None,
            history_index: None,
            running: true,
            is_command_mode: false,
            show_consts: false,
            show_help: false,
            show_functions: false,
            consts_search: String::new(),
            consts_selected: 0,
            funcs_search: String::new(),
            funcs_selected: 0,
        }
    }

    pub fn handle_action(&mut self, action: Action) {
        if matches!(action, Action::ShowHelp) {
            self.show_help = !self.show_help;
            self.show_consts = false;
            return;
        }

        if matches!(action, Action::ShowConsts) {
            self.show_consts = !self.show_consts;
            self.show_help = false;
            self.show_functions = false;
            if self.show_consts {
                self.consts_search.clear();
                self.consts_selected = 0;
            }
            return;
        }

        if matches!(action, Action::ShowFunctions) {
            self.show_functions = !self.show_functions;
            self.show_help = false;
            self.show_consts = false;
            if self.show_functions {
                self.funcs_search.clear();
                self.funcs_selected = 0;
            }
            return;
        }

        if self.show_functions {
            self.handle_funcs_action(action);
            return;
        }

        if self.show_consts || self.show_functions {
            self.handle_consts_action(action);
            return;
        }

        if self.show_help {
            if matches!(action, Action::Quit) {
                self.running = false;
            }
            self.show_help = false;
            return;
        }

        match action {
            Action::Quit => {
                self.running = false;
            }
            Action::Eval => {
                self.eval_input();
            }
            Action::ClearScreen => {
                self.clear();
            }
            Action::ClearInput => {
                self.input.clear();
                self.history_index = None;
            }
            Action::ClearAll => {
                self.history.clear();
                let _ = self.history.save();
                self.user_vars.clear();
                self.env = Environment::new();
                self.input.clear();
                self.history_index = None;
                self.last_result = None;
                self.last_error = None;
            }
            Action::HistoryUp => {
                self.history_up();
            }
            Action::HistoryDown => {
                self.history_down();
            }
            Action::Autocomplete => {
                self.autocomplete();
            }
            Action::CursorLeft => {
                self.input.cursor_left();
            }
            Action::CursorRight => {
                self.input.cursor_right();
            }
            Action::CursorHome => {
                self.input.cursor_home();
            }
            Action::CursorEnd => {
                self.input.cursor_end();
            }
            Action::DeleteBackward => {
                self.input.delete_char();
            }
            Action::DeleteForward => {
                self.input.delete_forward();
            }
            Action::InputChar(c) => {
                if c == ':' && self.input.is_empty() {
                    self.is_command_mode = true;
                }
                self.input.insert_char(c);
            }
            Action::CommandMode => {
                if !self.input.is_empty() {
                    self.input.clear();
                }
                self.is_command_mode = true;
                self.input.insert_char(':');
            }
            _ => {}
        }
    }

    fn eval_input(&mut self) {
        let expr_str = self.input.content().trim().to_string();

        if expr_str.is_empty() {
            self.is_command_mode = false;
            return;
        }

        if expr_str.starts_with(':') {
            self.handle_command(&expr_str[1..]);
            return;
        }

        match crate::core::parser::parse(&expr_str) {
            Ok(ast) => {
                // Check for function definition: f(x) = expr
                if let Some((name, params, body)) = ast.as_function_def() {
                    use crate::core::env::UserFunction;
                    let func = UserFunction {
                        name: name.to_string(),
                        params: params.to_vec(),
                        body: (*body).clone(),
                    };
                    self.env.set_function(func);
                    let formatted = format!("{}({}) defined", name, params.join(", "));
                    self.last_result = Some(formatted.clone());
                    self.last_error = None;
                    let workspace = self.capture_workspace();
                    self.history
                        .add(expr_str.clone(), formatted, false, workspace);
                } else if let Some((name, val_expr)) = ast.as_assignment() {
                    let name = name.to_string();
                    match val_expr.eval(&self.env) {
                        Ok(value) => {
                            self.env.set(name.clone(), value.clone());
                            self.user_vars.insert(name.clone(), value.clone());
                            // Also store in ans
                            self.env.set("ans".to_string(), value.clone());
                            self.user_vars.insert("ans".to_string(), value.clone());
                            let formatted = formatter::format_assignment(&name, &value);
                            self.last_result = Some(formatted.clone());
                            self.last_error = None;
                            let workspace = self.capture_workspace();
                            self.history
                                .add(expr_str.clone(), formatted, false, workspace);
                        }
                        Err(e) => {
                            let msg = e.to_string();
                            self.last_error = Some(msg.clone());
                            self.last_result = None;
                            let workspace = self.capture_workspace();
                            self.history.add(
                                expr_str,
                                formatter::format_error(&msg),
                                true,
                                workspace,
                            );
                        }
                    }
                } else {
                    match ast.eval(&self.env) {
                        Ok(value) => {
                            // Store result in ans variable
                            self.env.set("ans".to_string(), value.clone());
                            self.user_vars.insert("ans".to_string(), value.clone());
                            let formatted = formatter::format_value(&value);
                            self.last_result = Some(formatted.clone());
                            self.last_error = None;
                            let workspace = self.capture_workspace();
                            self.history
                                .add(expr_str.clone(), formatted, false, workspace);
                        }
                        Err(e) => {
                            let msg = e.to_string();
                            self.last_error = Some(msg.clone());
                            self.last_result = None;
                            let workspace = self.capture_workspace();
                            self.history.add(
                                expr_str,
                                formatter::format_error(&msg),
                                true,
                                workspace,
                            );
                        }
                    }
                }
            }
            Err(e) => {
                let msg = e.to_string();
                self.last_error = Some(msg.clone());
                self.last_result = None;
                let workspace = self.capture_workspace();
                self.history
                    .add(expr_str, formatter::format_error(&msg), true, workspace);
            }
        }

        self.input.clear();
        self.is_command_mode = false;
        self.history_index = None;
        let _ = self.history.save();
    }

    fn handle_command(&mut self, cmd: &str) {
        self.is_command_mode = false;
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.first() {
            Some(&"help") | Some(&"h") | Some(&"?") => {
                self.show_help = true;
            }
            Some(&"clear") | Some(&"cls") => {
                self.clear();
            }
            Some(&"vars") => {
                let mut msg = String::from("Variables:\n");
                if self.user_vars.is_empty() {
                    msg.push_str("  (none yet)\n");
                } else {
                    for (name, value) in &self.user_vars {
                        msg.push_str(&format!(
                            "  {} = {}\n",
                            name,
                            formatter::format_value(value)
                        ));
                    }
                }
                self.last_result = Some(msg);
                self.last_error = None;
            }
            Some(&"consts") | Some(&"constants") => {
                self.show_consts = true;
            }
            Some(&"history") | Some(&"hist") => {
                let mut msg = String::from("History:\n");
                for entry in self.history.last_n(30) {
                    msg.push_str(&format!("  {} = {}\n", entry.expression, entry.result));
                }
                self.last_result = Some(msg);
                self.last_error = None;
            }
            Some(&"clearhist") => {
                self.history.clear();
                let _ = self.history.save();
                self.last_result = Some("History cleared".to_string());
                self.last_error = None;
            }
            Some(&"quit") | Some(&"exit") | Some(&"q") => {
                self.running = false;
            }
            _ => {
                self.last_error = Some(format!("unknown command: {}", cmd));
                self.last_result = None;
            }
        }

        self.input.clear();
        self.history_index = None;
    }

    fn history_up(&mut self) {
        let exprs = self.history.get_expressions();
        if exprs.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.history_index = Some(exprs.len() - 1);
            }
            Some(idx) if idx > 0 => {
                self.history_index = Some(idx - 1);
            }
            Some(_) => {}
        }

        if let Some(idx) = self.history_index {
            if idx < exprs.len() {
                // Clone workspace first to avoid borrow issues
                let workspace = self.history.get_workspace_at(idx).cloned();
                self.input.set_content(exprs[idx].to_string());
                self.input.cursor_end();
                // Restore workspace state
                if let Some(ws) = workspace {
                    self.restore_workspace(&ws);
                }
            }
        }
    }

    fn history_down(&mut self) {
        match self.history_index {
            None => {}
            Some(idx) => {
                let exprs = self.history.get_expressions();
                if idx + 1 >= exprs.len() {
                    self.history_index = None;
                    self.input.clear();
                    self.is_command_mode = false;
                    // Keep current workspace when clearing input
                } else {
                    // Clone workspace first to avoid borrow issues
                    let workspace = self.history.get_workspace_at(idx + 1).cloned();
                    self.history_index = Some(idx + 1);
                    self.input.set_content(exprs[idx + 1].to_string());
                    self.input.cursor_end();
                    // Restore workspace state
                    if let Some(ws) = workspace {
                        self.restore_workspace(&ws);
                    }
                }
            }
        }
    }

    fn autocomplete(&mut self) {
        let text = self.input.content();
        let cursor = self.input.cursor_pos();

        let word_start = {
            let mut found = 0;
            for (i, c) in text.char_indices() {
                if i >= cursor {
                    break;
                }
                if !c.is_alphanumeric() && c != '_' {
                    found = i + c.len_utf8();
                }
            }
            if cursor > 0 {
                let last_char = text.chars().nth(cursor - 1);
                if let Some(c) = last_char {
                    if c.is_alphanumeric() || c == '_' {
                        let mut ws = 0;
                        for (i, c) in text.char_indices().take(cursor) {
                            if !c.is_alphanumeric() && c != '_' {
                                ws = i + c.len_utf8();
                            }
                        }
                        ws
                    } else {
                        found
                    }
                } else {
                    found
                }
            } else {
                0
            }
        };

        let prefix = &text[word_start..cursor];

        let mut suggestions: Vec<String> = Vec::new();

        for c in crate::core::constants::list() {
            if c.name.starts_with(prefix) && c.name.len() > prefix.len() {
                suggestions.push(c.name.to_string());
            }
        }

        for name in crate::core::functions::function_names() {
            if name.starts_with(prefix) && name.len() > prefix.len() {
                suggestions.push(name.to_string());
            }
        }

        for (var_name, _) in &self.user_vars {
            if var_name.starts_with(prefix) && var_name.len() > prefix.len() {
                suggestions.push(var_name.clone());
            }
        }

        for unit in crate::core::units::get_unit_categories()
            .iter()
            .flat_map(|(_, units)| units.iter())
        {
            if unit.starts_with(prefix) && unit.len() > prefix.len() {
                suggestions.push(unit.clone());
            }
        }

        suggestions.sort();
        suggestions.dedup();

        if let Some(suggestion) = suggestions.first() {
            let prefix_len = prefix.len();
            let sugg = suggestion.clone();
            let remaining = &sugg[prefix_len..];
            let insert_pos = word_start + prefix_len;
            let new_cursor_pos = self.input.cursor_pos() + sugg.len() - prefix_len;
            // Insert remaining chars at position by rebuilding content
            let current = self.input.content();
            let before: String = current.chars().take(insert_pos).collect();
            let after: String = current.chars().skip(insert_pos).collect();
            self.input
                .set_content(format!("{}{}{}", before, remaining, after));
            self.input.set_cursor_pos(new_cursor_pos);
        }
    }

    pub fn clear(&mut self) {
        self.last_result = None;
        self.last_error = None;
        self.input.clear();
        self.is_command_mode = false;
        self.history_index = None;
    }

    fn handle_consts_action(&mut self, action: Action) {
        match action {
            Action::Quit | Action::ClearInput | Action::ClearScreen => {
                self.show_consts = false;
                self.consts_search.clear();
                self.consts_selected = 0;
            }
            Action::HistoryUp => {
                let filtered = crate::core::constants::search(&self.consts_search);
                if !filtered.is_empty() && self.consts_selected > 0 {
                    self.consts_selected -= 1;
                }
            }
            Action::HistoryDown => {
                let filtered = crate::core::constants::search(&self.consts_search);
                if !filtered.is_empty() && self.consts_selected + 1 < filtered.len() {
                    self.consts_selected += 1;
                }
            }
            Action::Eval | Action::ShowConsts => {
                let filtered = crate::core::constants::search(&self.consts_search);
                if !filtered.is_empty() && self.consts_selected < filtered.len() {
                    let selected = &filtered[self.consts_selected];
                    let insert_at = self.input.cursor_pos();
                    let current = self.input.content();
                    let before: String = current.chars().take(insert_at).collect();
                    let after: String = current.chars().skip(insert_at).collect();
                    self.input
                        .set_content(format!("{}{}{}", before, selected.name, after));
                    self.input.set_cursor_pos(insert_at + selected.name.len());
                }
                self.show_consts = false;
                self.consts_search.clear();
                self.consts_selected = 0;
            }
            Action::InputChar(c) => {
                self.consts_search.push(c);
                self.consts_selected = 0;
            }
            Action::DeleteBackward => {
                self.consts_search.pop();
                self.consts_selected = 0;
            }
            Action::DeleteForward => {
                if !self.consts_search.is_empty() {
                    self.consts_search.remove(self.consts_search.len() - 1);
                    self.consts_selected = 0;
                }
            }
            _ => {}
        }
    }

    fn handle_funcs_action(&mut self, action: Action) {
        match action {
            Action::Quit | Action::ClearInput | Action::ClearScreen => {
                self.show_functions = false;
                self.funcs_search.clear();
                self.funcs_selected = 0;
            }
            Action::HistoryUp => {
                let filtered = self.filtered_functions();
                if !filtered.is_empty() && self.funcs_selected > 0 {
                    self.funcs_selected -= 1;
                }
            }
            Action::HistoryDown => {
                let filtered = self.filtered_functions();
                if !filtered.is_empty() && self.funcs_selected + 1 < filtered.len() {
                    self.funcs_selected += 1;
                }
            }
            Action::Eval | Action::ShowFunctions => {
                let filtered = self.filtered_functions();
                if !filtered.is_empty() && self.funcs_selected < filtered.len() {
                    let selected = &filtered[self.funcs_selected];
                    let insert_text = format!("{}(", selected.name);
                    let insert_at = self.input.cursor_pos();
                    let current = self.input.content();
                    let before: String = current.chars().take(insert_at).collect();
                    let after: String = current.chars().skip(insert_at).collect();
                    self.input
                        .set_content(format!("{}{}{}", before, insert_text, after));
                    self.input.set_cursor_pos(insert_at + insert_text.len());
                }
                self.show_functions = false;
                self.funcs_search.clear();
                self.funcs_selected = 0;
            }
            Action::InputChar(c) => {
                self.funcs_search.push(c);
                self.funcs_selected = 0;
            }
            Action::DeleteBackward => {
                self.funcs_search.pop();
                self.funcs_selected = 0;
            }
            Action::DeleteForward => {
                if !self.funcs_search.is_empty() {
                    self.funcs_search.remove(self.funcs_search.len() - 1);
                    self.funcs_selected = 0;
                }
            }
            _ => {}
        }
    }

    pub fn filtered_functions(&self) -> Vec<crate::core::functions::FunctionInfo> {
        let all = crate::core::functions::list_functions();
        if self.funcs_search.is_empty() {
            return all;
        }
        let q = self.funcs_search.to_lowercase();
        all.into_iter()
            .filter(|f| {
                f.name.to_lowercase().contains(&q) || f.description.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Capture current workspace state (variables and functions)
    fn capture_workspace(&self) -> crate::storage::history::WorkspaceState {
        use crate::storage::history::{UserFunctionDef, WorkspaceState};

        let mut variables = std::collections::HashMap::new();
        for (name, value) in &self.user_vars {
            variables.insert(name.clone(), value.number());
        }

        let mut functions = std::collections::HashMap::new();
        for (name, func) in self.env.iter_functions() {
            functions.insert(
                name.clone(),
                UserFunctionDef {
                    name: func.name.clone(),
                    params: func.params.clone(),
                    body: format!("{}", func.body), // Format the AST body as string
                },
            );
        }

        WorkspaceState {
            variables,
            functions,
        }
    }

    /// Restore workspace state from a snapshot
    fn restore_workspace(&mut self, workspace: &crate::storage::history::WorkspaceState) {
        use crate::core::env::UserFunction;
        use crate::core::parser;

        // Clear current workspace
        self.user_vars.clear();
        self.env.clear();

        // Restore variables
        for (name, value) in &workspace.variables {
            let val = crate::core::value::Value::new(*value);
            self.env.set(name.clone(), val.clone());
            self.user_vars.insert(name.clone(), val);
        }

        // Restore functions
        for (_name, func_def) in &workspace.functions {
            // Try to parse the body string back to AST
            if let Ok(body_expr) = parser::parse(&func_def.body) {
                let func = UserFunction {
                    name: func_def.name.clone(),
                    params: func_def.params.clone(),
                    body: body_expr,
                };
                self.env.set_function(func);
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
