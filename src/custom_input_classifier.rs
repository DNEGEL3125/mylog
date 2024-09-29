use std::sync::Mutex;

use event_emitter_rs::EventEmitter;
use lazy_static::lazy_static;
use minus::input::crossterm_event::*;
use minus::input::InputClassifier;
use minus::input::*;
use minus::PagerState;

lazy_static! {
    pub static ref EVENT_EMITTER: Mutex<EventEmitter> = Mutex::new(EventEmitter::new());
}

pub struct CustomInputClassifier;

impl InputClassifier for CustomInputClassifier {
    fn classify_input(&self, ev: Event, _ps: &PagerState) -> Option<InputEvent> {
        match ev {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
                ..
            }) => Some(InputEvent::Exit),
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                EVENT_EMITTER.lock().unwrap().emit("LEFT", ());
                Some(InputEvent::RestorePrompt)
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                EVENT_EMITTER.lock().unwrap().emit("RIGHT", ());
                Some(InputEvent::RestorePrompt)
            }
            _ => None,
        }
    }
}
