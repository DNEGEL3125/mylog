use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::{Days, NaiveDate};
use colored::Colorize;
use minus::Pager;

use crate::custom_input_classifier;
use crate::file_utils::get_file_content_by_path;
use crate::log_config::construct_log_file_path;

pub(crate) struct LogPager {
    pager: Pager,
    date: NaiveDate,
    log_dir_path: PathBuf,
}

fn get_today_date() -> NaiveDate {
    return chrono::prelude::Local::now().date_naive();
}

impl LogPager {
    pub fn new(date: NaiveDate, log_dir_path: PathBuf) -> Self {
        let pager = Pager::new();

        let mut ret = LogPager {
            pager,
            date,
            log_dir_path,
        };

        // let custom_input_classifier = Box::new(ret.init_input_classifier());
        // Add keyboard/mouse-bindings
        // ret.pager
        //     .set_input_classifier(custom_input_classifier)
        //     .unwrap();

        ret.update_pager();

        ret
    }

    pub fn init_input_classifier(log_pager: &Arc<Mutex<Self>>) {
        let event_emitter = &mut custom_input_classifier::EVENT_EMITTER.lock().unwrap();

        let log_pager_clone = log_pager.clone();
        event_emitter.on("LEFT", move |_: ()| {
            log_pager_clone.lock().unwrap().prev_day();
        });

        let log_pager_clone = log_pager.clone();
        event_emitter.on("RIGHT", move |_: ()| {
            log_pager_clone.lock().unwrap().next_day();
        });

        log_pager
            .lock()
            .unwrap()
            .pager
            .set_input_classifier(Box::new(custom_input_classifier::CustomInputClassifier))
            .unwrap();
    }

    pub fn pager(&self) -> &Pager {
        &self.pager
    }

    fn next_day(&mut self) {
        if self.date == get_today_date() {
            let err_msg = "This is already today";
            let result = self.pager.send_message(err_msg);
            if result.is_err() {
                println!("{}", err_msg);
            }
            return;
        }
        self.date = self
            .date
            .checked_add_days(Days::new(1))
            .expect("Date out of range");

        self.update_pager();
    }

    fn prev_day(&mut self) {
        self.date = self
            .date
            .checked_sub_days(Days::new(1))
            .expect("Date out of range");

        self.update_pager();
    }

    fn colour_log_conetent(content: String, date_color: colored::Color) -> String {
        let mut ret = String::new();

        for line in content.lines().map(|x| x.to_owned()) {
            let idx = match line.find("]") {
                Some(res) => res,
                None => continue,
            };
            if !line.starts_with("[") {
                continue;
            }

            let mut date_string = line;
            let log_content = date_string.split_off(idx + 1);

            ret.push_str(&format!("{}{log_content}\n", date_string.color(date_color)));
        }

        return ret;
    }

    fn update_pager(&mut self) {
        let file_path = construct_log_file_path(&self.log_dir_path, self.date);
        let file_content = if file_path.exists() {
            get_file_content_by_path(&file_path)
        } else {
            String::new()
        };
        let colored_content = Self::colour_log_conetent(file_content, colored::Color::Green);
        self.set_text(&colored_content);
        let _ = self.pager.set_prompt(self.date.to_string());
    }

    // pub fn _register_events(log_pager: &Arc<Mutex<Self>>) {
    //     use minus::input::InputEvent;

    //     let mut input_register = HashedEventRegister::default();

    //     // Creates another pointer to the same allocation
    //     let self_clone = Arc::clone(&log_pager);

    //     // Left key event
    //     input_register.add_key_events(&["down"], move |_, _| {
    //         return InputEvent::Exit;
    //         // To previous day
    //         self_clone.lock().expect("Threading error").prev_day();
    //         InputEvent::Ignore
    //     });

    //     // Creates another pointer to the same allocation
    //     let self_clone = Arc::clone(&log_pager);

    //     // Right key event
    //     input_register.add_key_events(&["l"], move |_, _| {
    //         // To next day
    //         self_clone.lock().expect("Threading error").next_day();
    //         InputEvent::Ignore
    //     });
    // }

    fn set_text(&self, s: &str) {
        self.pager.set_text(s).expect("Can't show pager");
    }

    fn _push_str(&self, s: impl Into<String>) {
        self.pager.push_str(s).expect("Can't show pager");
    }

    pub fn _start_paging(&mut self) {
        self.update_pager();
        minus::dynamic_paging(self.pager.clone()).expect("Can't show pager");
    }
}
