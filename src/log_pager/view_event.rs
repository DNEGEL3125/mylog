pub enum ViewEvent {
    NextDay,
    PrevDay,
    NextLine,
    PrevLine,
    GotoPageBegin,
    GotoPageEnd,
    Quit,
    Edit,
    Search,
    Resize(u16, u16),
    EnterCommandMode,
    None,
}

pub fn get_view_event() -> ViewEvent {
    use crossterm::event::KeyCode;
    match crossterm::event::read().expect("Unable to read events") {
        crossterm::event::Event::Key(key_event) => match key_event.code {
            KeyCode::Char('j') => ViewEvent::NextLine,
            KeyCode::Char('k') => ViewEvent::PrevLine,
            KeyCode::Char('g') => ViewEvent::GotoPageBegin,
            KeyCode::Char('G') => ViewEvent::GotoPageEnd,
            KeyCode::Char('l') => ViewEvent::NextDay,
            KeyCode::Char('h') => ViewEvent::PrevDay,
            KeyCode::Char('q') => ViewEvent::Quit,
            KeyCode::Char('e') => ViewEvent::Edit,
            KeyCode::Char(':') => ViewEvent::EnterCommandMode,
            _ => ViewEvent::None,
        },
        crossterm::event::Event::Resize(columns, rows) => ViewEvent::Resize(columns, rows),
        _ => ViewEvent::None,
    }
}
