use std::sync::Mutex;

use event_emitter_rs::EventEmitter;
use lazy_static::lazy_static;
use minus::input::crossterm_event::*;
use minus::input::InputClassifier;
use minus::input::*;
use minus::PagerState;

lazy_static! {
    pub static ref EVENT_EMITTER: Mutex<EventEmitter> = Mutex::new(EventEmitter::new());
    static ref STORED_NUMBER: Mutex<String> = Mutex::new(String::new());
}

/// Returns and empties the stored number. This number is typically used to repeat the next operation multiple times.
fn get_and_clear_stored_number() -> usize {
    let mut times_str = STORED_NUMBER.lock().unwrap();
    // Parse the string to integer
    let times: usize = times_str.parse().unwrap_or(1);
    times_str.clear();
    times
}

pub struct CustomInputClassifier;

impl InputClassifier for CustomInputClassifier {
    fn classify_input(&self, ev: Event, ps: &PagerState) -> Option<InputEvent> {
        match ev {
            // Quit
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
                ..
            }) => Some(InputEvent::Exit),
            // Previous date
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::NONE,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                EVENT_EMITTER.lock().unwrap().emit("LEFT", ());
                Some(InputEvent::RestorePrompt)
            }
            // Next date
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: KeyModifiers::NONE,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                EVENT_EMITTER.lock().unwrap().emit("RIGHT", ());
                Some(InputEvent::RestorePrompt)
            }
            // Previous line
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: KeyModifiers::NONE,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                let times = get_and_clear_stored_number();
                Some(InputEvent::UpdateUpperMark(
                    ps.upper_mark.saturating_sub(times),
                ))
            }
            // Next line
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: KeyModifiers::NONE,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                let times = get_and_clear_stored_number();
                Some(InputEvent::UpdateUpperMark(
                    ps.upper_mark.saturating_add(times),
                ))
            }

            Event::Key(KeyEvent {
                code: KeyCode::Char(char_code),
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                if char_code.is_digit(10) {
                    STORED_NUMBER.lock().unwrap().push(char_code);
                    return Some(InputEvent::Number(char_code));
                }
                return None;
            }
            _ => None,
        }
    }
}
