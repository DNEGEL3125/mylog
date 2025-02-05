pub enum UserEvent {
    NextDay,
    PrevDay,
    NextLine,
    PrevLine,
    Quit,
    Search,
    Resize(u16, u16),
    None,
}

pub fn get_user_event() -> UserEvent {
    use crossterm::event::KeyCode;
    match crossterm::event::read().expect("Unable to read events") {
        crossterm::event::Event::Key(key_event) => match key_event.code {
            KeyCode::Char('j') => UserEvent::NextLine,
            KeyCode::Char('k') => UserEvent::PrevLine,
            KeyCode::Char('l') => UserEvent::NextDay,
            KeyCode::Char('h') => UserEvent::PrevDay,
            KeyCode::Char('q') => UserEvent::Quit,
            _ => UserEvent::None,
        },
        crossterm::event::Event::Resize(columns, rows) => UserEvent::Resize(columns, rows),
        _ => UserEvent::None,
    }
}
