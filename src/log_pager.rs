use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::{Datelike, Days, NaiveDate};
use colored::Colorize;
use minus::Pager;

use crate::custom_input_classifier;
use crate::file_utils::get_file_content_by_path;
use crate::log_config::construct_log_file_path;

pub(crate) struct LogPager {
    pager: Pager,
    date: NaiveDate,
    log_dir_path: PathBuf,
    verbose: bool,
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
            verbose: false,
        };

        ret.update_pager();

        ret
    }

    pub fn set_verbose(&mut self, value: bool) {
        self.verbose = value;
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

    fn colour_log_conetent(content: &String, date_color: colored::Color) -> String {
        let mut ret = String::new();

        for line in content.lines() {
            let idx = match line.find("]") {
                Some(res) => res,
                None => {
                    ret.push_str(line);
                    ret.push('\n');
                    continue;
                }
            };
            if !line.starts_with("[") {
                ret.push_str(line);
                ret.push('\n');
                continue;
            }

            let mut date_string = line.to_owned();
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
            if self.verbose {
                self.pager
                    .send_message(format!("'{}' doesn't exist", file_path.display()))
                    .expect("Can't send messages");
            }
            String::new()
        };
        let colored_content = Self::colour_log_conetent(&file_content, colored::Color::Green);
        self.set_text(&colored_content);
        let _ = self
            .pager
            .set_prompt(format!("{} {}", self.date, self.date.weekday()));
    }

    fn set_text(&self, s: &str) {
        self.pager.set_text(s).expect("Can't open the pager");
    }

    fn _push_str(&self, s: impl Into<String>) {
        self.pager.push_str(s).expect("Can't open the pager");
    }

    pub fn _start_paging(&mut self) {
        self.update_pager();
        minus::dynamic_paging(self.pager.clone()).expect("Can't show pager");
    }
}
