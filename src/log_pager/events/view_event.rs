use crossterm::event::KeyCode;

pub enum ViewEvent {
    NextDay,
    PrevDay,
    NextLine,
    PrevLine,
    GotoPageBegin,
    GotoPageEnd,
    Quit,
    Edit,
    SearchNext,
    SearchPrev,
    Resize(u16, u16),
    EnterCommandMode,
    EnterSearchMode,
    None,
}

impl ViewEvent {
    pub fn from_crossterm_event(crossterm_event: crossterm::event::Event) -> Self {
        match crossterm_event {
            crossterm::event::Event::Key(key_event) => match key_event.code {
                KeyCode::Char('j') => ViewEvent::NextLine,
                KeyCode::Char('k') => ViewEvent::PrevLine,
                KeyCode::Char('g') => ViewEvent::GotoPageBegin,
                KeyCode::Char('G') => ViewEvent::GotoPageEnd,
                KeyCode::Char('l') => ViewEvent::NextDay,
                KeyCode::Char('h') => ViewEvent::PrevDay,
                KeyCode::Char('q') => ViewEvent::Quit,
                KeyCode::Char('e') => ViewEvent::Edit,
                KeyCode::Char('n') => ViewEvent::SearchNext,
                KeyCode::Char('N') => ViewEvent::SearchPrev,
                KeyCode::Char(':') => ViewEvent::EnterCommandMode,
                KeyCode::Char('/') => ViewEvent::EnterSearchMode,
                _ => ViewEvent::None,
            },
            crossterm::event::Event::Resize(columns, rows) => ViewEvent::Resize(columns, rows),
            _ => ViewEvent::None,
        }
    }
}
