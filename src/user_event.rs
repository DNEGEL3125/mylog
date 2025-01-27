pub enum UserEvent {
    NextDay,
    PrevDay,
    NextLine,
    PrevLine,
    Quit,
    Search,
    None,
}

pub fn get_user_event() -> UserEvent {
    use crossterm::event::KeyCode;
    match crossterm::event::read().expect("Unable to read events") {
        crossterm::event::Event::FocusGained => {}
        crossterm::event::Event::FocusLost => {}
        crossterm::event::Event::Key(key_event) => {
            if key_event.code == KeyCode::Char('j') {
                return UserEvent::NextLine;
            }
            if key_event.code == KeyCode::Char('k') {
                return UserEvent::PrevLine;
            }
            if key_event.code == KeyCode::Char('l') {
                return UserEvent::NextDay;
            }
            if key_event.code == KeyCode::Char('h') {
                return UserEvent::PrevDay;
            }
            if key_event.code == KeyCode::Char('q') {
                return UserEvent::Quit;
            }
        }
        crossterm::event::Event::Mouse(_mouse_event) => {}
        crossterm::event::Event::Paste(_) => {}
        crossterm::event::Event::Resize(_, _) => {}
    }

    return UserEvent::None;
}
