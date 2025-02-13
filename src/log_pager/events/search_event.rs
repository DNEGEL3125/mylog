use crossterm::event::{KeyCode, KeyModifiers};

pub enum SearchEvent {
    Backspace,
    Cancel,
    Char(char),
    ClearLine,
    Confirm,
    None,
}

impl SearchEvent {
    pub fn from_crossterm_event(crossterm_event: crossterm::event::Event) -> Self {
        match crossterm_event {
            crossterm::event::Event::Key(key_event) => {
                if key_event.modifiers.is_empty() {
                    match key_event.code {
                        KeyCode::Esc => SearchEvent::Cancel,
                        KeyCode::Char(c) => SearchEvent::Char(c),
                        KeyCode::Enter => SearchEvent::Confirm,
                        KeyCode::Backspace => SearchEvent::Backspace,
                        _ => SearchEvent::None,
                    }
                } else if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    match key_event.code {
                        KeyCode::Char('u') => SearchEvent::ClearLine,
                        _ => SearchEvent::None,
                    }
                } else {
                    SearchEvent::None
                }
            }
            _ => SearchEvent::None,
        }
    }
}
