use std::{
    fs::{File, OpenOptions},
    io::{self},
    path::PathBuf,
    process::exit,
    sync::{Arc, Mutex},
};

use chrono::NaiveDate;
use gumdrop::Options;
use log_pager::LogPager;
pub mod cl_params;
pub mod custom_input_classifier;
pub mod file_utils;
pub mod log_config;
pub mod log_pager;

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

fn create_file_if_not_exists(file_path: &PathBuf) {
    if file_path.exists() {
        return;
    }

    let result = File::create(file_path);
    if result.is_err() {
        println!("Can't create the file. \n{}", result.err().unwrap());
    }
}

fn append_line_to_file(file_path: &PathBuf, line: &str) -> io::Result<usize> {
    let mut file = OpenOptions::new()
        .write(true) // Enable writing
        .append(true) // Enable appending
        .open(file_path)?; // Open the file

    io::Write::write(&mut file, line.as_bytes()) // Write the line with a newline at the end
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

fn append_log_item_to_file(log_item: &LogItem, log_file_path: &PathBuf, quiet: bool) {
    let result = append_line_to_file(log_file_path, &log_item.to_string());
    if result.is_err() {
        println!("Can't write message to the log file");
        exit(3);
    }

    if !quiet {
        println!(
            r#"Written the log message to "{}""#,
            log_file_path.display()
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
    if !log_dir_path.exists() {
        println!("The log dir doesn't exist");
        exit(1);
    }

    // If the log file does not exist, create it
    create_file_if_not_exists(&log_file_path);

    // Command line parameters
    let opts = cl_params::MyOptions::parse_args_default_or_exit();
    let is_write = opts.write.is_some();
    if is_write {
        let log_item = LogItem {
            date_time: now.naive_local(),
            content: opts.write.unwrap(),
        };
        append_log_item_to_file(&log_item, &log_file_path, opts.quiet);
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
