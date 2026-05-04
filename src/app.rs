use crate::core::env::Environment;
use crate::core::formatter;
use crate::storage::history::History;
use crate::tui::events::Action;
use crate::tui::input::InputBuffer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    Command,
    Help,
}

pub struct App {
    pub input: InputBuffer,
    pub env: Environment,
    pub history: History,
    pub last_result: Option<String>,
    pub last_error: Option<String>,
    pub mode: AppMode,
    pub history_index: Option<usize>,
    pub running: bool,
}

impl App {
    pub fn new() -> Self {
        let history = History::load().unwrap_or_default();

        let mut env = Environment::new();

        // Load pre-defined constants as variables
        for (name, _desc, val) in crate::core::constants::list() {
            env.set(name, crate::core::value::Value::new(val));
        }

        Self {
            input: InputBuffer::new(),
            env,
            history,
            last_result: None,
            last_error: None,
            mode: AppMode::Normal,
            history_index: None,
            running: true,
        }
    }

    pub fn handle_action(&mut self, action: Action) {
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
                self.input.insert_char(c);
            }
            Action::CommandMode => {
                self.input.insert_char(':');
            }
        }
    }

    fn eval_input(&mut self) {
        let expr_str = self.input.content.trim().to_string();

        if expr_str.is_empty() {
            return;
        }

        if expr_str.starts_with(':') {
            self.handle_command(&expr_str[1..]);
            return;
        }

        match crate::core::parser::parse(&expr_str) {
            Ok(ast) => {
                // If assignment, evaluate and store
                if let Some((name, val_expr)) = ast.as_assignment() {
                    let name = name.to_string();
                    match val_expr.eval(&self.env) {
                        Ok(value) => {
                            self.env.set(name.clone(), value.clone());
                            let formatted = formatter::format_assignment(&name, &value);
                            self.last_result = Some(formatted.clone());
                            self.last_error = None;
                            self.history.add(expr_str.clone(), formatted, false);
                        }
                        Err(e) => {
                            let msg = e.to_string();
                            self.last_error = Some(msg.clone());
                            self.last_result = None;
                            self.history
                                .add(expr_str, formatter::format_error(&msg), true);
                        }
                    }
                } else {
                    match ast.eval(&self.env) {
                        Ok(value) => {
                            let formatted = formatter::format_value(&value);
                            self.last_result = Some(formatted.clone());
                            self.last_error = None;
                            self.history.add(expr_str.clone(), formatted, false);
                        }
                        Err(e) => {
                            let msg = e.to_string();
                            self.last_error = Some(msg.clone());
                            self.last_result = None;
                            self.history
                                .add(expr_str, formatter::format_error(&msg), true);
                        }
                    }
                }
            }
            Err(e) => {
                let msg = e.to_string();
                self.last_error = Some(msg.clone());
                self.last_result = None;
                self.history
                    .add(expr_str, formatter::format_error(&msg), true);
            }
        }

        self.input.clear();
        self.history_index = None;
        let _ = self.history.save();
    }

    fn handle_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.first() {
            Some(&"help") | Some(&"h") | Some(&"?") => {
                self.mode = AppMode::Help;
                self.last_result = Some(HELP_TEXT.to_string());
                self.last_error = None;
            }
            Some(&"clear") | Some(&"cls") => {
                self.clear();
            }
            Some(&"vars") => {
                let mut msg = String::from("Variables:\n");
                for (name, value) in self.env.iter() {
                    msg.push_str(&format!(
                        "  {} = {}\n",
                        name,
                        formatter::format_value(value)
                    ));
                }
                msg.push_str("\nConstants:\n");
                for (name, _desc, val) in crate::core::constants::list() {
                    let v = crate::core::value::Value::new(val);
                    msg.push_str(&format!("  {} ≈ {}\n", name, formatter::format_value(&v)));
                }
                self.last_result = Some(msg);
                self.last_error = None;
            }
            Some(&"consts") | Some(&"constants") => {
                let mut msg = String::from("Constants:\n");
                for (name, _desc, val) in crate::core::constants::list() {
                    let v = crate::core::value::Value::new(val);
                    msg.push_str(&format!("  {} ≈ {}\n", name, formatter::format_value(&v)));
                }
                self.last_result = Some(msg);
                self.last_error = None;
            }
            Some(&"history") | Some(&"hist") => {
                let mut msg = String::from("History:\n");
                for entry in self.history.last_n(30) {
                    msg.push_str(&format!("  {} = {}\n", entry.expression, entry.result));
                }
                self.last_result = Some(msg);
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
                self.input.content = exprs[idx].to_string();
                self.input.cursor_end();
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
                } else {
                    self.history_index = Some(idx + 1);
                    self.input.content = exprs[idx + 1].to_string();
                    self.input.cursor_end();
                }
            }
        }
    }

    fn autocomplete(&mut self) {
        let text = &self.input.content;
        let cursor = self.input.cursor_pos;

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
                        // find the start of current word
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

        for (name, _desc, _) in crate::core::constants::list() {
            if name.starts_with(prefix) && name.len() > prefix.len() {
                suggestions.push(name);
            }
        }

        for name in crate::core::functions::function_names() {
            if name.starts_with(prefix) && name.len() > prefix.len() {
                suggestions.push(name.to_string());
            }
        }

        for (var_name, _) in self.env.iter() {
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
            let new_cursor_pos = self.input.cursor_pos + sugg.len() - prefix_len;
            self.input.content.insert_str(insert_pos, remaining);
            self.input.cursor_pos = new_cursor_pos;
        }
    }

    pub fn clear(&mut self) {
        self.last_result = None;
        self.last_error = None;
        self.input.clear();
        self.history_index = None;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

const HELP_TEXT: &str = r#"engcalc - Scientific Calculator

KEYBINDINGS:
  Enter       Evaluate expression
  Esc         Quit application
  Ctrl+C      Quit application
  Ctrl+L      Clear screen
  Ctrl+U      Clear input
  ↑/↓         Navigate history
  Tab         Autocomplete
  ←/→         Move cursor
  Home/End    Go to start/end
  Backspace   Delete before cursor
  Delete      Delete after cursor
  :           Start command

COMMANDS:
  :help       Show this help
  :clear      Clear screen
  :vars       Show variables and constants
  :consts     Show constants
  :history    Show calculation history
  :quit       Exit application

OPERATORS:
  + - * / ^ %   Arithmetic
  -x            Unary minus
  2x 2(3+4)     Implicit multiplication

FUNCTIONS:
  sin cos tan asin acos atan
  sqrt ln log log10 exp
  abs floor ceil round min max pow

EXAMPLES:
  2 + 3 * 4        = 14
  (2 + 3) * 4      = 20
  2^3^2            = 512
  sin(pi / 2)      = 1
  x = 5
  2x + 1           = 11
  10 km in m       = 10000
  1 h in s         = 3600
  5 bar in Pa      = 500000
"#;
