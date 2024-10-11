use std::{
    path::PathBuf,
    process::exit,
    sync::{Arc, Mutex},
};

use chrono::NaiveDate;
use clap::Parser;
use file_utils::create_log_file_if_not_exists;
use log_config::{construct_log_file_path, LogConfig};
use log_item::LogItem;
use log_pager::LogPager;

pub mod cl_args;
pub mod custom_input_classifier;
pub mod file_utils;
pub mod log_config;
pub mod log_item;
pub mod log_pager;

fn paging_log_file_by_date(log_dir_path: PathBuf, date: NaiveDate, verbose: bool) {
    use std::thread;
    let mut log_pager = LogPager::new(date, log_dir_path);
    log_pager.set_verbose(verbose);

    let minus_pager = log_pager.pager().clone();

    let log_pager = Arc::new(Mutex::new(log_pager));

    LogPager::init_input_classifier(&log_pager);

    // Run the pager
    let pager_thread = thread::spawn(move || minus::dynamic_paging(minus_pager));

    // Wait for it to finish
    pager_thread.join().unwrap().unwrap();
}

fn parse_date_from_str(date_str: &str) -> NaiveDate {
    let date_fmt = "%Y-%m-%d";
    let date = NaiveDate::parse_from_str(&date_str, &date_fmt);
    if date.is_err() {
        println!(
            "Invalid date '{}', the date format should be '{}'",
            date_str, date_fmt
        );
        exit(-101);
    }

    date.unwrap()
}

fn view_logs(date_str: Option<String>, verbose: bool, log_dir_path: PathBuf) {
    let now = chrono::prelude::Local::now();

    let date = match date_str {
        Some(date_str) => parse_date_from_str(&date_str),
        // Default date is today
        None => now.date_naive(),
    };

    paging_log_file_by_date(log_dir_path, date, verbose);
}

fn write_log(log_content: String, verbose: bool, log_dir_path: &PathBuf) {
    let now = chrono::prelude::Local::now();

    let log_file_path = construct_log_file_path(log_dir_path, now.date_naive());

    // If the log file does not exist, create it
    create_log_file_if_not_exists(&log_file_path, verbose);

    let log_item = LogItem {
        date_time: now.naive_local(),
        content: log_content,
    };

    if verbose {
        println!("Log info: {:#?}\nWriting the log message...", log_item);
    }

    LogItem::append_to_file(&log_item, &log_file_path, verbose);
}

fn edit_logs(date_str: Option<String>, verbose: bool, log_dir_path: &PathBuf) {
    let now = chrono::prelude::Local::now();
    let date = match date_str {
        Some(date_str) => parse_date_from_str(&date_str),
        // Default date is today
        None => now.date_naive(),
    };

    let log_file_path = construct_log_file_path(log_dir_path, date);

    // If the log file does not exist, create it
    create_log_file_if_not_exists(&log_file_path, verbose);

    if verbose {
        let editor_path_res = edit::get_editor();
        match editor_path_res {
            Ok(editor_path) => {
                println!("Opening editor: {}", editor_path.display());
            }
            Err(_) => {
                println!("Can't find the editor");
            }
        }
    }

    let res = edit::edit_file(log_file_path);
    if res.is_err() {
        println!("Can't edit the file");
    }
}

fn main() {
    LogConfig::create_config_file_if_not_exists();
    let log_config = log_config::LogConfig::from_config_file();
    // let log_dir_path = PathBuf::from("/Users/dnegel3125/Documents/.private/MyLogs");
    let log_dir_path = log_config.log_dir_path;

    if !log_dir_path.exists() {
        println!(
            "The log directory doesn't exist.\nYou may want to configure it in '{}'",
            log_config::CONFIG_FILE_PATH.display()
        );
        exit(1);
    }

    // Command line parameters
    let cli = cl_args::Cli::parse();
    match cli.command {
        cl_args::Commands::View { date, verbose } => {
            view_logs(date, verbose, log_dir_path);
        }
        cl_args::Commands::Write { message, verbose } => write_log(message, verbose, &log_dir_path),
        cl_args::Commands::Config { .. } => todo!(),
        cl_args::Commands::Edit { date, verbose } => {
            edit_logs(date, verbose, &log_dir_path);
        }
    }
}
