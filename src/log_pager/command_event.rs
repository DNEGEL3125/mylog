use crossterm::event::{KeyCode, KeyModifiers};

pub enum CommandEvent {
    Cancel,
    Char(char),
    Execute,
    Backspace,
    ClearLine,
    None,
}

impl CommandEvent {
    pub fn from_crossterm_event(crossterm_event: crossterm::event::Event) -> Self {
        match crossterm_event {
            crossterm::event::Event::Key(key_event) => {
                if key_event.modifiers.is_empty() {
                    match key_event.code {
                        KeyCode::Esc => CommandEvent::Cancel,
                        KeyCode::Char(c) => CommandEvent::Char(c),
                        KeyCode::Enter => CommandEvent::Execute,
                        KeyCode::Backspace => CommandEvent::Backspace,
                        _ => CommandEvent::None,
                    }
                } else if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    match key_event.code {
                        KeyCode::Char('u') => CommandEvent::ClearLine,
                        _ => CommandEvent::None,
                    }
                } else {
                    CommandEvent::None
                }
            }
            _ => CommandEvent::None,
        }
    }
}
