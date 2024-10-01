use std::{
    path::PathBuf,
    process::exit,
    sync::{Arc, Mutex},
};

use chrono::NaiveDate;
use file_utils::create_log_file_if_not_exists;
use gumdrop::Options;
use log_config::LogConfig;
use log_item::LogItem;
use log_pager::LogPager;

pub mod cl_params;
pub mod custom_input_classifier;
pub mod file_utils;
pub mod log_config;
pub mod log_item;
pub mod log_pager;

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
    LogConfig::create_config_file_if_not_exists();
    let log_config = log_config::LogConfig::from_config_file();
    // let log_dir_path = PathBuf::from("/Users/dnegel3125/Documents/.private/MyLogs");
    let log_dir_path = log_config.log_dir_path;
    let now = chrono::prelude::Local::now();
    let log_file_path = log_dir_path.join(format!("{}.log", now.format("%Y-%m-%d")));

    // Command line parameters
    let opts = cl_params::MyOptions::parse_args_default_or_exit();
    let verbose = opts.verbose;
    let quiet = opts.quiet;

    if !log_dir_path.exists() && !quiet {
        println!(
            "The log directory doesn't exist.\nYou may want to configure it in '{}'",
            log_config::CONFIG_FILE_PATH.display()
        );
        exit(1);
    }

    let is_write = opts.write.is_some();
    if is_write {
        // If the log file does not exist, create it
        create_log_file_if_not_exists(&log_file_path, verbose);

        let log_item = LogItem {
            date_time: now.naive_local(),
            content: opts.write.unwrap(),
        };

        if verbose {
            println!("Log info: {:#?}\nWriting the log message...", log_item);
        }

        LogItem::append_to_file(&log_item, &log_file_path, quiet, verbose);
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
