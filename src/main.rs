use std::{
    path::PathBuf,
    process::exit,
    sync::{Arc, Mutex},
};

use chrono::NaiveDate;
use file_utils::{append_line_to_file, create_log_file_if_not_exists};
use gumdrop::Options;
use log_pager::LogPager;

pub mod cl_params;
pub mod custom_input_classifier;
pub mod file_utils;
pub mod log_config;
pub mod log_pager;

#[derive(Debug)]
struct LogItem {
    date_time: chrono::NaiveDateTime,
    content: String,
}

impl std::fmt::Display for LogItem {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let data = format!(
            "[{}] {}\n",
            self.date_time.format("%Y-%m-%d %H:%M"),
            self.content
        );

        fmt.write_str(&data)
    }
}

fn paging_log_file_by_date(log_dir_path: PathBuf, date: NaiveDate) {
    use std::thread;
    let pager = LogPager::new(date, log_dir_path);

    let minus_pager = pager.pager().clone();

    let pager = Arc::new(Mutex::new(pager));

    LogPager::init_input_classifier(&pager);

    // Run the pager
    let pager_thread = thread::spawn(move || minus::dynamic_paging(minus_pager));

    // Wait for it to finish
    pager_thread.join().unwrap().unwrap();
}

fn append_log_item_to_file(
    log_item: &LogItem,
    log_file_path: &PathBuf,
    quiet: bool,
    verbose: bool,
) {
    let result = append_line_to_file(log_file_path, &log_item.to_string());
    if result.is_err() {
        println!("Can't write message to the log file");
        exit(3);
    }

    if quiet {
    } else if verbose {
        println!(
            r#"Written the log message to "{}""#,
            log_file_path.display()
        );
    } else {
        println!(
            r#"Written the log message to "{}""#,
            log_file_path
                .file_name()
                .expect("Isn't a filename")
                .to_str()
                .expect("Invalid Unicode")
        );
    }
}

fn parse_date_from_str(date_str: &str) -> NaiveDate {
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d");
    if date.is_err() {
        println!(
            "Invalid date '{}', the date should look like '2023-8-3'",
            date_str
        );
        exit(-101);
    }

    date.unwrap()
}

fn main() {
    let log_dir_path = PathBuf::from("/Users/dnegel3125/Documents/.private/MyLogs");
    let now = chrono::prelude::Local::now();
    let log_file_path = log_dir_path.join(format!("{}.log", now.format("%Y-%m-%d")));

    // Command line parameters
    let opts = cl_params::MyOptions::parse_args_default_or_exit();
    let verbose = opts.verbose;
    let quiet = opts.quiet;

    if !log_dir_path.exists() && !quiet {
        println!("The log dir doesn't exist");
        exit(1);
    }

    // If the log file does not exist, create it
    create_log_file_if_not_exists(&log_file_path, verbose);

    let is_write = opts.write.is_some();
    if is_write {
        let log_item = LogItem {
            date_time: now.naive_local(),
            content: opts.write.unwrap(),
        };

        if verbose {
            println!("Log info: {:#?}\nWriting the log message...", log_item);
        }
        append_log_item_to_file(&log_item, &log_file_path, quiet, verbose);
    }

    // Read log file
    if !is_write || opts.read {
        let date = match opts.date {
            Some(date_str) => parse_date_from_str(&date_str),
            // Default date is today
            None => now.date_naive(),
        };

        paging_log_file_by_date(log_dir_path, date);
    }
}
