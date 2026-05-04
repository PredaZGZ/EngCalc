use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

pub enum AppEvent {
    Tick,
    Key(KeyEvent),
    Resize(u16, u16),
}

pub fn poll_event(timeout: Duration) -> Result<AppEvent, std::io::Error> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => Ok(AppEvent::Key(key)),
            Event::Resize(w, h) => Ok(AppEvent::Resize(w, h)),
            _ => Ok(AppEvent::Tick),
        }
    } else {
        Ok(AppEvent::Tick)
    }
}

pub fn handle_key(key: KeyEvent) -> Option<Action> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) => Some(Action::Quit),
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(Action::Quit),
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => Some(Action::ClearScreen),
        (_, KeyCode::Enter) => Some(Action::Eval),
        (_, KeyCode::Up) => Some(Action::HistoryUp),
        (_, KeyCode::Down) => Some(Action::HistoryDown),
        (_, KeyCode::Tab) => Some(Action::Autocomplete),
        (KeyModifiers::CONTROL, KeyCode::Char('u')) => Some(Action::ClearInput),
        (_, KeyCode::Left) => Some(Action::CursorLeft),
        (_, KeyCode::Right) => Some(Action::CursorRight),
        (_, KeyCode::Home) => Some(Action::CursorHome),
        (_, KeyCode::End) => Some(Action::CursorEnd),
        (_, KeyCode::Backspace) => Some(Action::DeleteBackward),
        (_, KeyCode::Delete) => Some(Action::DeleteForward),
        (_, KeyCode::Char('f')) if key.modifiers == KeyModifiers::CONTROL => {
            Some(Action::CursorRight)
        }
        (_, KeyCode::Char('b')) if key.modifiers == KeyModifiers::CONTROL => {
            Some(Action::CursorLeft)
        }
        (_, KeyCode::Char('a')) if key.modifiers == KeyModifiers::CONTROL => {
            Some(Action::CursorHome)
        }
        (_, KeyCode::Char('e')) if key.modifiers == KeyModifiers::CONTROL => {
            Some(Action::CursorEnd)
        }
        (_, KeyCode::Char(':')) => Some(Action::CommandMode),
        (_, KeyCode::Char(c)) => Some(Action::InputChar(c)),
        _ => None,
    }
}

#[derive(Debug)]
pub enum Action {
    Quit,
    Eval,
    ClearScreen,
    ClearInput,
    HistoryUp,
    HistoryDown,
    Autocomplete,
    CursorLeft,
    CursorRight,
    CursorHome,
    CursorEnd,
    DeleteBackward,
    DeleteForward,
    InputChar(char),
    CommandMode,
}
