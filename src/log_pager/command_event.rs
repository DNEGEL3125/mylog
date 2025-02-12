use crossterm::event::KeyModifiers;

pub enum CommandEvent {
    Cancel,
    Char(char),
    Execute,
    Backspace,
    ClearLine,
    None,
}

pub fn get_command_event() -> CommandEvent {
    use crossterm::event::KeyCode;
    match crossterm::event::read().expect("Unable to read events") {
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
