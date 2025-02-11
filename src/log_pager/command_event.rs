pub enum CommandEvent {
    Cancel,
    Char(char),
    Execute,
    None,
}

pub fn get_command_event() -> CommandEvent {
    use crossterm::event::KeyCode;
    match crossterm::event::read().expect("Unable to read events") {
        crossterm::event::Event::Key(key_event) => match key_event.code {
            KeyCode::Esc => CommandEvent::Cancel,
            KeyCode::Char(c) => CommandEvent::Char(c),
            KeyCode::Enter => CommandEvent::Execute,
            _ => CommandEvent::None,
        },
        _ => CommandEvent::None,
    }
}
