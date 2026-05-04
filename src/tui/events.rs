use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

pub enum AppEvent {
    Tick,
    Key(KeyEvent),
    Resize(u16, u16),
}

pub fn poll_event(timeout: Duration) -> Result<AppEvent, std::io::Error> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Release {
                    return Ok(AppEvent::Tick);
                }
                Ok(AppEvent::Key(key))
            }
            Event::Resize(w, h) => Ok(AppEvent::Resize(w, h)),
            _ => Ok(AppEvent::Tick),
        }
    } else {
        Ok(AppEvent::Tick)
    }
}

pub fn handle_key(key: KeyEvent) -> Option<Action> {
    let ctrl = key.modifiers == KeyModifiers::CONTROL;

    match key.code {
        KeyCode::Esc => Some(Action::ClearInput),
        KeyCode::Char('c') if ctrl => Some(Action::Quit),
        KeyCode::Enter => Some(Action::Eval),
        KeyCode::Up => Some(Action::HistoryUp),
        KeyCode::Down => Some(Action::HistoryDown),
        KeyCode::Tab => Some(Action::Autocomplete),
        KeyCode::Char('l') if ctrl => Some(Action::ClearScreen),
        KeyCode::Char('u') if ctrl => Some(Action::ClearInput),
        KeyCode::Left => Some(Action::CursorLeft),
        KeyCode::Right => Some(Action::CursorRight),
        KeyCode::Home => Some(Action::CursorHome),
        KeyCode::End => Some(Action::CursorEnd),
        KeyCode::Backspace => Some(Action::DeleteBackward),
        KeyCode::Delete => Some(Action::DeleteForward),
        KeyCode::F(1) => Some(Action::ShowHelp),
        KeyCode::F(2) => Some(Action::ShowConsts),
        KeyCode::Char('f') if ctrl => Some(Action::CursorRight),
        KeyCode::Char('b') if ctrl => Some(Action::CursorLeft),
        KeyCode::Char('a') if ctrl => Some(Action::CursorHome),
        KeyCode::Char('e') if ctrl => Some(Action::CursorEnd),
        KeyCode::Char(':') => Some(Action::CommandMode),
        KeyCode::Char(c) => Some(Action::InputChar(c)),
        _ => None,
    }
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Quit,
    Eval,
    ClearScreen,
    ClearInput,
    HistoryUp,
    HistoryDown,
    Autocomplete,
    ShowHelp,
    ShowConsts,
    CursorLeft,
    CursorRight,
    CursorHome,
    CursorEnd,
    DeleteBackward,
    DeleteForward,
    InputChar(char),
    CommandMode,
}
